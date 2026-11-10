//! Query live Celenium mainnet balances + UTXOs by address over gRPC.
//!
//! Read-only verification: derives nothing, signs nothing, spends nothing.
//! Defaults to the premine owner+dev addresses when no args are given.
//!
//! Usage:
//!   cargo run -p celenium-grpc-simple-client-example --example balance -- \
//!       [--url grpc://127.0.0.1:26110] [addr1 addr2 ...]

use celenium_addresses::Address;
use celenium_consensus_core::tx::UtxoEntry;
use celenium_grpc_client::GrpcClient;
use celenium_rpc_core::api::rpc::RpcApi;
use celenium_rpc_core::notify::mode::NotificationMode;
use std::process::ExitCode;

const OWNER: &str = "cen69:qze3sl4udchv2ykhwve274th0mafrga0cea4ljm9vz6mmy0puxh4gjunasrfn";
const DEV: &str = "cen69:qp369ngwtlyryr5nn87j2z9f70dytku5qx0kr2ys604y9mej8nu7vxtt8wf5a";

fn pradha_to_cen(pradha: u64) -> f64 {
    pradha as f64 / 100_000_000.0
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            println!("BALANCE_FAILED: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), String> {
    let mut url = "grpc://127.0.0.1:26110".to_string();
    let mut addrs: Vec<String> = vec![];
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--url" => url = it.next().ok_or("--url needs a value")?,
            other => addrs.push(other.to_string()),
        }
    }
    if addrs.is_empty() {
        addrs = vec![OWNER.to_string(), DEV.to_string()];
    }

    let client = GrpcClient::connect_with_args(
        NotificationMode::Direct,
        url.clone(),
        None,
        false,
        None,
        false,
        Some(500_000),
        Default::default(),
    )
    .await
    .map_err(|e| format!("connect to {} failed: {e}", url))?;

    let info = client.get_server_info().await.map_err(|e| format!("get_server_info: {e}"))?;
    println!("node            : v{} (network {})", info.server_version, info.network_id);
    println!("node synced     : {}", info.is_synced);
    println!("utxo index      : {}", info.has_utxo_index);

    let dag = client.get_block_dag_info().await.map_err(|e| format!("get_block_dag_info: {e}"))?;
    println!("virtual DAA     : {}", dag.virtual_daa_score);
    println!("difficulty      : {}", dag.difficulty);
    println!("pruning point   : {}", dag.pruning_point_hash);

    let parsed: Vec<Address> = addrs
        .iter()
        .map(|s| Address::try_from(s.as_str()).map_err(|e| format!("bad address {s}: {e}")))
        .collect::<Result<_, _>>()?;

    let balances = client
        .get_balances_by_addresses(parsed.clone())
        .await
        .map_err(|e| format!("get_balances_by_addresses: {e}"))?;

    println!();
    println!("--- balances ({} address) ---", balances.len());
    for b in &balances {
        let bal = b.balance.unwrap_or(0);
        println!("  {}  {:.8} CEN  ({} pradha)", b.address, pradha_to_cen(bal), bal);
    }

    // UTXO detail per address
    for a in &parsed {
        let utxos = client
            .get_utxos_by_addresses(vec![a.clone()])
            .await
            .map_err(|e| format!("get_utxos_by_addresses: {e}"))?;
        println!();
        println!("--- {} ({} utxo) ---", a, utxos.len());
        for u in &utxos {
            let entry: UtxoEntry = u.utxo_entry.clone().into();
            println!(
                "  tx {} idx {}  amount {:.8} CEN  daa {}  coinbase {}",
                u.outpoint.transaction_id,
                u.outpoint.index,
                pradha_to_cen(entry.amount),
                entry.block_daa_score,
                entry.is_coinbase
            );
        }
    }

    client.disconnect().await.map_err(|e| format!("disconnect: {e}"))?;
    println!();
    println!("BALANCE_OK");
    Ok(())
}