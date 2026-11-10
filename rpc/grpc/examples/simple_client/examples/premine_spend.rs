//! Spend the 2-of-3 multisig premine ("coin owner" wallet) toward an app wallet,
//! returning the change to the premine P2SH address.
//!
//! The premine is a single P2SH 2-of-3 multisig output (`is_coinbase: false`).
//! This tool derives the redeem script from three 64-hex secp256k1 secrets read
//! from a file (one key per line, order K1, K2, K3), signs with K1 and K2 via the
//! PSKT state machine, verifies the assembled scriptSig locally through the full
//! script engine (`extract_tx`), then submits over gRPC and confirms the mempool.
//!
//! Secrets are read from a file path (never a CLI argument), so they never appear
//! in `ps` output. The caller is responsible for shredding the file.
//!
//! Usage:
//!   premine_spend --keys-file /path/to/3-keys.txt [--check-only]
//!   premine_spend --keys-file /path/to/3-keys.txt --to <addr> --amount <pradha> \
//!                 [--fee <pradha>] [--dry-run] [--url grpc://...]
#![allow(clippy::arithmetic_side_effects)]

use celenium_addresses::{Address, Prefix};
use celenium_consensus_core::config::params::MAINNET_PARAMS;
use celenium_consensus_core::constants::TX_VERSION;
use celenium_consensus_core::hashing::sighash::{calc_schnorr_signature_hash, SigHashReusedValuesUnsync};
use celenium_consensus_core::tx::{TransactionOutpoint, UtxoEntry};
use celenium_grpc_client::GrpcClient;
use celenium_rpc_core::api::rpc::RpcApi;
use celenium_txscript::{
    extract_script_pub_key_address, multisig_redeem_script, opcodes::codes::OpData65,
    pay_to_address_script, pay_to_script_hash_script, script_builder::ScriptBuilder,
};
use celenium_wallet_pskt::prelude::{
    Creator, Inner, InputBuilder, OutputBuilder, PSKT, SignInputOk, Signature,
};
use secp256k1::{Keypair, SECP256K1};
use std::{iter, process::ExitCode};

// Known x-only pubkeys K1..K3 (public, not secret) — sanity-check for key order.
const K1_XONLY: &str = "1b37ab3e285f75fcbbfbd589cb67f1eba0c73256ad2c988c6e332b302b2a3b8c";
const K2_XONLY: &str = "67d8ce2167a91cbabdc7b69a34c03b41afd3d020797dce8681c8b2bb3c30d434";
const K3_XONLY: &str = "fc5cec1121da28ba647338cd68ad63fc245b45c6c2ff46e4c68e47594dbb5ed5";

struct Args {
    url: String,
    keys_file: String,
    to: String,
    amount: u64,
    fee: u64,
    check_only: bool,
    dry_run: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut url = "grpc://127.0.0.1:26110".to_string();
    let mut keys_file = String::new();
    let mut to = String::new();
    let mut amount: u64 = 0;
    let mut fee: u64 = 1_000_000;
    let mut check_only = false;
    let mut dry_run = false;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--url" => url = it.next().ok_or("--url needs a value")?,
            "--keys-file" => keys_file = it.next().ok_or("--keys-file needs a value")?,
            "--to" => to = it.next().ok_or("--to needs a value")?,
            "--amount" => {
                amount = it.next().ok_or("--amount needs a value")?.parse().map_err(|e| format!("bad --amount: {e}"))?
            }
            "--fee" => fee = it.next().ok_or("--fee needs a value")?.parse().map_err(|e| format!("bad --fee: {e}"))?,
            "--check-only" => check_only = true,
            "--dry-run" => dry_run = true,
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    if keys_file.is_empty() {
        return Err("usage: premine_spend --keys-file <3-lines-64-hex> [--check-only]".into());
    }
    if !check_only && (to.is_empty() || amount == 0) {
        return Err("usage: premine_spend --keys-file <f> --to <address> --amount <pradha> [--fee ...] [--dry-run]".into());
    }
    Ok(Args { url, keys_file, to, amount, fee, check_only, dry_run })
}

fn read_three_secrets(path: &str) -> Result<[[u8; 32]; 3], String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let lines: Vec<&str> =
        raw.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).collect();
    if lines.len() != 3 {
        return Err(format!("--keys-file must contain exactly 3 lines of 64-hex (K1,K2,K3), got {}", lines.len()));
    }
    let mut out = [[0u8; 32]; 3];
    for (i, l) in lines.iter().enumerate() {
        let hex = l.trim_start_matches("0x");
        let bytes = hex::decode(hex).map_err(|e| format!("line {}: bad hex: {e}", i + 1))?;
        let arr: [u8; 32] =
            bytes.try_into().map_err(|v: Vec<u8>| format!("line {}: {} bytes, want 32", i + 1, v.len()))?;
        out[i] = arr;
    }
    Ok(out)
}

fn load_keypairs(secrets: &[[u8; 32]; 3]) -> Result<[Keypair; 3], String> {
    let kps = [
        Keypair::from_seckey_slice(SECP256K1, &secrets[0]).map_err(|e| format!("key 1 invalid: {e}"))?,
        Keypair::from_seckey_slice(SECP256K1, &secrets[1]).map_err(|e| format!("key 2 invalid: {e}"))?,
        Keypair::from_seckey_slice(SECP256K1, &secrets[2]).map_err(|e| format!("key 3 invalid: {e}"))?,
    ];
    let expected = [K1_XONLY, K2_XONLY, K3_XONLY];
    for (i, kp) in kps.iter().enumerate() {
        let xonly = hex::encode(kp.x_only_public_key().0.serialize());
        if xonly != expected[i] {
            return Err(format!(
                "key {} x-only pubkey does not match K{}: derived {} expected {}",
                i + 1,
                i + 1,
                xonly,
                expected[i]
            ));
        }
    }
    Ok(kps)
}

fn pradha_to_cen(pradha: u64) -> f64 {
    pradha as f64 / 100_000_000.0
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            println!("PREMINE_SPEND_FAILED: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), String> {
    let args = parse_args()?;
    let secrets = read_three_secrets(&args.keys_file)?;
    let kps = load_keypairs(&secrets)?;

    // Derive the 2-of-3 redeem script and the P2SH source address.
    let redeem_script: Vec<u8> =
        multisig_redeem_script(kps.iter().map(|kp| kp.x_only_public_key().0.serialize()), 2)
            .map_err(|e| format!("multisig_redeem_script: {e}"))?;
    let p2sh_spk = pay_to_script_hash_script(&redeem_script);
    let source = extract_script_pub_key_address(&p2sh_spk, Prefix::Mainnet)
        .map_err(|e| format!("extract source address: {e}"))?;
    let dest = if args.to.is_empty() {
        None
    } else {
        Some(Address::try_from(args.to.as_str()).map_err(|e| format!("bad --to address: {e}"))?)
    };

    println!("source address : {source}  (premine P2SH 2-of-3)");
    if let Some(d) = &dest {
        println!("dest address   : {d}");
    }

    let client = GrpcClient::connect_with_args(
        celenium_rpc_core::notify::mode::NotificationMode::Direct,
        args.url.clone(),
        None,
        false,
        None,
        false,
        Some(500_000),
        Default::default(),
    )
    .await
    .map_err(|e| format!("connect to {} failed: {e}", args.url))?;

    let info = client.get_server_info().await.map_err(|e| format!("get_server_info: {e}"))?;
    println!("node           : is_synced={} has_utxo_index={} network={}", info.is_synced, info.has_utxo_index, info.network_id);
    if !info.has_utxo_index {
        return Err("node has no utxoindex — start celeniumd with --utxoindex".into());
    }
    if !info.is_synced {
        println!("WARN: node reports not synced (expected with --enable-unsynced-mining on an empty DAG); continuing");
    }

    let dag = client.get_block_dag_info().await.map_err(|e| format!("get_block_dag_info: {e}"))?;
    let daa = dag.virtual_daa_score;
    println!("virtual DAA    : {daa}");

    let mut query = vec![source.clone()];
    if let Some(d) = &dest {
        query.push(d.clone());
    }
    let balances = client
        .get_balances_by_addresses(query)
        .await
        .map_err(|e| format!("get_balances_by_addresses: {e}"))?;
    println!("--- balances ---");
    for b in &balances {
        let bal = b.balance.unwrap_or(0);
        println!("  {}  {} CEN ({} pradha)", b.address, pradha_to_cen(bal), bal);
    }

    let utxos = client
        .get_utxos_by_addresses(vec![source.clone()])
        .await
        .map_err(|e| format!("get_utxos_by_addresses: {e}"))?;
    println!("source utxos   : {}", utxos.len());

    if args.check_only {
        for u in &utxos {
            let entry: UtxoEntry = u.utxo_entry.clone().into();
            println!(
                "  {} idx {} amount {} CEN daa {} coinbase {}",
                u.outpoint.transaction_id,
                u.outpoint.index,
                pradha_to_cen(entry.amount),
                entry.block_daa_score,
                entry.is_coinbase
            );
        }
        client.disconnect().await.map_err(|e| format!("disconnect: {e}"))?;
        println!("CHECK_ONLY_DONE");
        return Ok(());
    }

    // Pick the (single) premine UTXO.
    let mut chosen: Option<(TransactionOutpoint, UtxoEntry)> = None;
    for u in &utxos {
        let entry: UtxoEntry = u.utxo_entry.clone().into();
        if chosen.is_none() && entry.amount > args.amount + args.fee {
            chosen = Some((u.outpoint.into(), entry));
        }
    }
    let (outpoint, entry) =
        chosen.ok_or_else(|| "no spendable premine UTXO with enough value".to_string())?;
    println!("spending       : idx {} amount {} CEN", outpoint.index, pradha_to_cen(entry.amount));

    let total_in = entry.amount;
    let change = total_in - args.amount - args.fee;
    println!("change         : {} CEN -> back to premine P2SH", pradha_to_cen(change));

    // --- assemble via PSKT: Creator -> Constructor -> Updater -> Signer -> Finalizer -> Extractor ---
    let input_0 = InputBuilder::default()
        .utxo_entry(entry)
        .previous_outpoint(outpoint)
        .sig_op_count(2)
        .redeem_script(redeem_script)
        .build()
        .map_err(|e| format!("input build: {e}"))?;

    let output_dest = OutputBuilder::default()
        .amount(args.amount)
        .script_public_key(pay_to_address_script(dest.as_ref().ok_or("missing --to address")?))
        .build()
        .map_err(|e| format!("dest output build: {e}"))?;

    let output_change = OutputBuilder::default()
        .amount(change)
        .script_public_key(p2sh_spk.clone())
        .build()
        .map_err(|e| format!("change output build: {e}"))?;

    let constructor = PSKT::<Creator>::default()
        .inputs_modifiable()
        .outputs_modifiable()
        .constructor()
        .set_tx_version(TX_VERSION)
        .input(input_0)
        .output(output_dest)
        .map_err(|e| format!("add dest output: {e}"))?
        .output(output_change)
        .map_err(|e| format!("add change output: {e}"))?;

    let signer = constructor
        .updater()
        .set_sequence(u64::MAX, 0)
        .map_err(|e| format!("set_sequence: {e}"))?
        .signer();

    let reused_values = SigHashReusedValuesUnsync::new();
    let signer = signer
        .pass_signature_sync(|tx, sighash| -> Result<Vec<SignInputOk>, String> {
            Ok(tx
                .tx
                .inputs
                .iter()
                .enumerate()
                .map(|(idx, _input)| {
                    let hash = calc_schnorr_signature_hash(&tx.as_verifiable(), idx, sighash[idx], &reused_values);
                    let msg = secp256k1::Message::from_digest_slice(hash.as_bytes().as_slice()).unwrap();
                    SignInputOk {
                        signature: Signature::Schnorr(kps[0].sign_schnorr(msg)),
                        pub_key: kps[0].public_key(),
                        key_source: None,
                    }
                })
                .collect())
        })
        .map_err(|e| format!("sign K1: {e}"))?;
    let signer = signer
        .pass_signature_sync(|tx, sighash| -> Result<Vec<SignInputOk>, String> {
            Ok(tx
                .tx
                .inputs
                .iter()
                .enumerate()
                .map(|(idx, _input)| {
                    let hash = calc_schnorr_signature_hash(&tx.as_verifiable(), idx, sighash[idx], &reused_values);
                    let msg = secp256k1::Message::from_digest_slice(hash.as_bytes().as_slice()).unwrap();
                    SignInputOk {
                        signature: Signature::Schnorr(kps[1].sign_schnorr(msg)),
                        pub_key: kps[1].public_key(),
                        key_source: None,
                    }
                })
                .collect())
        })
        .map_err(|e| format!("sign K2: {e}"))?;

    let finalizer = signer.finalizer();
    let finalizer = finalizer
        .finalize_sync(|inner: &Inner| -> Result<Vec<Vec<u8>>, String> {
            Ok(inner
                .inputs
                .iter()
                .map(|input| -> Vec<u8> {
                    let signatures: Vec<_> = [&kps[0], &kps[1]]
                        .iter()
                        .flat_map(|kp| {
                            let sig = input.partial_sigs.get(&kp.public_key()).unwrap().into_bytes();
                            iter::once(OpData65).chain(sig).chain([input.sighash_type.to_u8()])
                        })
                        .collect();
                    signatures
                        .into_iter()
                        .chain(
                            ScriptBuilder::new()
                                .add_data(input.redeem_script.as_ref().unwrap().as_slice())
                                .unwrap()
                                .drain()
                                .iter()
                                .cloned(),
                        )
                        .collect()
                })
                .collect())
        })
        .map_err(|e| format!("finalize: {e}"))?;

    let extractor = finalizer.extractor().map_err(|e| format!("extractor: {e}"))?;
    let tx = extractor.extract_tx(&MAINNET_PARAMS).map_err(|e| format!("extract_tx: {e}"))?.tx;
    let txid = tx.id();
    println!("local verify   : OK (extract_tx ran the full script engine)");
    println!("tx id          : {txid}");

    if args.dry_run {
        client.disconnect().await.map_err(|e| format!("disconnect: {e}"))?;
        println!("DRY_RUN_DONE — not submitted");
        return Ok(());
    }

    let rpc_tx: celenium_rpc_core::RpcTransaction = (&tx).into();
    let submitted = client
        .submit_transaction(rpc_tx, false)
        .await
        .map_err(|e| format!("submit_transaction rejected: {e}"))?;
    println!("submitted      : {submitted}");

    if submitted != txid {
        return Err(format!("returned transaction id {submitted} does not match the sent tx {txid}"));
    }

    let mempool = client
        .get_mempool_entry(txid, true, false)
        .await
        .map_err(|e| format!("get_mempool_entry: {e}"))?;
    println!("mempool        : accepted (fee {} pradha)", mempool.fee);

    client.disconnect().await.map_err(|e| format!("disconnect: {e}"))?;
    println!();
    println!("PREMINE_SPEND_PASSED — multisig premine spent, change returned to coin owner");
    Ok(())
}