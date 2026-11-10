use celenium_hashes::{Hash, KHeavyHash, PowHash};

// Vektor nyata dari notify bridge (mainnet testnet Celenium) + nonce yang dikirim miner.
// head (LE u64) = [5460885386052743478, 4149492402066251054, 9987509850103645117, 16569751555461548343]
// timestamp = 1794276294877
#[test]
fn pow_known_vector() {
    let words: [u64; 4] = [
        5460885386052743478,
        4149492402066251054,
        9987509850103645117,
        16569751555461548343,
    ];
    let pp = Hash::from_le_u64(words);
    let ts: u64 = 1794276294877;

    for nonce in [0xc00000ce1dbf4u64, 0xc00000ce3f489u64] {
        let h = PowHash::new(pp, ts).finalize_with_nonce(nonce);
        let hh = KHeavyHash::hash(h).as_bytes();
        // Tampilkan sebagai big-endian MSB-first (sama seperti format log bridge).
        let mut msb_first: Vec<u8> = hh.to_vec();
        msb_first.reverse();
        print!("nonce {:016x} pow_msb=", nonce);
        for b in &msb_first {
            print!("{:02x}", b);
        }
        println!();
    }
}