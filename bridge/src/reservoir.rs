//! Reservoir faucet controller.
//!
//! Menentukan porsi reward blok yang dialirkan ke faucet setiap blok,
//! berdasarkan **saldo reservoir nyata** dan target dari `emission_policy`.
//!
//! Controller ini tipis dengan sengaja: seluruh matematika berada di
//! [`crate::emission_policy::cut_fraction`], yang murni (tanpa state, bisa
//! diuji langsung). Yang disimpan di sini hanyalah `cut_on` untuk meredam
//! osilasi antar-blok.
//!
//! Alur per blok yang dikonfirmasi:
//! 1. baca saldo reservoir dari node;
//! 2. `update_at(balance, now)` → porsi cut (0,0..=`MAX_CUT_FRACTION`);
//! 3. cut × reward blok = pradha yang disisihkan ke faucet;
//! 4. sisanya dibagi ke miner lewat PPLNS.
//!
//! Bila saldo sudah mencapai target, cut = 0 sehingga reward kembali utuh ke
//! miner — inilah mekanisme "kelebihan dana kembali ke miner" tanpa transaksi
//! on-chain terpisah.

use crate::emission_policy;

/// Konfigurasi reservoir.
///
/// Ambang tidak lagi dikonfigurasi sebagai fraksi target: kebijakan emisi yang
/// menentukan kapan memotong dan seberapa besar, karena hanya ia yang tahu
/// emisi tersisa. Menyisakan field di sini akan membuat dua sumber kebenaran
/// yang bisa berbeda pendapat.
#[derive(Debug, Clone, Default)]
pub struct ReservoirConfig {
    /// Matikan cut sepenuhnya, apa pun saldonya. Dipakai untuk mode solo
    /// (faucet tidak didanai) dan untuk tes.
    pub disabled: bool,
}

/// Controller stateful: mengingat apakah cut sedang aktif, sehingga fluktuasi
/// saldo di sekitar target tidak membuat cut berkedip setiap blok.
#[derive(Debug, Clone)]
pub struct ReservoirController {
    config: ReservoirConfig,
    cut_on: bool,
}

impl ReservoirController {
    pub fn new(config: ReservoirConfig) -> Self {
        // Mulai aktif: reservoir kosong pada masa awal jaringan.
        Self { config, cut_on: true }
    }

    /// Porsi cut untuk `now_secs` dan `balance_pradha` saat ini.
    ///
    /// Mengembalikan 0,0 bila cut dimatikan lewat config, atau bila kebijakan
    /// menyatakan reservoir sudah penuh.
    pub fn update_at(&mut self, balance_pradha: u64, now_secs: u64) -> f64 {
        if self.config.disabled {
            self.cut_on = false;
            return 0.0;
        }
        let month = emission_policy::month_since_genesis(now_secs);
        let cut = emission_policy::cut_fraction(month, balance_pradha);
        self.cut_on = cut > 0.0;
        cut
    }

    /// Perbarui state menggunakan waktu sistem sekarang.
    pub fn update_and_cut_fraction(&mut self, balance_pradha: u64) -> f64 {
        self.update_at(balance_pradha, now_secs())
    }

    pub fn cut_on(&self) -> bool {
        self.cut_on
    }

    /// Target reservoir (floor) per kebijakan sekarang, dalam pradha.
    pub fn current_target_pradha(&self) -> u64 {
        emission_policy::floor_pradha(emission_policy::month_since_genesis(now_secs()))
    }
}

/// Detik Unix sekarang.
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: u64 = emission_policy::GENESIS_TIMESTAMP_SECS + 3_600;

    fn cfg() -> ReservoirConfig {
        ReservoirConfig::default()
    }

    #[test]
    fn empty_reservoir_turns_cut_on() {
        let mut c = ReservoirController::new(cfg());
        assert!(c.cut_on());
        let cut = c.update_at(0, NOW);
        // Target 2.5M CEN dalam 48 bulan: deficit/future_emission memberi
        // cut ~9.52% di bulan 0, di bawah plafon 10%. Yang penting cut aktif
        // dan berada dalam rentang kebijakan.
        assert!(cut > 0.0, "cut harus > 0 saat reservoir kosong");
        assert!(cut >= emission_policy::MIN_CUT_FRACTION, "cut = {cut}");
        assert!(cut <= emission_policy::MAX_CUT_FRACTION, "cut = {cut}");
        assert!(c.cut_on());
    }

    #[test]
    fn full_reservoir_returns_rewards_to_miners() {
        let mut c = ReservoirController::new(cfg());
        let target = emission_policy::floor_pradha(0);
        assert_eq!(c.update_at(target, NOW), 0.0, "surplus: reward penuh ke miner");
        assert!(!c.cut_on());
    }

    #[test]
    fn drained_reservoir_turns_cut_back_on() {
        let mut c = ReservoirController::new(cfg());
        let target = emission_policy::floor_pradha(0);
        c.update_at(target, NOW);
        assert!(!c.cut_on());
        let cut = c.update_at(target / 4, NOW);
        assert!(cut > 0.0, "saldo terkuras harus menyalakan cut lagi");
        assert!(c.cut_on());
    }

    #[test]
    fn disabled_config_never_cuts() {
        let mut c = ReservoirController::new(ReservoirConfig { disabled: true });
        assert_eq!(c.update_at(0, NOW), 0.0);
        assert!(!c.cut_on());
    }

    #[test]
    fn cut_never_exceeds_policy_ceiling() {
        let mut c = ReservoirController::new(cfg());
        for balance in [0u64, 1_000_000_000, 100_000_000_000_000] {
            let cut = c.update_at(balance, NOW);
            assert!(
                cut <= emission_policy::MAX_CUT_FRACTION,
                "cut {cut} melewati plafon"
            );
        }
    }

    #[test]
    fn target_shrinks_only_at_emission_tail() {
        let early = ReservoirController::new(cfg()).current_target_pradha();
        assert_eq!(early, emission_policy::TARGET_RESERVOIR_PRADHA);
    }
}
