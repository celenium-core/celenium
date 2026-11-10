//! Generates the CLTV-locked premine outputs for the CEN genesis block.
//!
//! Builds 264 genesis outputs (2 vaults x 132 monthly tranches). Each output is
//! a P2SH paying to
//!   `<5th-of-month, ms> OP_CHECKLOCKTIMEVERIFY OP_2 <K> OP_3 OP_CHECKMULTISIG`
//! i.e. on-chain time-locked 2-of-3 multisig — a tranche cannot be spent before
//! the 5th of its month (enforced by the CLTV opcode), and always requires 2 of
//! 3 signatures after that.
//!
//!   Dev vault   : K1/K2/K3, 9,600,000 CEN, 132 tranches, first unlock 5 Dec 2026
//!   Owner vault : K4/K5/K6, 6,900,000 CEN, 132 tranches, first unlock 5 Dec 2026
//!
//! Prints a public-only summary to stdout. Given two output paths it also writes
//! the generated Rust `const` block (for splicing into `genesis.rs`) and a
//! markdown schedule (for docs / website). No private keys are involved.

use std::fmt::Write as _;

use chrono::{TimeZone, Utc};
use celenium_addresses::Prefix;
use celenium_txscript::{
    extract_script_pub_key_address, multisig_redeem_script, opcodes::codes::OpCheckLockTimeVerify,
    pay_to_script_hash_script, script_builder::ScriptBuilder,
};

const K1: &str = "1b37ab3e285f75fcbbfbd589cb67f1eba0c73256ad2c988c6e332b302b2a3b8c";
const K2: &str = "67d8ce2167a91cbabdc7b69a34c03b41afd3d020797dce8681c8b2bb3c30d434";
const K3: &str = "fc5cec1121da28ba647338cd68ad63fc245b45c6c2ff46e4c68e47594dbb5ed5";
const K4: &str = "289c5f5a473b0547ecd89ff5be7d2dedb66a1655077bd34e00b8a2afe847ecdf";
const K5: &str = "d92d526cd3827d2478ad95d688af5157e6ac92064a22619b34f8dbaa2bc2e2ad";
const K6: &str = "111b443c9b2510dbe3d3f59a135fbcab376ec1f60f2f8bea97989f3ba4cda21e";

const TRANCHES: usize = 132;
const DEV_TOTAL: u64 = 9_600_000 * 100_000_000; // 9.6M CEN in pradha
const OWNER_TOTAL: u64 = 6_900_000 * 100_000_000; // 6.9M CEN in pradha
const P2SH_LEN: usize = 35;

fn hex32(s: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).expect("hex pubkey");
    }
    out
}

fn cen_str(pradha: u64) -> String {
    format!("{}.{:08}", pradha / 100_000_000, pradha % 100_000_000)
}

fn redeem_script(keys: &[[u8; 32]; 3], lock_time_ms: u64) -> Vec<u8> {
    let mut builder = ScriptBuilder::new();
    builder.add_lock_time(lock_time_ms).expect("lock time fits");
    builder.add_op(OpCheckLockTimeVerify).expect("opcode fits");
    let multisig = multisig_redeem_script(keys.iter().copied(), 2).expect("2-of-3");
    builder.add_ops(&multisig).expect("append multisig");
    builder.drain()
}

struct Output {
    amount: u64,
    p2sh: [u8; P2SH_LEN],
    address: String,
    ts_ms: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dev_keys = [hex32(K1), hex32(K2), hex32(K3)];
    let owner_keys = [hex32(K4), hex32(K5), hex32(K6)];

    // 132 monthly timestamps: the 5th of each month, Dec 2026 .. Nov 2037, 00:00 UTC.
    let mut timestamps = Vec::with_capacity(TRANCHES);
    {
        let mut year = 2026i32;
        let mut month = 12u32;
        for _ in 0..TRANCHES {
            let dt = Utc.with_ymd_and_hms(year, month, 5, 0, 0, 0).single().expect("valid date");
            timestamps.push((dt.timestamp_millis()) as u64);
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }
    }

    let dev_base = DEV_TOTAL / TRANCHES as u64;
    let dev_rem = DEV_TOTAL % TRANCHES as u64;
    let owner_base = OWNER_TOTAL / TRANCHES as u64;
    let owner_rem = OWNER_TOTAL % TRANCHES as u64;

    let mut outputs: Vec<Output> = Vec::with_capacity(TRANCHES * 2);
    for vault in 0..2 {
        let (keys, base, rem) = match vault {
            0 => (&dev_keys, dev_base, dev_rem),
            _ => (&owner_keys, owner_base, owner_rem),
        };
        for i in 0..TRANCHES {
            let amount = base + if i == TRANCHES - 1 { rem } else { 0 };
            let ts = timestamps[i];
            let redeem = redeem_script(keys, ts);
            let spk = pay_to_script_hash_script(&redeem);
            let script = spk.script();
            assert_eq!(script.len(), P2SH_LEN, "P2SH must be 35 bytes");
            let mut p2sh = [0u8; P2SH_LEN];
            p2sh.copy_from_slice(script);
            let address = extract_script_pub_key_address(&spk, Prefix::Mainnet)?.to_string();
            outputs.push(Output { amount, p2sh, address, ts_ms: ts });
        }
    }

    let dev_sum: u64 = outputs[..TRANCHES].iter().map(|o| o.amount).sum();
    let owner_sum: u64 = outputs[TRANCHES..].iter().map(|o| o.amount).sum();
    assert_eq!(dev_sum, DEV_TOTAL, "dev total mismatch");
    assert_eq!(owner_sum, OWNER_TOTAL, "owner total mismatch");
    assert_eq!(outputs.len(), 264, "expected 264 outputs");

    // ---- stdout summary (public data) ----
    println!("outputs            = {}", outputs.len());
    println!("dev_total_pradha   = {DEV_TOTAL}  ({} CEN)", cen_str(DEV_TOTAL));
    println!("owner_total_pradha = {OWNER_TOTAL}  ({} CEN)", cen_str(OWNER_TOTAL));
    println!("premine_total_pradha = {}  ({} CEN)", DEV_TOTAL + OWNER_TOTAL, cen_str(DEV_TOTAL + OWNER_TOTAL));
    println!("dev_cen_per_month  = {}  (base {} CEN + {} pradha on last)", cen_str(dev_base), cen_str(dev_base), dev_rem);
    println!("owner_cen_per_month= {}  (base {} CEN + {} pradha on last)", cen_str(owner_base), cen_str(owner_base), owner_rem);
    println!("first_unlock_ms    = {}", timestamps[0]);
    println!("last_unlock_ms     = {}", timestamps[TRANCHES - 1]);
    println!("dev_address[0]     = {}", outputs[0].address);
    println!("dev_address[131]   = {}", outputs[TRANCHES - 1].address);
    println!("owner_address[0]   = {}", outputs[TRANCHES].address);
    println!("owner_address[131] = {}", outputs[2 * TRANCHES - 1].address);

    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 {
        write_const(&args[1], &outputs)?;
        write_schedule(&args[2], &outputs, &timestamps)?;
    }

    println!("DONE");
    Ok(())
}

fn write_const(path: &str, outputs: &[Output]) -> Result<(), Box<dyn std::error::Error>> {
    let mut s = String::with_capacity(outputs.len() * 60);
    s.push_str("/// CLTV-locked premine: 2 vaults x 132 monthly tranches = 264 outputs.\n");
    s.push_str("/// dev vault (K1/K2/K3, 9.6M CEN) then owner vault (K4/K5/K6, 6.9M CEN).\n");
    s.push_str("/// Each output is a P2SH of <locktime> OP_CHECKLOCKTIMEVERIFY + 2-of-3\n");
    s.push_str("/// CHECKMULTISIG; generated by `gen_premine_cltv`. Do not hand-edit.\n");
    s.push_str("#[rustfmt::skip]\n");
    s.push_str(&format!("const PREMINE_OUTPUTS: [(u64, [u8; {P2SH_LEN}]); 264] = [\n"));
    for o in outputs {
        let _ = write!(s, "    ({}u64, [", o.amount);
        for (j, b) in o.p2sh.iter().enumerate() {
            let _ = write!(s, "0x{b:02x}");
            if j + 1 < P2SH_LEN {
                s.push_str(", ");
            }
        }
        s.push_str("]),\n");
    }
    s.push_str("];\n");
    std::fs::write(path, s)?;
    eprintln!("wrote const -> {path}");
    Ok(())
}

fn write_schedule(
    path: &str,
    outputs: &[Output],
    timestamps: &[u64],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut s = String::with_capacity(64 * 1024);
    s.push_str("# CEN Premine — CLTV schedule\n\n");
    s.push_str("Auto-unlock on-chain every 5th of the month (UTC), 132 tranches per vault.\n");
    s.push_str("Each tranche is a P2SH of `OP_CHECKLOCKTIMEVERIFY(<ts ms>) + 2-of-3 multisig`.\n\n");
    s.push_str("- **Dev vault** — 9,600,000 CEN, 2-of-3 (K1/K2/K3)\n");
    s.push_str("- **Owner vault** — 6,900,000 CEN, 2-of-3 (K4/K5/K6)\n");
    s.push_str("- Total premine — 16,500,000 CEN (23.913% of 69,000,000)\n\n");

    let write_vault = |s: &mut String, label: &str, slice: &[Output]| {
        let _ = write!(s, "## {label}\n\n| # | Unlock (UTC) | Timestamp (ms) | Amount (pradha) | Amount (CEN) | Address |\n");
        s.push_str("|---|--------------|----------------|-----------------|--------------|---------|\n");
        for (i, o) in slice.iter().enumerate() {
            let ms = o.ts_ms;
            // index into the shared timestamps array is the tranche index
            let _ = timestamps; // kept for clarity of ordering
            let _ = write!(
                s,
                "| {} | {} | {} | {} | {} | {} |\n",
                i + 1,
                fmt_ms_date(ms),
                ms,
                o.amount,
                cen_str(o.amount),
                o.address
            );
        }
    };
    write_vault(&mut s, "Dev vault (K1/K2/K3) — 9,600,000 CEN", &outputs[..TRANCHES]);
    write_vault(&mut s, "Owner vault (K4/K5/K6) — 6,900,000 CEN", &outputs[TRANCHES..]);

    std::fs::write(path, s)?;
    eprintln!("wrote schedule -> {path}");
    Ok(())
}

fn fmt_ms_date(ms: u64) -> String {
    Utc.timestamp_millis_opt(ms as i64).single().map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default()
}