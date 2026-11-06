//! Demonstrates the Celenium wallet mnemonic ("12 secret words") end-to-end.
//!
//! 1. Cross-checks the manual BIP32 path m/44'/111111'/0'/0/{i} against a known
//!    mnemonic->address vector produced by the wallet's own WalletDerivationManager.
//! 2. Generates a fresh 12-word mnemonic, derives its receive[0] simnet address and
//!    the raw 32-byte signing secret.
//! 3. Restores from the phrase and confirms the SAME secret + address come back.

use std::str::FromStr;

use celenium_addresses::{Address, Prefix, Version};
use celenium_bip32::{
    DerivationPath, ExtendedPrivateKey, Language, Mnemonic, SecretKey, WordCount,
};
use secp256k1::Secp256k1;

fn hex32(b: &[u8; 32]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn xonly(secret: &SecretKey) -> [u8; 32] {
    let secp = Secp256k1::new();
    secret.public_key(&secp).x_only_public_key().0.serialize()
}

fn address_of(secret: &SecretKey, prefix: Prefix) -> String {
    Address::new(prefix, Version::PubKey, &xonly(secret)).to_string()
}

fn receive_secret(mnemonic: &Mnemonic, index: u32) -> Result<SecretKey, Box<dyn std::error::Error>> {
    let master = ExtendedPrivateKey::<SecretKey>::new(mnemonic.to_seed(""))?;
    let path = DerivationPath::from_str(&format!("m/44'/111111'/0'/0/{index}"))?;
    let child = master.derive_path(&path)?;
    Ok(*child.private_key())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kprv_prefix = celenium_bip32::Prefix::KPRV;

    // 1. Known-answer cross-check (24 words, receive index 1, testnet).
    let known = "hunt bitter praise lift buyer topic crane leopard uniform network inquiry over \
                 grain pass match crush marine strike doll relax fortune trumpet sunny silk";
    let m = Mnemonic::new(known, Language::English)?;
    let s = receive_secret(&m, 1)?;
    let a = address_of(&s, Prefix::Testnet);
    assert_eq!(
        a,
        "cen69test:qrc2959g0pqda53glnfd238cdnmk24zxzkj8n5x83rkktx4h73dkcqjkk5zdg",
        "manual BIP32 path disagrees with wallet's own derivation"
    );
    println!("[VECTOR] manual path m/44'/111111'/0'/0/1 matches known testnet address -> OK");

    // 2. Fresh 12-word mnemonic (or restore from argv[1]).
    let fresh = match std::env::args().nth(1) {
        Some(p) if !p.is_empty() => {
            println!("[RESTORE] importing phrase from argv");
            Mnemonic::new(&p, Language::English)?
        }
        _ => Mnemonic::random(WordCount::Words12, Language::English)?,
    };
    let phrase = fresh.phrase().to_string();
    let wc = phrase.split_whitespace().count();
    let secret = receive_secret(&fresh, 0)?;
    let addr = address_of(&secret, Prefix::Simnet);
    let master = ExtendedPrivateKey::<SecretKey>::new(fresh.to_seed(""))?;
    let kprv = master.to_string(kprv_prefix).to_string();

    // 3. Restore from the phrase.
    let restored = Mnemonic::new(phrase.as_str(), Language::English)?;
    let secret2 = receive_secret(&restored, 0)?;
    let addr2 = address_of(&secret2, Prefix::Simnet);
    assert_eq!(secret.secret_bytes(), secret2.secret_bytes(), "restored secret mismatch");
    assert_eq!(addr, addr2, "restored address mismatch");

    println!("[FRESH] word-count  : {wc}");
    println!("[FRESH] mnemonic    : {phrase}");
    println!("[FRESH] master-kprv : {kprv}");
    println!("[FRESH] secret-hex  : {}", hex32(&secret.secret_bytes()));
    println!("[FRESH] sim-address : {addr}");
    println!("[RESTORE] same secret + same address after re-import -> OK");

    Ok(())
}