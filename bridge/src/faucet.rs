//! Faucet klaim harian: buku besar off-chain partisipasi.
//!
//! Setiap alamat boleh klaim `FAUCET_REWARD_PRADHA` sekali per hari (hari Unix).
//! Saldo tercatat off-chain sampai user menariknya on-chain. Ini "rewards
//! partisipasi", bukan mining — lihat `docs/faucet-reservoir-spec.md`.
//!
//! Akuntansi memakai pradha (1 CEN = 1e8 pradha) agar integer, tanpa float.

use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Reward per klaim: 1000 pradha = 0.00001 CEN.
pub const FAUCET_REWARD_PRADHA: u64 = 1_000;

/// Cap anggaran harian total: 250 CEN = 25_000_000_000 pradha.
pub const FAUCET_DAILY_CAP_PRADHA: u64 = 25_000_000_000;

/// File tempat ledger faucet disimpan (relatif ke working directory bridge).
pub const FAUCET_LEDGER_PATH: &str = "faucet_ledger.json";

/// Maksimal address wallet berbeda per device (Android ID) — anti-sybil lunak.
pub const FAUCET_MAX_ACCOUNTS_PER_DEVICE: usize = 5;

/// Detik per hari (hari Unix untuk throttling klaim).
const DAY_SECS: u64 = 86_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct FaucetEntry {
    pub balance_pradha: u64,
    pub last_claim_day: u64,
    pub total_claimed_pradha: u64,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FaucetState {
    ledger: HashMap<String, FaucetEntry>,
    /// device_id (Android ID) → set address wallet yang pernah klaim.
    devices: HashMap<String, HashSet<String>>,
    /// Total pradha dikeluarkan pada hari Unix `claim_day`.
    claimed_today: u64,
    /// Hari Unix yang sedang dihitung untuk `claimed_today`.
    claim_day: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimError {
    AlreadyClaimed,
    DailyCapReached,
    EmptyAddress,
    TooManyAccounts,
}

impl std::fmt::Display for ClaimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ClaimError::AlreadyClaimed => "already claimed today",
            ClaimError::DailyCapReached => "daily faucet cap reached",
            ClaimError::EmptyAddress => "address is empty",
            ClaimError::TooManyAccounts => "device has too many accounts",
        };
        f.write_str(s)
    }
}

impl std::error::Error for ClaimError {}

impl FaucetState {
    pub fn current_day() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() / DAY_SECS)
            .unwrap_or(0)
    }

    fn roll_day(&mut self) {
        let day = Self::current_day();
        if day != self.claim_day {
            self.claim_day = day;
            self.claimed_today = 0;
        }
    }

    /// Klaim harian untuk `address`. Kembalikan saldo baru bila berhasil.
    pub fn claim(&mut self, address: &str, device_id: &str) -> Result<FaucetEntry, ClaimError> {
        if address.trim().is_empty() {
            return Err(ClaimError::EmptyAddress);
        }
        // Anti-sybil lunak: batasi address wallet per device (Android ID). Kosong
        // = tak terlacak (backward-compat, hanya dibatasi per-address).
        let did = device_id.trim();
        if !did.is_empty() {
            let set = self.devices.entry(did.to_string()).or_default();
            if !set.contains(address) && set.len() >= FAUCET_MAX_ACCOUNTS_PER_DEVICE {
                return Err(ClaimError::TooManyAccounts);
            }
            set.insert(address.to_string());
        }
        self.roll_day();
        let day = self.claim_day;
        let entry = self.ledger.entry(address.to_string()).or_default();
        if entry.last_claim_day == day {
            return Err(ClaimError::AlreadyClaimed);
        }
        if self.claimed_today + FAUCET_REWARD_PRADHA > FAUCET_DAILY_CAP_PRADHA {
            return Err(ClaimError::DailyCapReached);
        }
        entry.balance_pradha += FAUCET_REWARD_PRADHA;
        entry.total_claimed_pradha += FAUCET_REWARD_PRADHA;
        entry.last_claim_day = day;
        self.claimed_today += FAUCET_REWARD_PRADHA;
        Ok(*entry)
    }

    pub fn balance(&self, address: &str) -> Option<&FaucetEntry> {
        self.ledger.get(address)
    }

    /// Kurangi saldo (dipanggil setelah withdraw on-chain berhasil).
    pub fn debit(&mut self, address: &str, amount_pradha: u64) -> bool {
        if let Some(e) = self.ledger.get_mut(address)
            && e.balance_pradha >= amount_pradha
        {
            e.balance_pradha -= amount_pradha;
            return true;
        }
        false
    }

    /// Total saldo tergabung (untuk laporan/reservoir check).
    pub fn total_outstanding(&self) -> u64 {
        self.ledger.values().map(|e| e.balance_pradha).sum()
    }

    pub fn len(&self) -> usize {
        self.ledger.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ledger.is_empty()
    }

    /// Klaim hari ini yang sudah dihitung (0 bila sudah ganti hari).
    pub fn claimed_today(&self) -> u64 {
        if Self::current_day() != self.claim_day {
            0
        } else {
            self.claimed_today
        }
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        match std::fs::read(path) {
            Ok(bytes) => match serde_json::from_slice(&bytes) {
                Ok(s) => Ok(s),
                Err(_) => Ok(Self::default()),
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e),
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let bytes = serde_json::to_vec_pretty(self)?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADDR: &str = "cen69:test0000000000000000000000000000000000000000";

    #[test]
    fn claims_once_per_day() {
        let mut s = FaucetState::default();
        let day = FaucetState::current_day();
        s.ledger.insert(
            ADDR.to_string(),
            FaucetEntry {
                balance_pradha: 0,
                last_claim_day: day.saturating_sub(1),
                total_claimed_pradha: 0,
            },
        );

        assert_eq!(s.claim(ADDR, "").unwrap().balance_pradha, FAUCET_REWARD_PRADHA);
        assert_eq!(s.claim(ADDR, ""), Err(ClaimError::AlreadyClaimed));
    }

    #[test]
    fn rejects_empty_address() {
        let mut s = FaucetState::default();
        assert_eq!(s.claim("", ""), Err(ClaimError::EmptyAddress));
        assert_eq!(s.claim("   ", ""), Err(ClaimError::EmptyAddress));
    }

    #[test]
    fn debit_deducts_balance() {
        let mut s = FaucetState::default();
        s.ledger.insert(
            ADDR.to_string(),
            FaucetEntry {
                balance_pradha: 5000,
                last_claim_day: 0,
                total_claimed_pradha: 5000,
            },
        );
        assert!(s.debit(ADDR, 3000));
        assert_eq!(s.balance(ADDR).unwrap().balance_pradha, 2000);
        assert!(!s.debit(ADDR, 9999));
    }

    #[test]
    fn total_outstanding_tracks_ledger() {
        let mut s = FaucetState::default();
        s.ledger.insert(
            "a".into(),
            FaucetEntry {
                balance_pradha: 1000,
                last_claim_day: 0,
                total_claimed_pradha: 1000,
            },
        );
        s.ledger.insert(
            "b".into(),
            FaucetEntry {
                balance_pradha: 2000,
                last_claim_day: 0,
                total_claimed_pradha: 2000,
            },
        );
        assert_eq!(s.total_outstanding(), 3000);
    }

    #[test]
    fn daily_cap_stops_issuance() {
        let mut s = FaucetState::default();
        s.claim_day = FaucetState::current_day();
        s.claimed_today = FAUCET_DAILY_CAP_PRADHA;
        assert_eq!(s.claim(ADDR, ""), Err(ClaimError::DailyCapReached));
    }

    #[test]
    fn device_cap_limits_accounts() {
        let mut s = FaucetState::default();
        // Daftarkan 5 address berbeda pada device yang sama.
        for i in 0..FAUCET_MAX_ACCOUNTS_PER_DEVICE {
            let a = format!("cen69:test{i:042}");
            s.ledger.insert(
                a.clone(),
                FaucetEntry { balance_pradha: 0, last_claim_day: 0, total_claimed_pradha: 0 },
            );
            s.devices.entry("dev-1".to_string()).or_default().insert(a);
        }
        // Address ke-6 ditolak.
        let sixth = "cen69:test9999999999999999999999999999999999999999";
        assert_eq!(s.claim(sixth, "dev-1"), Err(ClaimError::TooManyAccounts));
        // Device lain masih diizinkan.
        assert!(s.claim(sixth, "dev-2").is_ok());
    }

    #[test]
    fn save_load_roundtrip() {
        let mut s = FaucetState::default();
        s.ledger.insert(
            ADDR.to_string(),
            FaucetEntry {
                balance_pradha: 7000,
                last_claim_day: 5,
                total_claimed_pradha: 7000,
            },
        );
        let p = std::env::temp_dir().join("faucet_test_roundtrip.json");
        s.save(&p).unwrap();
        let loaded = FaucetState::load(&p).unwrap();
        assert_eq!(loaded.balance(ADDR).unwrap().balance_pradha, 7000);
        let _ = std::fs::remove_file(&p);
    }
}