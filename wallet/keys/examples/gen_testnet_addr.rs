//! Generate a fresh Celenium testnet address + secret for the CPU miner reward.
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mneh = Mnemonic::random(WordCount::Words12, Language::English)?;
    let master = ExtendedPrivateKey::<SecretKey>::new(mneh.to_seed(""))?;
    let path = DerivationPath::from_str("m/44'/111111'/0'/0/0")?;
    let child = master.derive_path(&path)?;
    let secret = child.private_key();
    let addr = address_of(secret, Prefix::Testnet);
    println!("mnemonic: {}", mneh.phrase());
    println!("secret_hex: {}", hex32(&secret.secret_bytes()));
    println!("testnet_address: {}", addr);
    Ok(())
}