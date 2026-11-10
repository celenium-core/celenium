//! Generate 3 fresh secp256k1 keys for the CEN owner vault (2-of-3, K4/K5/K6).
//!
//! Usage: `gen_owner_multisig [privkey_out_path]`
//!   - prints the three x-only PUBLIC keys (hex, K4/K5/K6 order) to stdout ONLY;
//!   - if a path is given, writes the three PRIVATE keys (hex, one per line,
//!     `K4_PRIVKEY=` / `K5_PRIVKEY=` / `K6_PRIVKEY=`) there with mode 0600.
//! Private keys are never printed to stdout or logged.
//!
//! CSPRNG via `rand::thread_rng` (OsRng-backed). The caller GPG-encrypts the
//! private-key file out-of-band and shreds the plaintext.

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use rand::RngCore;
use secp256k1::{PublicKey, SecretKey};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn gen_key() -> SecretKey {
    loop {
        let mut seed = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut seed);
        if let Ok(k) = SecretKey::from_slice(&seed) {
            return k;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keys: Vec<SecretKey> = (0..3).map(|_| gen_key()).collect();

    // Public x-only keys — safe to print.
    for (i, key) in keys.iter().enumerate() {
        let public = PublicKey::from_secret_key_global(key);
        let (xonly, _) = public.x_only_public_key();
        println!("K{}={}", i + 4, hex(&xonly.serialize()));
    }

    // Private keys — written to the caller-specified path only, never stdout.
    if let Some(path) = env::args().nth(1) {
        let mut content = String::new();
        for (i, key) in keys.iter().enumerate() {
            content.push_str(&format!("K{}_PRIVKEY={}\n", i + 4, hex(&key.secret_bytes())));
        }
        fs::write(&path, &content)?;
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)?;
    }

    Ok(())
}