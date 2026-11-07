use crate::{
    block::Block,
    header::{CompressedParents, Header},
    subnets::SUBNETWORK_ID_COINBASE,
    tx::{ScriptPublicKey, Transaction, TransactionOutput},
};
use alloc::vec;
use alloc::vec::Vec;
use celenium_hashes::{Hash, ZERO_HASH};

/// Premine allocation written into the genesis coinbase, in base units.
/// The entire premine — 34,500,000 CEN (50% of total supply) — sits in a
/// single 2-of-3 multisig P2SH output, locked from the very first block
/// (Opsi A). The dev/owner split (22.2M / 12.3M) is off-chain bookkeeping,
/// released monthly per docs/vesting-schedule.md as a single sweep.
const PREMINE_TOTAL_AMOUNT: u64 = 34_500_000 * 100_000_000;

// Self-custody 2-of-3 multisig premine.
//
// Three fresh x-only secp256k1 keys guard the premine under a Schnorr
// CHECKMULTISIG redeem script; the genesis output pays to its P2SH, so the
// premine is never in a spendable single-sig state.
//
// Public keys (the matching PRIVATE keys live offline, encrypted, outside this
// repository — key vault file `premine-multisig-keys-SECRET.txt.gpg`):
//
//   K1 = 1b37ab3e285f75fcbbfbd589cb67f1eba0c73256ad2c988c6e332b302b2a3b8c
//   K2 = 67d8ce2167a91cbabdc7b69a34c03b41afd3d020797dce8681c8b2bb3c30d434
//   K3 = fc5cec1121da28ba647338cd68ad63fc245b45c6c2ff46e4c68e47594dbb5ed5
//
// redeem script (2-of-3) = 52 20<K1> 20<K2> 20<K3> 53 ae
// P2SH script (35 bytes)  = aa 20 <BLAKE2b256(redeem_script)> 87  (below)
// P2SH address = cen69:prmk748vqrhv7huyfy6lx34ppqz3svs7e08uwauuq3gr8l7acmdlyel55kmee
//
// Derived with celenium_txscript::{multisig_redeem_script, pay_to_script_hash_script}.
// Reproduce/verify with `cargo run -p celenium-txscript --example verify_premine_multisig`.
//
// Changing this script changes the genesis coinbase payload, the merkle root
// and therefore every genesis hash constant below. After any edit here, run
// `cargo test --package celenium-consensus-core print_genesis_regeneration_values`
// and update the hashes per scripts/generate-genesis.md.
const PREMINE_P2SH_SCRIPT: [u8; 35] = [
    0xaa, 0x20, // OP_BLAKE2b, push 32 bytes
    0xf7, 0x6f, 0x54, 0xec, 0x00, 0xee, 0xcf, 0x5f, 0x84, 0x49, 0x35, 0xf3, 0x46, 0xa1, 0x08, 0x05,
    0x18, 0x32, 0x1e, 0xcb, 0xcf, 0xc7, 0x77, 0x9c, 0x04, 0x50, 0x33, 0xff, 0xdd, 0xc6, 0xdb, 0xf2,
    0x87, // OP_EQUAL
];

fn pay_to_premine_multisig_script() -> ScriptPublicKey {
    ScriptPublicKey::from_vec(0, PREMINE_P2SH_SCRIPT.to_vec())
}

/// The constants uniquely representing the genesis block
#[derive(Clone, Debug)]
pub struct GenesisBlock {
    pub hash: Hash,
    pub version: u16,
    pub hash_merkle_root: Hash,
    pub utxo_commitment: Hash,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
    pub daa_score: u64,
    pub coinbase_payload: &'static [u8],
}

impl GenesisBlock {
    pub fn build_genesis_transactions(&self) -> Vec<Transaction> {
        vec![Transaction::new(
            0,
            Vec::new(),
            vec![
                TransactionOutput::new(PREMINE_TOTAL_AMOUNT, pay_to_premine_multisig_script()),
            ],
            0,
            SUBNETWORK_ID_COINBASE,
            0,
            self.coinbase_payload.to_vec(),
        )]
    }
}

impl From<&GenesisBlock> for Header {
    fn from(genesis: &GenesisBlock) -> Self {
        Header::new_finalized(
            genesis.version,
            CompressedParents::default(),
            genesis.hash_merkle_root,
            ZERO_HASH,
            genesis.utxo_commitment,
            genesis.timestamp,
            genesis.bits,
            genesis.nonce,
            genesis.daa_score,
            0.into(),
            0,
            ZERO_HASH,
        )
    }
}

impl From<&GenesisBlock> for Block {
    fn from(genesis: &GenesisBlock) -> Self {
        Block::new(genesis.into(), genesis.build_genesis_transactions())
    }
}

impl From<(&Header, &'static [u8])> for GenesisBlock {
    fn from((header, payload): (&Header, &'static [u8])) -> Self {
        Self {
            hash: header.hash,
            version: header.version,
            hash_merkle_root: header.hash_merkle_root,
            utxo_commitment: header.utxo_commitment,
            timestamp: header.timestamp,
            bits: header.bits,
            nonce: header.nonce,
            daa_score: header.daa_score,
            coinbase_payload: payload,
        }
    }
}

/// The genesis block of the block-DAG which serves as the public transaction ledger for the main network.
pub const GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x07, 0x0e, 0x91, 0x15, 0x0b, 0x59, 0xbe, 0x80,
        0x5d, 0xed, 0x3f, 0x64, 0xa6, 0xd9, 0x34, 0x70,
        0xde, 0xf3, 0x4e, 0x15, 0xe4, 0x07, 0x8a, 0x0f,
        0xa9, 0x2f, 0xdb, 0x31, 0x7c, 0x02, 0x6a, 0xf1,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x25, 0xb0, 0xff, 0x8f, 0x80, 0x72, 0xbb, 0xcd,
        0xc7, 0x01, 0x89, 0x8d, 0x84, 0xb1, 0x2b, 0xe3,
        0x27, 0xe6, 0x3b, 0xea, 0xf8, 0xed, 0xe5, 0x26,
        0xb7, 0xc2, 0x1e, 0x38, 0x03, 0x35, 0xd6, 0xd7,
    ]),
    utxo_commitment: Hash::from_bytes([
        0x85, 0x37, 0x2b, 0x7f, 0xc7, 0x29, 0x16, 0xe4,
        0x52, 0x9c, 0x52, 0xe1, 0x0f, 0xec, 0x06, 0xfd,
        0x11, 0x47, 0x9e, 0xea, 0x4c, 0x09, 0xfe, 0x90,
        0x26, 0xea, 0x3f, 0xb6, 0x5e, 0x79, 0x37, 0x0e,
    ]),
    timestamp: 1794012518183,
    bits: 0x1e21bc1c, // fair-launch difficulty (matches devnet; CPU/GPU-minable first blocks, DAA ramps up)
    nonce: 0x3392c,
    daa_score: 0, // fresh network: DAA clock starts at zero
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xe1, 0xf5, 0x05, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x43, 0x65, 0x6c, 0x65,
        0x6e, 0x69, 0x75, 0x6d, 0x20, 0x6d, 0x61, 0x69,
        0x6e, 0x6e, 0x65, 0x74, 0x20, 0x67, 0x65, 0x6e,
        0x65, 0x73, 0x69, 0x73,
    ],
};

pub const TESTNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x69, 0x8d, 0x1f, 0xff, 0x49, 0xa6, 0x04, 0xf0,
        0xb4, 0x2f, 0x53, 0xd4, 0xb0, 0x9a, 0x52, 0xfa,
        0xd4, 0xe7, 0x99, 0xf4, 0x8b, 0x8e, 0x0d, 0xc6,
        0xc5, 0xf7, 0xb4, 0xb7, 0xbf, 0xa5, 0x61, 0x6b,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x89, 0x9f, 0xef, 0x6b, 0xb2, 0x60, 0x22, 0xb7,
        0x75, 0x8c, 0xa6, 0x7a, 0x4c, 0x2c, 0x52, 0x59,
        0x56, 0xa4, 0xd3, 0x68, 0xd5, 0x62, 0x03, 0xf2,
        0x1a, 0x21, 0xd2, 0x3a, 0xd8, 0x5a, 0x48, 0xbe,
    ]),
    utxo_commitment: Hash::from_bytes([
        0xc6, 0xc1, 0x7e, 0x29, 0xe4, 0x8a, 0x5e, 0x41,
        0x81, 0xc5, 0x14, 0x6e, 0x1f, 0x73, 0x7f, 0x43,
        0x2f, 0x46, 0xcc, 0x15, 0x2a, 0x17, 0x9b, 0xfe,
        0x53, 0x7a, 0x4c, 0xb7, 0x1a, 0x01, 0xb1, 0x72,
    ]),
    timestamp: 1794012518183,
    bits: 0x1e7fffff,
    nonce: 0x29b2b,
    daa_score: 0,
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xe1, 0xf5, 0x05, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x63, 0x65, 0x6c, 0x65,
        0x6e, 0x69, 0x75, 0x6d, 0x2d, 0x74, 0x65, 0x73,
        0x74, 0x6e, 0x65, 0x74,
    ],
};

pub const SIMNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0x5d, 0x3b, 0x7c, 0x9b, 0xf2, 0x97, 0x27, 0x27,
        0xf6, 0x66, 0xed, 0x4c, 0xe3, 0x44, 0x87, 0x35,
        0x64, 0x0a, 0xca, 0x31, 0x04, 0xa7, 0x09, 0x59,
        0xaa, 0xd0, 0x68, 0x16, 0x22, 0xb9, 0xba, 0x9d,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x22, 0x15, 0x8d, 0x63, 0xf8, 0x67, 0x05, 0xd6,
        0xde, 0xdd, 0xf6, 0x32, 0xbc, 0x0f, 0x94, 0x43,
        0x79, 0xd9, 0xc1, 0xb2, 0xfe, 0x4e, 0xc9, 0xee,
        0xde, 0x7a, 0x26, 0x45, 0xdb, 0x71, 0x17, 0xb7,
    ]),
    utxo_commitment: Hash::from_bytes([
        0x34, 0x7b, 0x0c, 0xea, 0xdf, 0xd3, 0x50, 0xf8,
        0x94, 0xa3, 0xa7, 0x66, 0xee, 0xfb, 0x0f, 0x4d,
        0x70, 0x5d, 0xda, 0x59, 0x28, 0xb6, 0x47, 0x62,
        0x87, 0x38, 0xe0, 0x23, 0x80, 0x13, 0x86, 0x07,
    ]),
    timestamp: 1794012518183,
    bits: 0x207fffff,
    nonce: 0x1,
    daa_score: 0,
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xe1, 0xf5, 0x05, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x63, 0x65, 0x6c, 0x65,
        0x6e, 0x69, 0x75, 0x6d, 0x2d, 0x73, 0x69, 0x6d,
        0x6e, 0x65, 0x74,
    ],
};

pub const DEVNET_GENESIS: GenesisBlock = GenesisBlock {
    hash: Hash::from_bytes([
        0xc3, 0x99, 0x3c, 0x9c, 0x4b, 0x6f, 0x04, 0xdf,
        0xb2, 0x3c, 0xd0, 0xbc, 0x98, 0x6b, 0xa4, 0x99,
        0x7f, 0x3a, 0xa2, 0xee, 0x5c, 0x49, 0x11, 0x60,
        0x47, 0xe5, 0x47, 0xfe, 0xb8, 0xdc, 0x44, 0x11,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x90, 0x00, 0x58, 0xf2, 0x01, 0xb6, 0xef, 0x46,
        0x4a, 0x7f, 0x3d, 0x52, 0x57, 0xc6, 0x27, 0x68,
        0x51, 0xa7, 0xa0, 0xe6, 0x63, 0xef, 0xda, 0x69,
        0xcb, 0xe9, 0xba, 0xf3, 0x2d, 0x34, 0xba, 0xfa,
    ]),
    utxo_commitment: Hash::from_bytes([
        0x1c, 0x9d, 0x54, 0x6c, 0xb6, 0xea, 0x3a, 0xec,
        0x70, 0xd1, 0x34, 0x34, 0xa8, 0x2f, 0xea, 0x6c,
        0x3f, 0x28, 0x7e, 0xf3, 0xdd, 0xe3, 0xc4, 0x52,
        0xe5, 0x73, 0xb3, 0x44, 0xe3, 0xe4, 0xf5, 0x76,
    ]),
    timestamp: 1794012518183,
    // bits: 525264379, // Golang devnet genesis bits
    bits: 0x1e21bc1c, // Bits with ~testnet-like difficulty for slow devnet start
    nonce: 0x83e98,
    daa_score: 0,
    #[rustfmt::skip]
    coinbase_payload: &[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xe1, 0xf5, 0x05, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x63, 0x65, 0x6c, 0x65,
        0x6e, 0x69, 0x75, 0x6d, 0x2d, 0x64, 0x65, 0x76,
        0x6e, 0x65, 0x74,
    ],
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merkle::calc_hash_merkle_root;
    use crate::muhash::MuHashExtensions;
    use crate::tx::TransactionOutpoint;
    use crate::utxo::UtxoEntry;
    use celenium_muhash::MuHash;

    #[test]
    fn test_genesis_hashes() {
        [GENESIS, TESTNET_GENESIS, SIMNET_GENESIS, DEVNET_GENESIS].into_iter().for_each(|genesis| {
            let block: Block = (&genesis).into();
            assert_hashes_eq(calc_hash_merkle_root(block.transactions.iter()), block.header.hash_merkle_root, "hash_merkle_root");
            assert_hashes_eq(block.hash(), genesis.hash, "genesis hash");
        });
    }

    /// Prints the values needed to regenerate the genesis constants after the
    /// premine outputs change: merkle root, utxo commitment over the premine
    /// UTXOs, and the resulting block hash.
    #[test]
    fn print_genesis_regeneration_values() {
        for (name, genesis) in
            [("GENESIS", &GENESIS), ("TESTNET_GENESIS", &TESTNET_GENESIS), ("SIMNET_GENESIS", &SIMNET_GENESIS), ("DEVNET_GENESIS", &DEVNET_GENESIS)]
        {
            let block: Block = genesis.into();
            let merkle_root = calc_hash_merkle_root(block.transactions.iter());

            let tx = &block.transactions[0];
            let txid = tx.id();
            let mut muhash = MuHash::new();
            for (index, out) in tx.outputs.iter().enumerate() {
                let outpoint = TransactionOutpoint { transaction_id: txid, index: index as u32 };
                let entry = UtxoEntry {
                    amount: out.value,
                    script_public_key: out.script_public_key.clone(),
                    block_daa_score: 0,
                    is_coinbase: false,
                    covenant_id: None,
                };
                muhash.add_utxo(&outpoint, &entry);
            }
            let commitment = muhash.finalize();

            let header = Header::new_finalized(
                genesis.version,
                CompressedParents::default(),
                merkle_root,
                ZERO_HASH,
                commitment,
                genesis.timestamp,
                genesis.bits,
                genesis.nonce,
                genesis.daa_score,
                0.into(),
                0,
                ZERO_HASH,
            );

            println!("=== {} ===", name);
            println!("hash_merkle_root: {:#04x?}", merkle_root.as_bytes());
            println!("utxo_commitment:  {:#04x?}", commitment.as_bytes());
            println!("hash:             {:#04x?}", header.hash.as_bytes());
        }
    }

    fn assert_hashes_eq(got: Hash, expected: Hash, field: &str) {
        if got != expected {
            // Special hex print to ease changing the genesis hash according to the print if needed
            panic!("Invalid field '{}': Got hash {:#04x?} while expecting {:#04x?}", field, got.as_bytes(), expected.as_bytes());
        }
    }
}
