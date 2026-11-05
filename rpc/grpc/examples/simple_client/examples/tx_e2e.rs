//! End-to-end transaction test against a live Celenium node.
//!
//! Spends a real UTXO over gRPC: reads a 32-byte secp256k1 secret from a file,
//! derives the P2PK address, fetches its mature UTXOs, builds a
//! 1-input / 2-output transaction, signs it with the project's own signer,
//! submits it, and confirms acceptance in the mempool.
//!
//! This exercises the full on-chain path that the wallet CLI would take, but
//! without a TTY: genesis pubkey -> address -> UTXO -> sign -> mempool.
//!
//! The secret is read from a file path (never a CLI argument, so it never
//! appears in `ps` output). The caller is responsible for shredding it.
//!
//! Usage:
//!   tx_e2e --url grpc://127.0.0.1:26510 \
//!          --privkey-file /path/to/hex \
//!          --to cen69sim:qq... --amount 1000000000000 --fee 1000000

use celenium_addresses::{Address, Prefix, Version};
use celenium_consensus_core::constants::{MAX_TX_IN_SEQUENCE_NUM, TX_VERSION};
use celenium_consensus_core::subnets::SUBNETWORK_ID_NATIVE;
use celenium_consensus_core::tx::{
    SignableTransaction, Transaction, TransactionInput, TransactionOutpoint, TransactionOutput, UtxoEntry,
};
use celenium_consensus_core::{sign, tx::ScriptPublicKey};
use celenium_grpc_client::GrpcClient;
use celenium_rpc_core::api::rpc::RpcApi;
use celenium_txscript::standard::pay_to_address_script;
use secp256k1::{Keypair, SECP256K1};
use std::process::ExitCode;

/// Coinbase outputs are spendable only after this many DAA blocks (10 BPS params).
const COINBASE_MATURITY_DAA: u64 = 200;

struct Args {
    url: String,
    privkey_file: String,
    to: String,
    amount: u64,
    fee: u64,
    check_only: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut url = "grpc://127.0.0.1:26510".to_string();
    let mut privkey_file = String::new();
    let mut to = String::new();
    let mut amount: u64 = 0;
    let mut fee: u64 = 1_000_000;
    let mut check_only = false;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--url" => url = it.next().ok_or("--url needs a value")?,
            "--privkey-file" => privkey_file = it.next().ok_or("--privkey-file needs a value")?,
            "--to" => to = it.next().ok_or("--to needs a value")?,
            "--amount" => amount = it.next().ok_or("--amount needs a value")?.parse().map_err(|e| format!("bad --amount: {e}"))?,
            "--fee" => fee = it.next().ok_or("--fee needs a value")?.parse().map_err(|e| format!("bad --fee: {e}"))?,
            "--check-only" => check_only = true,
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    if privkey_file.is_empty() {
        return Err("usage: tx_e2e --privkey-file <hex> [--check-only] | --to <address> --amount <pradha>".into());
    }
    if !check_only && (to.is_empty() || amount == 0) {
        return Err("usage: tx_e2e --privkey-file <hex> --to <address> --amount <pradha> [--url ...] [--fee ...]".into());
    }
    Ok(Args { url, privkey_file, to, amount, fee, check_only })
}

fn read_secret(path: &str) -> Result<[u8; 32], String> {
    let raw = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let trimmed = raw.trim();
    // accept an optional "PRIVKEY=" / "PRIVKEY:" style label, and 0x prefix
    let hex = trimmed
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .and_then(|l| l.split(['=', ':', ' ']).find(|s| !s.is_empty()))
        .unwrap_or(trimmed)
        .trim_start_matches("0x");
    let bytes = hex::decode(hex).map_err(|e| format!("secret is not valid hex: {e}"))?;
    bytes.try_into().map_err(|v: Vec<u8>| format!("secret must be 32 bytes, got {}", v.len()))
}

/// Builds the P2PK script `0x20 <xonly-pubkey> 0xac`, the same form used by
/// `sign_with_multiple_v2` and by the genesis premine outputs.
fn p2pk_script(pubkey: &secp256k1::XOnlyPublicKey) -> Vec<u8> {
    std::iter::once(0x20u8).chain(pubkey.serialize()).chain(std::iter::once(0xacu8)).collect()
}

fn pradha_to_cen(pradha: u64) -> f64 {
    pradha as f64 / 100_000_000.0
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            println!("TX_E2E_FAILED: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), String> {
    let args = parse_args()?;
    let secret = read_secret(&args.privkey_file)?;

    let keypair = Keypair::from_seckey_slice(SECP256K1, &secret).map_err(|e| format!("invalid secret key: {e}"))?;
    let xonly = keypair.public_key().x_only_public_key().0;
    let script = p2pk_script(&xonly);
    let spk = ScriptPublicKey::new(0, celenium_consensus_core::tx::ScriptVec::from_slice(&script));

    // Derive the source address from the same script, so what we query matches what we sign.
    let source = Address::new(Prefix::Simnet, Version::PubKey, &xonly.serialize());
    let dest = if args.to.is_empty() {
        None
    } else {
        Some(Address::try_from(args.to.as_str()).map_err(|e| format!("bad --to address: {e}"))?)
    };
    let dest_spk = dest.as_ref().map(pay_to_address_script);

    println!("source address : {source}");
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
    println!("node synced    : {} (network {})", info.is_synced, info.network_id);
    if !info.is_synced {
        return Err("node is not synced — the genesis timestamp sync window has expired; regenerate the timestamp".into());
    }
    if !info.has_utxo_index {
        return Err("node has no utxoindex — start celeniumd with --utxoindex".into());
    }

    let dag = client.get_block_dag_info().await.map_err(|e| format!("get_block_dag_info: {e}"))?;
    let daa = dag.virtual_daa_score;
    println!("virtual DAA    : {daa}");

    // --- balance before ---
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
        let addr = b.address.to_string();
        let bal = b.balance.unwrap_or(0);
        println!("  {addr}  {} CEN ({} pradha)", pradha_to_cen(bal), bal);
    }

    if args.check_only {
        let utxos = client
            .get_utxos_by_addresses(vec![source.clone()])
            .await
            .map_err(|e| format!("get_utxos_by_addresses: {e}"))?;
        println!("--- source utxos ({}) ---", utxos.len());
        for u in &utxos {
            let entry: UtxoEntry = u.utxo_entry.clone().into();
            let mature = if entry.is_coinbase {
                entry.block_daa_score + COINBASE_MATURITY_DAA <= daa
            } else {
                true
            };
            println!(
                "  {} idx {} amount {} CEN daa {} coinbase {} mature {}",
                u.outpoint.transaction_id,
                u.outpoint.index,
                pradha_to_cen(entry.amount),
                entry.block_daa_score,
                entry.is_coinbase,
                mature
            );
        }
        client.disconnect().await.map_err(|e| format!("disconnect: {e}"))?;
        println!("CHECK_ONLY_DONE");
        return Ok(());
    }

    // --- pick a spendable UTXO ---
    let utxos = client
        .get_utxos_by_addresses(vec![source.clone()])
        .await
        .map_err(|e| format!("get_utxos_by_addresses: {e}"))?;
    println!("utxos found    : {}", utxos.len());

    let mut chosen: Option<(TransactionOutpoint, UtxoEntry)> = None;
    for u in &utxos {
        let entry: UtxoEntry = u.utxo_entry.clone().into();
        let mature = if entry.is_coinbase {
            entry.block_daa_score + COINBASE_MATURITY_DAA <= daa
        } else {
            true
        };
        println!(
            "  outpoint {} idx {} amount {} CEN daa {} coinbase {} mature {}",
            u.outpoint.transaction_id,
            u.outpoint.index,
            pradha_to_cen(entry.amount),
            entry.block_daa_score,
            entry.is_coinbase,
            mature
        );
        if chosen.is_none() && mature && entry.amount > args.amount + args.fee {
            chosen = Some((u.outpoint.into(), entry));
        }
    }

    let (outpoint, entry) =
        chosen.ok_or_else(|| "no mature UTXO with enough value — mine more blocks or wait for coinbase maturity".to_string())?;
    println!("spending       : outpoint idx {} amount {} CEN", outpoint.index, pradha_to_cen(entry.amount));

    // --- build the transaction ---
    // 1 input, 2 outputs (payment + change). Fee is subtracted from the change output.
    let total_in = entry.amount;
    let change = total_in - args.amount - args.fee;
    let input = TransactionInput::new(outpoint, vec![], MAX_TX_IN_SEQUENCE_NUM, 1);
    let payment_output = TransactionOutput::new(args.amount, dest_spk.ok_or("missing --to address")?);
    let change_output = TransactionOutput::new(change, spk.clone());
    let tx = Transaction::new(
        TX_VERSION,
        vec![input],
        vec![payment_output, change_output],
        0,
        SUBNETWORK_ID_NATIVE,
        0,
        vec![],
    );

    let signable = SignableTransaction::with_entries(tx, vec![entry]);
    let signed = sign::sign_with_multiple_v2(signable, &[secret]);
    let signed = match signed {
        sign::Signed::Fully(t) => t,
        sign::Signed::Partially(_) => return Err("transaction is only partially signed".into()),
    };

    // Sanity: verify the signature locally before bothering the node.
    sign::verify(&signed.as_verifiable()).map_err(|e| format!("local signature verification failed: {e}"))?;
    println!("local verify   : OK");

    let tx = signed.tx;
    let txid = tx.id();
    println!("tx id          : {txid}");

    let rpc_tx: celenium_rpc_core::RpcTransaction = (&tx).into();
    let submitted = client
        .submit_transaction(rpc_tx, false)
        .await
        .map_err(|e| format!("submit_transaction rejected: {e}"))?;
    println!("submitted      : {submitted}");

    if submitted != txid {
        return Err(format!("returned transaction id {submitted} does not match the sent tx {txid}"));
    }

    // --- confirm the node actually holds it in the mempool ---
    let entry = client
        .get_mempool_entry(txid, true, false)
        .await
        .map_err(|e| format!("get_mempool_entry: {e}"))?;
    println!("mempool        : accepted (fee {} pradha)", entry.fee);

    client.disconnect().await.map_err(|e| format!("disconnect: {e}"))?;

    println!();
    println!("TX_E2E_PASSED — transaction signed, submitted, and accepted by the node");
    Ok(())
}
