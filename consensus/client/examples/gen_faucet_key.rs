//! Generates the faucet wallet keypair (operator-side signer for withdrawals).
//!
//! Usage: `gen_faucet_key <out-path>`
//!
//! Writes `privkey=<hex>` to `out-path` (meant for the encrypted key vault on E:)
//! and prints the derived `cen69:` PubKey address to stdout. The secret comes
//! from `/dev/urandom`, never from a seed phrase or passphrase.

use celenium_addresses::{Address, Prefix, Version};
use secp256k1::{Keypair, Secp256k1};
use std::io::Read;

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = std::env::args().nth(1).ok_or("usage: gen_faucet_key <out-path>")?;

    let mut secret = [0u8; 32];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut secret)?;

    let secp = Secp256k1::new();
    let keypair = Keypair::from_seckey_slice(&secp, &secret)?;
    let (xonly, _parity) = keypair.x_only_public_key();

    let address = Address::new(Prefix::Mainnet, Version::PubKey, &xonly.serialize());
    let pubkey_hex = hex(&xonly.serialize());

    std::fs::write(
        &out_path,
        format!("label=faucet-mainnet\nprivkey={}\npubkey={}\naddress={}\n", hex(&secret), pubkey_hex, address),
    )?;

    println!("FAUCET_ADDRESS={address}");
    println!("FAUCET_PUBKEY={pubkey_hex}");
    println!("FAUCET_KEYFILE={out_path}");
    Ok(())
}
