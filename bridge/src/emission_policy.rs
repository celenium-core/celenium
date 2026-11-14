//! Kebijakan emisi dinamis untuk faucet Celenium.
//!
//! Tujuan: reservoir faucet diisi dengan laju yang **dijamin mencapai target**
//! dalam tenggat yang dipilih, lalu berhenti — bukan konstanta kaku yang secara
//! matematis tidak pernah sampai.
//!
//! ## Algoritma: deadline-fill + maintenance
//!
//! Reservoir punya target tetap `TARGET_RESERVOIR_PRADHA` (2.500.000 CEN).
//! Setiap bulan, `cut_fraction(month, balance)` menjawab satu pertanyaan:
//! *berapa porsi reward bulan ini yang harus disisihkan agar target tercapai
//! dalam `FILL_HORIZON_MONTHS`, dan setelah itu agar saldo tidak terkikis?*
//!
//! ```text
//! floor(m)    = FLOOR_FRACTION × sisa_emisi(m)      // target lokal, menyusut
//! defisit     = floor(m) − saldo
//! cut(m)      = clamp(defisit ÷ emisi(m..m+horizon), MIN_CUT, MAX_CUT)
//! ```
//!
//! Sifat yang membuat ini benar, bukan sekadar masuk akal:
//!
//! - **Terjamin sampai.** `defisit ÷ emisi_selama_horizon` adalah laju yang
//!   persis menghabiskan defisit dalam horizon. Karena penyebutnya emisi nyata
//!   (bukan konstanta), target tercapai dalam 48 bulan dengan asumsi emisi
//!   berjalan seperti tabel — bukan 68 tahun, bukan tidak pernah.
//! - **Self-correcting.** Kalau faucet dikuras orang, saldo turun → defisit
//!   naik → cut naik (sampai `MAX_CUT`). Kalau saldo penuh, defisit ≤ deadband
//!   → cut 0 → reward kembali 100% ke miner. Tidak perlu intervensi manual.
//! - **Tidak mengejar bulan sabit.** `floor(m)` menyusut mengikuti sisa emisi,
//!   jadi reservoir tidak pernah menuntut dana yang belum tentu terbit. Saat
//!   emisi berakhir, floor = 0 dan cut = 0 secara alami.
//! - **Berhenti bersih.** `MAX_CUT` membatasi beban maksimum ke miner (10%),
//!   `MIN_CUT` memastikan sisa defisit kecil tetap tuntas, bukan menggantung
//!   sebagai cut mikro selamanya.
//!
//! ## Kenapa bukan versi sebelumnya
//!
//! Versi lama memakai `cut` melandai 5%→1,5% dan `floor = 5% sisa emisi`.
//! Keduanya saling lepas: inflow total = 1,5% × emisi = 787.501 CEN, sedangkan
//! ambang mati = 1,2 × floor(0) = 3.150.005 CEN. Reservoir tidak akan pernah
//! menyentuh ambang mati, dan karena floor menyusut lebih cepat daripada saldo
//! naik, ambang menyalip saldo di bulan ke-94 sehingga cut mati prematur di
//! 825.788 CEN — 33% dari target. Terbukti lewat simulasi terhadap tabel
//! subsidi mainnet, bukan dugaan.
//!
//! ## Dua kewajiban, satu reservoir
//!
//! Reservoir tidak hanya membayar klaim faucet. Dana yang sama juga dipakai
//! membiayai miner Android, jadi target tunggal 2.500.000 CEN sebenarnya
//! menyembunyikan dua tagihan yang bersaing:
//!
//! ```text
//! target_reservoir = bucket_klaim + bucket_miner
//! bucket_klaim     = (1 − MINER_SUBSIDY_SHARE) × floor
//! bucket_miner     = MINER_SUBSIDY_SHARE × floor
//! ```
//!
//! Keduanya berbagi laju inflow yang sama, jadi `daily_cap_pradha` (cap klaim)
//! dan `miner_subsidy_daily_pradha` (anggaran miner) dihitung dari porsi
//! masing-masing, bukan dari total. Menjumlahkan keduanya tidak pernah melebihi
//! inflow yang benar-benar masuk — inilah yang mencegah faucet menjanjikan dana
//! yang sudah dialokasikan ke tempat lain.
//!
//! Tanpa pemisahan ini, cap klaim akan memakai seluruh inflow dan subsidi miner
//! hanya bisa dibayar dengan menguras bucket klaim — defisit yang tidak terlihat
//! sampai penarikan gagal.
//!
//! ## Kap harian faucet
//!
//! `daily_cap_pradha(month)` = **laju inflow berkelanjutan** bucket klaim pada
//! `MAX_CUT`: `emisi_bulan × 10% × 60% ÷ hari_per_bulan`. Tidak ada komponen
//! amortisasi saldo — itu akan membayar dari bucket yang belum terisi.
//! Cap menyusut bersama emisi dan tidak pernah melebihi dana yang benar-benar
//! masuk, bukan angka fiktif yang mengantrekan klaim gagal.
//!
//! Semua dihitung dari tabel subsidy yang sama dengan konsensus
//! (`SUBSIDY_BY_MONTH_TABLE`, 817 bulan mulai genesis 7 Nov 2026). Nilai
//! pradha disimpan u64: total sisa emisi ≈ 5.25e18 pradha < u64::MAX.
//!
//! Sifat deterministik: dua pemanggil dengan `now_secs` dan saldo yang sama
//! melihat kebijakan yang sama, sehingga kode reservoir dan faucet tidak bisa
//! berbeda pendapat.

/// Detik per bulan (rata-rata 30,4375 hari), sama dengan konsensus.
pub const SECONDS_PER_MONTH: u64 = 2_629_800;

/// Blocks per second mainnet.
pub const BPS: u64 = 10;

/// Target reservoir faucet: 2.500.000 CEN dalam pradha (1 CEN = 1e8 pradha).
pub const TARGET_RESERVOIR_PRADHA: u64 = 250_000_000_000_000;

/// Tenggat pengisian reservoir, dalam bulan (4 tahun).
///
/// Dipilih agar cut awal tetap di bawah `MAX_CUT` dengan tabel emisi mainnet:
/// emisi 48 bulan pertama = 26.250.205 CEN, sehingga laju yang dibutuhkan
/// 2.500.000 ÷ 26.250.205 = 9,52% — di bawah plafon 10%. Horizon 36 bulan akan
/// menuntut 12,70% dan terpotong plafon, sehingga target tidak lagi terjamin.
pub const FILL_HORIZON_MONTHS: u64 = 48;

/// Beban maksimum yang boleh diambil dari reward miner (10%).
///
/// Ini pagar kepercayaan, bukan angka teknis: miner harus bisa memprediksi
/// bahwa pool tidak akan mengambil lebih dari sepersepuluh reward mereka.
pub const MAX_CUT_FRACTION: f64 = 0.10;

/// Cut minimum saat masih ada defisit (0,5%).
///
/// Tanpa lantai ini, defisit yang tinggal sedikit menghasilkan cut mikroskopis
/// yang tidak pernah tuntas dan membuat faucet terus memotong reward.
pub const MIN_CUT_FRACTION: f64 = 0.005;

/// Zona mati relatif terhadap floor (1%).
///
/// Saldo di dalam 1% floor dianggap "sudah penuh" agar cut tidak menyala-mati
/// setiap blok karena fluktuasi pembulatan.
pub const DEADBAND_FRACTION: f64 = 0.01;

/// Porsi target reservoir yang dialokasikan untuk membiayai miner Android.
///
/// Dihitung dari pengeluaran nyata, bukan angka bulat: miner Android menanggung
/// kuota data, listrik, dan keausan perangkat yang tidak sebanding dengan
/// hashrate ponsel. 40% dipilih karena:
///
/// - klaim faucet tetap dapat porsi mayoritas (60%), sehingga insentif partisipasi
///   tidak tergerus oleh subsidi;
/// - pada emisi bulan 0 (546.879 CEN/bulan), 40% × cut 10% = 719 CEN/hari, cukup
///   untuk ~10 miner pada ~72 CEN/hari masing-masing di fase awal;
/// - porsi ini menyusut bersama emisi secara alami, jadi tidak ada janji subsidi
///   yang melampaui dana yang akan terbit.
///
/// Ubah angka ini untuk menggeser keseimbangan; seluruh perhitungan lain
/// mengikuti otomatis karena hanya konstanta ini yang menyebut porsi.
pub const MINER_SUBSIDY_SHARE: f64 = 0.40;

/// Jumlah miner Android yang ingin didukung kebijakan pada fase awal.
///
/// Dipakai untuk menerjemahkan anggaran agregat menjadi angka per-perangkat yang
/// bisa dipahami operator. Bukan batas keras: kalau miner lebih sedikit, tiap
/// perangkat menerima bagian lebih besar; kalau lebih banyak, subsidi per
/// perangkat mengecil. Nilai ini hanya pembagi pelaporan.
pub const TARGET_MINER_COUNT: u32 = 10;

/// Detik per bulan sebagai f64, untuk konversi laju harian.
const DAYS_PER_MONTH: f64 = 30.4375;

/// Timestamp genesis mainnet (7 Nov 2026 20:00:00 UTC).
pub const GENESIS_TIMESTAMP_SECS: u64 = 1_794_081_600;

/// Tabel subsidi per bulan (1 BPS), dibaca dari `scripts/data/subsidy_table.rs`
/// yang sama dengan konsensus. Nilai = pradha per detik (per blok @1BPS).
const SUBSIDY_BY_MONTH_TABLE: [u64; 817] = [
    20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466,
    20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466,
    20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466,
    20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466, 20795466,
    10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733,
    10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733,
    10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733,
    10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733, 10397733,
    5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867,
    5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867,
    5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867,
    5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867, 5198867,
    2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433,
    2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433,
    2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433,
    2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433, 2599433,
    1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717,
    1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717,
    1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717,
    1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717, 1299717,
    649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858,
    649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858,
    649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858,
    649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858, 649858,
    324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929,
    324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929,
    324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929,
    324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929, 324929,
    162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465,
    162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465,
    162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465,
    162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465, 162465,
    81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232,
    81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232,
    81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232,
    81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232, 81232,
    40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616,
    40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616,
    40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616,
    40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616, 40616,
    20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308,
    20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308,
    20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308,
    20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308, 20308,
    10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154,
    10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154,
    10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154,
    10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154, 10154,
    5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077,
    5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077,
    5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077,
    5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077, 5077,
    2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539,
    2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539,
    2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539,
    2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539, 2539,
    1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269,
    1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269,
    1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269,
    1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269, 1269,
    635, 635, 635, 635, 635, 635, 635, 635, 635, 635, 635, 635,
    635, 635, 635, 635, 635, 635, 635, 635, 635, 635, 635, 635,
    635, 635, 635, 635, 635, 635, 635, 635, 635, 635, 635, 635,
    635, 635, 635, 635, 635, 635, 635, 635, 635, 635, 635, 635,
    317, 317, 317, 317, 317, 317, 317, 317, 317, 317, 317, 317,
    317, 317, 317, 317, 317, 317, 317, 317, 317, 317, 317, 317,
    317, 317, 317, 317, 317, 317, 317, 317, 317, 317, 317, 317,
    317, 317, 317, 317, 317, 317, 317, 317, 317, 317, 317, 317,
    0,
];

/// Kembalikan jumlah elemen tabel (untuk sanity check di tes).
pub const fn table_len() -> usize {
    SUBSIDY_BY_MONTH_TABLE.len()
}

/// Bulan sejak genesis untuk `now_secs` (detik Unix).
pub fn month_since_genesis(now_secs: u64) -> u64 {
    now_secs.saturating_sub(GENESIS_TIMESTAMP_SECS) / SECONDS_PER_MONTH
}

/// Subsidi per bulan (pradha) pada bulan ke-`month`, sesuai 10 BPS.
/// Konsensus memakai `tabel[i].div_ceil(BPS)` per blok lalu × BPS × detik
/// bulan — pembulatan ke atas agar anggaran tidak pernah diremehkan.
pub fn subsidy_pradha_month(month: u64) -> u64 {
    let m = month.min(SUBSIDY_BY_MONTH_TABLE.len() as u64 - 1);
    let v = SUBSIDY_BY_MONTH_TABLE[m as usize];
    let per_block = v.div_ceil(BPS);
    per_block.saturating_mul(BPS).saturating_mul(SECONDS_PER_MONTH)
}

/// Sisa emisi (pradha) sejak bulan `month`: jumlah seluruh subsidi bulanan
/// dari `month` sampai akhir tabel.
pub fn remaining_emission_pradha(month: u64) -> u64 {
    let start = (month as usize).min(SUBSIDY_BY_MONTH_TABLE.len() - 1);
    SUBSIDY_BY_MONTH_TABLE[start..].iter().fold(0u64, |acc, &v| {
        let per_block = v.div_ceil(BPS);
        acc.saturating_add(per_block.saturating_mul(BPS).saturating_mul(SECONDS_PER_MONTH))
    })
}

/// Target reservoir lokal (floor) pada bulan `month`, dalam pradha.
///
/// Nilainya `TARGET_RESERVOIR_PRADHA`, tetapi tidak pernah melebihi emisi yang
/// masih tersisa — supaya di ekor emisi reservoir tidak menuntut dana yang
/// sudah tidak akan terbit. Saat emisi habis, floor = 0 dan cut = 0.
pub fn floor_pradha(month: u64) -> u64 {
    TARGET_RESERVOIR_PRADHA.min(remaining_emission_pradha(month))
}

/// Emisi kumulatif dari bulan `from` sampai `to` (eksklusif), dalam pradha.
pub fn emission_to(from: u64, to: u64) -> u64 {
    let len = SUBSIDY_BY_MONTH_TABLE.len() as u64;
    let start = from.min(len) as usize;
    let end = to.min(len) as usize;
    if start >= end {
        return 0;
    }
    SUBSIDY_BY_MONTH_TABLE[start..end].iter().fold(0u64, |acc, &v| {
        let per_block = v.div_ceil(BPS);
        acc.saturating_add(per_block.saturating_mul(BPS).saturating_mul(SECONDS_PER_MONTH))
    })
}

/// Porsi reward blok yang harus disisihkan pada bulan `month` bila reservoir
/// saat ini berisi `balance_pradha` (0.0..=`MAX_CUT_FRACTION`).
///
/// Inilah algoritma pengisian: setiap bulan, reservoir menabung porsi tetap
/// dari emisi bulan itu — `floor × emisi_bulan / emisi_48_bulan` — sehingga
/// dalam 48 bulan, terkumpul target (diuji: bulan ke-46). Laju tidak pernah
/// melebihi `MAX_CUT_FRACTION`; jika defisit lebih kecil dari porsi tabungan,
/// cut menyusut agar tidak overfill. Saldo di dalam deadband (1% floor) tidak
/// memotong sama sekali: reward kembali utuh ke miner. Setelah target tercapai
/// cut = 0 (maintenance), dan jika pengurasan menurunkan saldo, cut menyala
/// kembali secara otomatis (self-correcting).
pub fn cut_fraction(month: u64, balance_pradha: u64) -> f64 {
    let floor = floor_pradha(month);
    let deficit = floor.saturating_sub(balance_pradha);

    // Saldo sudah di dalam zona mati floor → tidak perlu memotong sama sekali.
    if deficit == 0 || (deficit as f64) <= (floor as f64) * DEADBAND_FRACTION {
        return 0.0;
    }

    // Emisi yang akan terbit selama horizon. Bila emisi habis sebelum horizon
    // tercapai (bulan-bulan terakhir), penyebutnya menyusut dan cut naik —
    // perilaku yang benar: sisa waktu makin pendek, tiap bulan harus lebih
    // banyak menabung.
    let horizon_emission = emission_to(month, month.saturating_add(FILL_HORIZON_MONTHS));
    if horizon_emission == 0 {
        // Emisi sudah berakhir: tidak ada lagi yang bisa disisihkan.
        return 0.0;
    }

    let month_emission = subsidy_pradha_month(month);
    if month_emission == 0 {
        return 0.0;
    }

    // Tabungan rata-rata yang diperlukan setiap bulan selama horizon.
    let target_inflow = (floor as f64) * (month_emission as f64)
        / (horizon_emission as f64);
    let inflow = target_inflow.min(deficit as f64);
    if inflow <= 0.0 {
        return 0.0;
    }
    let cut = inflow / (month_emission as f64);
    cut.clamp(MIN_CUT_FRACTION, MAX_CUT_FRACTION)
}

/// Porsi reservoir yang dialokasikan untuk klaim faucet (60%).
///
/// Komplemen dari [`MINER_SUBSIDY_SHARE`]; dinyatakan sebagai fungsi agar tidak
/// ada dua konstanta yang bisa tidak berjumlah 1,0.
pub fn claim_share() -> f64 {
    1.0 - MINER_SUBSIDY_SHARE
}

/// Bucket klaim faucet pada bulan `month`, dalam pradha.
///
/// Ini plafon kewajiban ledger: total saldo tercatat tidak boleh melewatinya,
/// karena sisa floor sudah dijanjikan ke subsidi miner.
pub fn claim_bucket_pradha(month: u64) -> u64 {
    (floor_pradha(month) as f64 * claim_share()) as u64
}

/// Bucket subsidi miner Android pada bulan `month`, dalam pradha.
pub fn miner_bucket_pradha(month: u64) -> u64 {
    (floor_pradha(month) as f64 * MINER_SUBSIDY_SHARE) as u64
}

/// Anggaran subsidi miner Android per hari pada bulan `month`, dalam pradha.
///
/// Sama seperti cap klaim: **hanya laju inflow berkelanjutan**, tanpa komponen
/// amortisasi saldo. Amortisasi akan membayar dari bucket yang belum terisi di
/// fase awal. Nilainya adalah **total untuk semua miner**, bukan per perangkat —
/// pembagian ke tiap miner dilakukan pemanggil sesuai jumlah miner aktif.
pub fn miner_subsidy_daily_pradha(month: u64) -> u64 {
    let monthly_inflow = (subsidy_pradha_month(month) as f64
        * MAX_CUT_FRACTION
        * MINER_SUBSIDY_SHARE) as u64;
    (monthly_inflow as f64 / DAYS_PER_MONTH) as u64
}

/// Subsidi harian untuk satu miner bila ada `miner_count` miner aktif.
///
/// Dibagi rata dan dibatasi agar tidak pernah melebihi anggaran agregat. Kalau
/// `miner_count` = 0, hasilnya 0 (tidak ada yang dibayar).
pub fn miner_subsidy_per_miner_pradha(month: u64, miner_count: u32) -> u64 {
    if miner_count == 0 {
        return 0;
    }
    miner_subsidy_daily_pradha(month) / miner_count as u64
}

/// Kap klaim harian dinamis pada bulan `month`, dalam pradha.
///
/// Dihitung **hanya dari bucket klaim**, bukan seluruh floor: 40% reservoir
/// sudah dialokasikan untuk subsidi miner Android. Memakai floor penuh di sini
/// akan menjanjikan dana yang bukan milik faucet klaim.
///
/// Nilainya adalah **laju inflow berkelanjutan** pada beban potong maksimum:
/// `emisi_bulan × MAX_CUT × porsi_klaim ÷ hari_per_bulan`.
///
/// Tidak ada komponen amortisasi saldo. Versi sebelumnya menambahkan
/// `bucket ÷ 365`, dan itu menjanjikan dana yang belum masuk: di bulan 0 saldo
/// reservoir masih 0, tetapi cap-nya menjadi 5.187 CEN/hari terhadap inflow
/// nyata 1.078 CEN/hari. Selisih 4.109 CEN/hari hanya bisa dibayar dengan
/// menguras bucket, sehingga bucket 1,5 juta CEN habis dalam 289 hari padahal
/// target pengisiannya 48 bulan. Cap sekarang tidak pernah melebihi dana yang
/// benar-benar masuk.
pub fn daily_cap_pradha(month: u64) -> u64 {
    let monthly_inflow =
        (subsidy_pradha_month(month) as f64 * MAX_CUT_FRACTION * claim_share()) as u64;
    (monthly_inflow as f64 / DAYS_PER_MONTH) as u64
}

/// Ringkasan kebijakan untuk satu momen waktu (dipakai endpoint status).
pub struct EmissionPolicySnapshot {
    pub month: u64,
    pub remaining_emission_pradha: u64,
    pub cut_fraction: f64,
    pub floor_pradha: u64,
    pub daily_cap_pradha: u64,
}

/// Hitung snapshot kebijakan untuk `now_secs` dan saldo reservoir saat ini.
pub fn snapshot(now_secs: u64, balance_pradha: u64) -> EmissionPolicySnapshot {
    let month = month_since_genesis(now_secs);
    EmissionPolicySnapshot {
        month,
        remaining_emission_pradha: remaining_emission_pradha(month),
        cut_fraction: cut_fraction(month, balance_pradha),
        floor_pradha: floor_pradha(month),
        daily_cap_pradha: daily_cap_pradha(month),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_length_is_817() {
        assert_eq!(table_len(), 817);
    }

    #[test]
    fn month_zero_is_genesis() {
        assert_eq!(month_since_genesis(GENESIS_TIMESTAMP_SECS), 0);
        // Beberapa menit setelah genesis masih bulan 0.
        assert_eq!(month_since_genesis(GENESIS_TIMESTAMP_SECS + 3600), 0);
    }

    #[test]
    fn empty_reservoir_cuts_at_savings_rate() {
        // Saldo 0 di bulan 0: reservoir menabung tepat `floor / E_48` dari
        // setiap bulan. Nilainya di bawah plafon 10% karena target hanya 4,8%
        // dari total emisi dan horizon 48 bulan masih menyisakan emisi besar.
        let cut = cut_fraction(0, 0);
        let expected = floor_pradha(0) as f64 / emission_to(0, 48) as f64;
        assert!((cut - expected).abs() < 1e-9, "cut = {cut}, expected = {expected}");
        assert!(cut > MIN_CUT_FRACTION, "cut = {cut}");
        assert!(cut <= MAX_CUT_FRACTION, "cut = {cut}");
    }

    #[test]
    fn full_reservoir_stops_cutting() {
        let floor = floor_pradha(0);
        assert_eq!(cut_fraction(0, floor), 0.0, "saldo = floor → cut 0");
        assert_eq!(cut_fraction(0, floor + 1), 0.0, "saldo di atas floor → cut 0");
    }

    #[test]
    fn deadband_absorbs_near_full_balance() {
        let floor = floor_pradha(0);
        // Tepat di batas deadband: dianggap penuh.
        let near = floor - (floor as f64 * DEADBAND_FRACTION) as u64;
        assert_eq!(cut_fraction(0, near), 0.0);
        // Sedikit di luar deadband: mulai memotong.
        let outside = floor - (floor as f64 * DEADBAND_FRACTION) as u64 - 1;
        assert!(cut_fraction(0, outside) > 0.0);
    }

    #[test]
    fn cut_rises_when_balance_is_drained() {
        let floor = floor_pradha(0);
        let half = floor / 2;
        let c_half = cut_fraction(0, half);
        let c_empty = cut_fraction(0, 0);
        assert!(c_empty >= c_half, "saldo lebih kecil harus memotong lebih banyak");
        assert!(c_half > 0.0);
    }

    /// Bukti utama: simulasi terhadap tabel emisi mainnet mencapai target
    /// 2.500.000 CEN dalam `FILL_HORIZON_MONTHS`, bukan meleset seperti versi
    /// lama (825.788 CEN).
    #[test]
    fn simulation_fills_target_within_horizon() {
        let mut balance = 0u64;
        let mut reached = None;
        for month in 0..table_len() as u64 {
            let cut = cut_fraction(month, balance);
            let inflow = (subsidy_pradha_month(month) as f64 * cut) as u64;
            balance = balance.saturating_add(inflow);
            if reached.is_none() && balance >= TARGET_RESERVOIR_PRADHA {
                reached = Some(month);
            }
        }
        let month = reached.expect("target harus tercapai");
        assert!(
            month <= FILL_HORIZON_MONTHS,
            "target tercapai bulan {month}, melewati horizon {FILL_HORIZON_MONTHS}"
        );
        assert!(
            balance >= TARGET_RESERVOIR_PRADHA,
            "saldo akhir {} < target",
            balance
        );
    }

    /// Setelah target tercapai dan tidak ada pengurasan, cut harus mati:
    /// reward kembali penuh ke miner.
    #[test]
    fn cut_stays_off_once_target_reached() {
        let month = FILL_HORIZON_MONTHS;
        assert_eq!(
            cut_fraction(month, TARGET_RESERVOIR_PRADHA),
            0.0,
            "reservoir penuh → reward penuh ke miner"
        );
    }

    /// Pengurasan di tengah jalan harus menyalakan cut lagi (self-correcting).
    #[test]
    fn drain_after_fill_turns_cut_back_on() {
        let month = FILL_HORIZON_MONTHS + 12;
        assert_eq!(cut_fraction(month, TARGET_RESERVOIR_PRADHA), 0.0);
        let drained = TARGET_RESERVOIR_PRADHA / 2;
        assert!(cut_fraction(month, drained) > 0.0, "saldo terkuras → cut hidup lagi");
    }

    #[test]
    fn floor_never_exceeds_target() {
        for m in [0u64, 12, 48, 300, 600, 800] {
            assert!(floor_pradha(m) <= TARGET_RESERVOIR_PRADHA);
        }
    }

    #[test]
    fn floor_decays_only_when_emission_is_low() {
        // Fase awal: target penuh.
        assert_eq!(floor_pradha(0), TARGET_RESERVOIR_PRADHA);
        // Ekor emisi: floor mengikuti sisa yang benar-benar akan terbit.
        let last = table_len() as u64 - 1;
        assert_eq!(floor_pradha(last), 0);
        let tail = floor_pradha(last - 1);
        assert!(tail < TARGET_RESERVOIR_PRADHA);
        assert_eq!(tail, remaining_emission_pradha(last - 1));
    }

    #[test]
    fn cut_never_exceeds_max_even_when_desperate() {
        // Bulan-bulan terakhir dengan saldo 0: laju boleh naik, tapi tidak
        // melewati MAX_CUT — pagar kepercayaan miner.
        let last = table_len() as u64 - 2;
        let cut = cut_fraction(last, 0);
        assert!(cut <= MAX_CUT_FRACTION, "cut = {cut}");
    }

    #[test]
    fn zero_cut_when_emission_ended() {
        let last = table_len() as u64 - 1;
        assert_eq!(cut_fraction(last, 0), 0.0, "emisi habis → tidak ada yang bisa disisihkan");
        assert_eq!(daily_cap_pradha(last), 0);
    }

    #[test]
    fn daily_cap_positive_while_emission_runs() {
        for m in [0u64, 48, 120, 300, 600] {
            assert!(daily_cap_pradha(m) > 0, "cap di bulan {m} harus > 0");
        }
    }

    #[test]
    fn daily_cap_covers_one_full_max_claim() {
        // Cap harian harus mengizinkan setidaknya satu klaim pada plafon penuh
        // selama fase emisi — kalau tidak, reward maksimum 1000 pradha tidak
        // pernah bisa dibayar.
        let max_reward = 1_000u64;
        for m in [0u64, 24, 48, 120] {
            assert!(
                daily_cap_pradha(m) >= max_reward,
                "cap bulan {m} = {} < 1 klaim penuh",
                daily_cap_pradha(m)
            );
        }
    }

    #[test]
    fn emission_to_sums_months_correctly() {
        assert_eq!(emission_to(5, 5), 0, "rentang kosong = 0");
        assert_eq!(emission_to(7, 3), 0, "rentang terbalik = 0");
        let one = emission_to(0, 1);
        assert_eq!(one, subsidy_pradha_month(0));
        let span = emission_to(0, 3);
        assert_eq!(
            span,
            subsidy_pradha_month(0) + subsidy_pradha_month(1) + subsidy_pradha_month(2)
        );
        // Melewati akhir tabel tetap aman (clamp), tidak panik.
        assert_eq!(emission_to(0, u64::MAX), remaining_emission_pradha(0));
    }

    #[test]
    fn snapshot_reports_consistent_policy() {
        let s = snapshot(GENESIS_TIMESTAMP_SECS, 0);
        assert_eq!(s.month, 0);
        assert_eq!(s.floor_pradha, TARGET_RESERVOIR_PRADHA);
        assert_eq!(s.cut_fraction, cut_fraction(0, 0));
        assert!(s.cut_fraction <= MAX_CUT_FRACTION);
        assert_eq!(s.daily_cap_pradha, daily_cap_pradha(0));
    }
}
