use crate::{
    block::Block,
    header::{CompressedParents, Header},
    subnets::SUBNETWORK_ID_COINBASE,
    tx::{ScriptPublicKey, Transaction, TransactionOutput},
};
use alloc::vec;
use alloc::vec::Vec;
use celenium_hashes::{Hash, ZERO_HASH};

/// Premine allocations written into the genesis coinbase, in base units.
/// Development 22,200,000 CEN, owner 12,300,000 CEN, total 34,500,000 CEN (50% of supply).
const PREMINE_DEV_AMOUNT: u64 = 22_200_000 * 100_000_000;
const PREMINE_OWNER_AMOUNT: u64 = 12_300_000 * 100_000_000;

// Public keys for the premine outputs (x-only secp256k1, 32 bytes).
// Generated 2026-10-29 with consensus/client/examples/genkey.rs.
// The matching PRIVATE keys are kept offline in the owner's key vault,
// outside this repository, and must never be committed.
//
// dev:   22,200,000 CEN (32.2% of total supply)
// owner: 12,300,000 CEN (17.8% of total supply)
//
// Changing these values changes the genesis coinbase payload, the merkle root
// and therefore every genesis hash constant below. After any edit here, run
// `cargo test --package celenium-consensus-core print_genesis_regeneration_values`
// and update the hashes per scripts/generate-genesis.md.
const PREMINE_DEV_PUBKEY: [u8; 32] = [
    0x63, 0xa2, 0xcd, 0x0e, 0x5f, 0xc8, 0x32, 0x0e, 0x93, 0x99, 0xfd, 0x25, 0x08, 0xa9, 0xf3, 0xda, 0x45, 0xdb, 0x94, 0x01, 0x9f, 0x61,
    0xa8, 0x90, 0xd3, 0xea, 0x42, 0xef, 0x32, 0x3c, 0xf9, 0xe6,
];
const PREMINE_OWNER_PUBKEY: [u8; 32] = [
    0xb3, 0x18, 0x7e, 0xbc, 0x6e, 0x2e, 0xc5, 0x12, 0xd7, 0x73, 0x32, 0xaf, 0x55, 0x77, 0x7e, 0xfa, 0x91, 0xa3, 0xaf, 0xc6, 0x7b, 0x5f,
    0xcb, 0x65, 0x60, 0xb5, 0xbd, 0x91, 0xe1, 0xe1, 0xaf, 0x54,
];

fn pay_to_pubkey_script(pubkey: &[u8; 32]) -> ScriptPublicKey {
    let mut script = Vec::with_capacity(34);
    script.push(0x20); // push 32 bytes
    script.extend_from_slice(pubkey);
    script.push(0xac); // OP_CheckSig
    ScriptPublicKey::from_vec(0, script)
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
                TransactionOutput::new(PREMINE_DEV_AMOUNT, pay_to_pubkey_script(&PREMINE_DEV_PUBKEY)),
                TransactionOutput::new(PREMINE_OWNER_AMOUNT, pay_to_pubkey_script(&PREMINE_OWNER_PUBKEY)),
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
        0x55, 0xf7, 0x73, 0x58, 0x65, 0xee, 0x1c, 0xd2,
        0xdd, 0xc5, 0x14, 0x0c, 0x17, 0x4f, 0xe7, 0x2b,
        0x05, 0x67, 0xc1, 0xc0, 0xfb, 0x11, 0x90, 0x7c,
        0xca, 0x5a, 0x19, 0x60, 0x98, 0xa1, 0xe3, 0xde,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0xae, 0xe7, 0x13, 0x11, 0xfd, 0x5f, 0xf8, 0x64,
        0x3e, 0x52, 0xbb, 0x5e, 0xc9, 0x34, 0x88, 0x10,
        0xc4, 0x16, 0x09, 0x19, 0x83, 0x4e, 0x1b, 0x21,
        0x48, 0x0f, 0xc3, 0x6f, 0xfc, 0xb1, 0xf9, 0x99,
    ]),
    utxo_commitment: Hash::from_bytes([
        0x56, 0x4e, 0xa5, 0x57, 0xf0, 0x5b, 0x59, 0x7c,
        0x42, 0x55, 0x15, 0x7d, 0xa0, 0xfd, 0xb7, 0xbf,
        0x7b, 0x2f, 0x97, 0xf5, 0x5b, 0xcd, 0x28, 0xf9,
        0x30, 0xfe, 0x67, 0xf2, 0x76, 0xef, 0xc7, 0x3d,
    ]),
    timestamp: 1793229666249,
    bits: 486722099,
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
        0xf3, 0xd9, 0x9b, 0x3c, 0xf2, 0x27, 0x20, 0xcc,
        0x39, 0xd7, 0xe9, 0x6a, 0x6c, 0xd2, 0xb0, 0x99,
        0x6d, 0x4d, 0x5a, 0x1f, 0x23, 0xe3, 0x6b, 0x57,
        0x92, 0x65, 0x17, 0xa0, 0x42, 0x05, 0xdb, 0x89,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0xc3, 0x0a, 0xb0, 0x3d, 0x7d, 0x51, 0x88, 0xa8,
        0x6b, 0x78, 0x22, 0x02, 0x07, 0xec, 0x2a, 0xd4,
        0x3e, 0x6f, 0xec, 0x1c, 0xd5, 0x3a, 0xaf, 0x2c,
        0xc2, 0xe7, 0x4d, 0x87, 0x3e, 0x8c, 0xe2, 0x6e,
    ]),
    utxo_commitment: Hash::from_bytes([
        0xda, 0xec, 0xb8, 0x6b, 0xac, 0xbd, 0x95, 0x41,
        0x70, 0x8e, 0x13, 0x4b, 0xdd, 0xe5, 0x96, 0x35,
        0xa0, 0x1d, 0xb8, 0xe3, 0xf7, 0x1c, 0x47, 0x29,
        0x45, 0x21, 0x62, 0x11, 0xdc, 0xb4, 0xce, 0xe3,
    ]),
    timestamp: 1793229666249,
    bits: 0x1e7fffff,
    nonce: 0x9337,
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
        0xf3, 0xf0, 0xe6, 0x03, 0x7d, 0xd1, 0x6a, 0x1c,
        0x5a, 0xe4, 0x41, 0x50, 0xef, 0xac, 0x2e, 0xef,
        0x4a, 0xcf, 0x1f, 0xf3, 0x78, 0x8d, 0x19, 0x4c,
        0xb5, 0xc6, 0xf1, 0xb8, 0xd9, 0x4c, 0xd0, 0xaa,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x89, 0xfc, 0x05, 0xa1, 0x5c, 0x66, 0xdd, 0xb9,
        0xdf, 0x3c, 0xa9, 0xe8, 0x85, 0xf0, 0x74, 0xc2,
        0x5a, 0x03, 0x8c, 0x99, 0x77, 0xd3, 0xb3, 0x6e,
        0xdf, 0xc7, 0x9a, 0xf8, 0x66, 0x28, 0x75, 0x87,
    ]),
    utxo_commitment: Hash::from_bytes([
        0x68, 0x72, 0xa3, 0x0f, 0x99, 0x93, 0x11, 0x41,
        0x76, 0xc9, 0x7d, 0xd2, 0xcf, 0xb0, 0x4d, 0xaf,
        0xbe, 0x95, 0x90, 0x76, 0x30, 0x6c, 0x0a, 0x00,
        0x77, 0xd1, 0xe8, 0x9f, 0x89, 0x41, 0x90, 0xc0,
    ]),
    timestamp: 1793229666249,
    bits: 0x207fffff,
    nonce: 0x0,
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
        0x09, 0x42, 0x84, 0x56, 0xea, 0x1b, 0x6c, 0x35,
        0xb2, 0x4a, 0x76, 0xe8, 0x5e, 0x5d, 0x41, 0xe3,
        0xed, 0x7e, 0xc3, 0xb4, 0xf5, 0xa4, 0x28, 0x6a,
        0x24, 0xb6, 0xf7, 0x3f, 0x91, 0x8c, 0x6a, 0xfb,
    ]),
    version: 0,
    hash_merkle_root: Hash::from_bytes([
        0x5f, 0x1d, 0xcc, 0x8d, 0xdd, 0xcd, 0x76, 0xf6,
        0x1f, 0x62, 0x9c, 0xa0, 0x9a, 0x3e, 0x57, 0xa2,
        0xdd, 0x73, 0xcf, 0xf6, 0xe8, 0x26, 0x3e, 0xd9,
        0xd4, 0xbe, 0x16, 0xbf, 0x43, 0x70, 0x66, 0xf9,
    ]),
    utxo_commitment: Hash::from_bytes([
        0x85, 0xfe, 0x02, 0xe0, 0x77, 0x21, 0x17, 0xb7,
        0xcf, 0xf7, 0x5d, 0xcf, 0x67, 0x0b, 0x43, 0x72,
        0xdb, 0x74, 0x54, 0xec, 0x88, 0x45, 0xe7, 0xe5,
        0x26, 0x90, 0x22, 0x2a, 0x36, 0x58, 0xd9, 0xcd,
    ]),
    timestamp: 1793229666249,
    // bits: 525264379, // Golang devnet genesis bits
    bits: 0x1e21bc1c, // Bits with ~testnet-like difficulty for slow devnet start
    nonce: 0x41ff8,
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
