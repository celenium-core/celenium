//! Temporary tool: generate premine keypairs for the CEN genesis block.
//!
//! Produces raw 32-byte secp256k1 secrets and their x-only public keys, in the
//! exact format expected by `pay_to_pubkey_script` in
//! `consensus/core/src/config/genesis.rs` (see `sign.rs` for the same pattern).
//!
//! PRIVATE KEYS MUST NEVER ENTER THE REPOSITORY. Output is meant to be
//! redirected to a private location outside the project tree.

use secp256k1::{Keypair, SECP256K1};
use std::fs::File;
use std::io::{Read, Write};

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// 32 bytes from the OS CSPRNG. Not derived from anything — pure entropy.
fn random_secret() -> [u8; 32] {
    let mut secret = [0u8; 32];
    let mut f = File::open("/dev/urandom").expect("open /dev/urandom");
    f.read_exact(&mut secret).expect("read 32 bytes of entropy");
    secret
}

fn main() {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    for label in ["dev", "owner"] {
        let secret = random_secret();
        let kp = Keypair::from_seckey_slice(SECP256K1, &secret).expect("valid secret key");
        let pubkey = kp.public_key().x_only_public_key().0;

        // Sanity: the derived script must match pay_to_pubkey_script's output.
        let mut script = vec![0x20u8];
        script.extend_from_slice(&pubkey.serialize());
        script.push(0xac);
        assert_eq!(script.len(), 34, "P2PK script must be 34 bytes");

        writeln!(out, "LABEL={label}").unwrap();
        writeln!(out, "PRIVKEY={}", hex(&secret)).unwrap();
        writeln!(out, "PUBKEY={}", hex(&pubkey.serialize())).unwrap();
        writeln!(out, "SCRIPT={}", hex(&script)).unwrap();
        writeln!(out).unwrap();
    }
}
