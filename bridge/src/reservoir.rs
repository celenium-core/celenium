//! Reservoir faucet controller.
//!
//! Menentukan porsi reward blok yang dialirkan ke faucet (0 atau `cut_fraction`)
//! berdasarkan saldo reservoir, dengan hysteresis agar tidak berosilasi antar blok.
//! Threshold hanya untuk keputusan on/off — akuntansi nilai tetap integer (pradha).

#[derive(Debug, Clone)]
pub struct ReservoirConfig {
    /// Porsi reward blok ke faucet saat cut aktif (mis. 0.015 = 1.5%).
    pub cut_fraction: f64,
    /// Level target reservoir dalam pradha (mis. 7500 CEN = 750_000_000_000).
    pub target_pradha: u64,
    /// Cut AKTIF saat saldo <= target * hysteresis_low.
    pub hysteresis_low: f64,
    /// Cut NONAKTIF saat saldo >= target * hysteresis_high.
    pub hysteresis_high: f64,
}

impl ReservoirConfig {
    pub fn low_threshold_pradha(&self) -> u64 {
        (self.target_pradha as f64 * self.hysteresis_low) as u64
    }

    pub fn high_threshold_pradha(&self) -> u64 {
        (self.target_pradha as f64 * self.hysteresis_high) as u64
    }
}

/// Controller stateful: mengingat apakah cut sedang aktif, sehingga zona tengah
/// (di antara low/high) mempertahankan state sebelumnya — inti dari hysteresis.
#[derive(Debug, Clone)]
pub struct ReservoirController {
    config: ReservoirConfig,
    cut_on: bool,
}

impl ReservoirController {
    pub fn new(config: ReservoirConfig) -> Self {
        // Default aktif: reservoir kosong pada masa awal jaringan, jadi cut hidup.
        Self { config, cut_on: true }
    }

    /// Perbarui state berdasarkan saldo reservoir, kembalikan porsi cut saat ini.
    /// Nilai kembalian: 0.0 (cut nonaktif) atau `config.cut_fraction` (cut aktif).
    pub fn update_and_cut_fraction(&mut self, balance_pradha: u64) -> f64 {
        let low = self.config.low_threshold_pradha();
        let high = self.config.high_threshold_pradha();
        if balance_pradha <= low {
            self.cut_on = true;
        } else if balance_pradha >= high {
            self.cut_on = false;
        }
        if self.cut_on { self.config.cut_fraction } else { 0.0 }
    }

    pub fn cut_on(&self) -> bool {
        self.cut_on
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ReservoirConfig {
        ReservoirConfig {
            cut_fraction: 0.015,
            target_pradha: 750_000_000_000, // 7500 CEN
            hysteresis_low: 0.8,
            hysteresis_high: 1.2,
        }
    }

    #[test]
    fn empty_reservoir_turns_cut_on() {
        let mut c = ReservoirController::new(cfg());
        assert!(c.cut_on());
        assert_eq!(c.update_and_cut_fraction(0), 0.015);
    }

    #[test]
    fn above_high_threshold_turns_cut_off() {
        let mut c = ReservoirController::new(cfg());
        // high = 1.2 * 750e9 = 900e9
        assert_eq!(c.update_and_cut_fraction(900_000_000_000), 0.0);
        assert!(!c.cut_on());
    }

    #[test]
    fn below_low_threshold_turns_cut_on() {
        let mut c = ReservoirController::new(cfg());
        // matikan dulu
        c.update_and_cut_fraction(900_000_000_000);
        assert!(!c.cut_on());
        // low = 0.8 * 750e9 = 600e9
        assert_eq!(c.update_and_cut_fraction(600_000_000_000), 0.015);
        assert!(c.cut_on());
    }

    #[test]
    fn hysteresis_keeps_previous_state_in_middle_zone() {
        let mut c = ReservoirController::new(cfg());
        // Mulai cut on, saldo di zona tengah (antara 600e9 dan 900e9)
        assert_eq!(c.update_and_cut_fraction(700_000_000_000), 0.015);
        assert!(c.cut_on());
        // Tetap on meski di zona tengah
        assert_eq!(c.update_and_cut_fraction(800_000_000_000), 0.015);
        assert!(c.cut_on());

        // Matikan lewat ambang atas
        c.update_and_cut_fraction(900_000_000_001);
        assert!(!c.cut_on());
        // Zona tengah kini mempertahankan OFF
        assert_eq!(c.update_and_cut_fraction(700_000_000_000), 0.0);
        assert!(!c.cut_on());
    }
}