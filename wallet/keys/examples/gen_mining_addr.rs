//! Generate a fresh CEN mainnet mining (reward) address + private key.
//!
//! Usage: `gen_mining_addr [privkey_out_path]`
//!   - prints the ADDRESS (public) to stdout only;
//!   - if a path is given, writes the PRIVATE KEY hex there (mode 0600).
//! The private key is never printed to stdout/logs.

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use celenium_addresses::{Address, Prefix, Version};
use rand::RngCore;
use secp256k1::{PublicKey, SecretKey};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secret_key = {
        let mut seed = [0u8; 32];
        loop {
            rand::thread_rng().fill_bytes(&mut seed);
            if let Ok(k) = SecretKey::from_slice(&seed) {
                break k;
            }
        }
    };

    let public_key = PublicKey::from_secret_key_global(&secret_key);
    let (xonly, _) = public_key.x_only_public_key();
    let address = Address::new(Prefix::Mainnet, Version::PubKey, &xonly.serialize());

    if let Some(path) = env::args().nth(1) {
        fs::write(&path, hex(&secret_key.secret_bytes()))?;
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)?;
    }

    println!("{address}");
    Ok(())
}