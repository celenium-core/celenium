//! Deterministic verifier for the CEN premine 2-of-3 multisig P2SH.
//!
//! Hardcodes the three premine public keys (committed in
//! `consensus/core/src/config/genesis.rs`) and reproduces the redeem script,
//! the 35-byte P2SH script and the mainnet address. Run it to prove the P2SH
//! bytes hardcoded in genesis.rs are derived correctly from these keys — no
//! randomness, no private keys.

use std::fmt::Write as _;

use celenium_addresses::Prefix;
use celenium_txscript::{extract_script_pub_key_address, multisig_redeem_script, pay_to_script_hash_script};

const K1: &str = "1b37ab3e285f75fcbbfbd589cb67f1eba0c73256ad2c988c6e332b302b2a3b8c";
const K2: &str = "67d8ce2167a91cbabdc7b69a34c03b41afd3d020797dce8681c8b2bb3c30d434";
const K3: &str = "fc5cec1121da28ba647338cd68ad63fc245b45c6c2ff46e4c68e47594dbb5ed5";

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn hex32(s: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).expect("hex pubkey");
    }
    out
}

fn main() {
    let pubkeys = [hex32(K1), hex32(K2), hex32(K3)];
    let redeem = multisig_redeem_script(pubkeys.iter().copied(), 2).expect("2-of-3 always valid");
    let p2sh = pay_to_script_hash_script(&redeem);
    let address = extract_script_pub_key_address(&p2sh, Prefix::Mainnet).expect("valid P2SH address");

    println!("REDEEM_SCRIPT  {}", hex(&redeem));
    println!("P2SH_SCRIPT    {}", hex(p2sh.script()));
    println!("P2SH_ADDRESS   {address}");

    // The values committed in genesis.rs MUST match — fail loudly otherwise.
    assert_eq!(
        hex(&redeem),
        "52201b37ab3e285f75fcbbfbd589cb67f1eba0c73256ad2c988c6e332b302b2a3b8c\
         2067d8ce2167a91cbabdc7b69a34c03b41afd3d020797dce8681c8b2bb3c30d434\
         20fc5cec1121da28ba647338cd68ad63fc245b45c6c2ff46e4c68e47594dbb5ed553ae"
    );
    assert_eq!(hex(p2sh.script()), "aa20f76f54ec00eecf5f844935f346a1080518321ecbcfc7779c045033ffddc6dbf287");
    assert_eq!(address.to_string(), "cen69:prmk748vqrhv7huyfy6lx34ppqz3svs7e08uwauuq3gr8l7acmdlyel55kmee");

    println!("VERIFY_OK");
}