use celenium_stratum_bridge::{emission_policy as ep, faucet::FAUCET_MAX_REWARD_PRADHA};

fn main() {
    let cen = |p: u64| p as f64 / 1e8;
    let max_r_cen = FAUCET_MAX_REWARD_PRADHA as f64 / 1e8;
    println!(
        "{:>5} {:>7} {:>16} {:>7} {:>15} {:>14} {:>13}",
        "bulan", "tahun", "sisa emisi CEN", "cut%", "floor CEN", "cap/hari", "klaim@max"
    );
    let months = [0u64, 1, 6, 12, 24, 36, 48, 60, 120, 240, 360, 480, 600, 720, 816];
    for &m in &months {
        let r = ep::remaining_emission_pradha(m);
        // Demand cut jika reservoir kosong (batas atas laju pengisian).
        let cut = ep::cut_fraction(m, 0);
        let cap = ep::daily_cap_pradha(m);
        println!(
            "{:>5} {:>7.1} {:>16.0} {:>7.2} {:>15.0} {:>14.2} {:>13.0}",
            m,
            m as f64 / 12.0,
            cen(r),
            cut * 100.0,
            cen(ep::floor_pradha(m)),
            cen(cap),
            cen(cap) / max_r_cen,
        );
    }
    println!("\ngenesis unix = {}", ep::GENESIS_TIMESTAMP_SECS);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Snapshot dengan saldo reservoir 0 (kondisi awal/pengisian).
    let s = ep::snapshot(now, 0);
    println!(
        "SEKARANG: bulan={} cut={:.2}% floor={:.0} CEN cap={:.2} CEN/hari ({} klaim @ plafon {:.8} CEN)",
        s.month,
        s.cut_fraction * 100.0,
        cen(s.floor_pradha),
        cen(s.daily_cap_pradha),
        (cen(s.daily_cap_pradha) / max_r_cen) as u64,
        max_r_cen,
    );
}
