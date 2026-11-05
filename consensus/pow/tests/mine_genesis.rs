//! Mines fresh PoW nonces for the genesis blocks with attainable targets.
//!
//! Run after any change to genesis contents (payload, premine, timestamp,
//! daa_score) and after `print_genesis_regeneration_values` + apply step:
//!
//! ```text
//! cargo test -p celenium-pow --release mine_genesis_nonces -- --ignored --nocapture
//! ```
//!
//! Mainnet is deliberately skipped: its genesis bits (0x1d00ffff) put the
//! target near 2^224 (~2^-32 hit probability), so a naive search cannot find
//! a nonce. Consensus exempts genesis from PoW validation (same as upstream
//! rusty-kaspa), so the mainnet nonce stays the inherited value.
//!
//! [ignore]d by default: normal test runs must not spin on mining.

use celenium_consensus_core::{
    block::Block,
    config::genesis::{DEVNET_GENESIS, SIMNET_GENESIS, TESTNET_GENESIS},
};
use celenium_pow::State;

#[test]
#[ignore]
fn mine_genesis_nonces() {
    let cap: u64 = 2_000_000_000;
    for (name, genesis) in [
        ("SIMNET_GENESIS", &SIMNET_GENESIS),
        ("TESTNET_GENESIS", &TESTNET_GENESIS),
        ("DEVNET_GENESIS", &DEVNET_GENESIS),
    ] {
        println!("mining {name}...");
        let block: Block = genesis.into();
        let state = State::new(&block.header);
        let start = std::time::Instant::now();
        let mut nonce = None;
        for i in 0u64..cap {
            if let (true, _) = state.check_pow(i) {
                nonce = Some(i);
                break;
            }
        }
        match nonce {
            Some(n) => println!(
                "{name}: nonce = {n} (0x{n:x}) dalam {:.1?}",
                start.elapsed()
            ),
            None => println!(
                "{name}: tidak ditemukan dalam {cap} percobaan ({:.1?}) — target terlalu sulit",
                start.elapsed()
            ),
        }
    }
}
