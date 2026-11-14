//! Faucet klaim harian: buku besar off-chain partisipasi.
//!
//! Setiap alamat boleh klaim sekali per hari (hari Unix). Nilai reward-nya
//! **dinamis**: plafon `FAUCET_MAX_REWARD_PRADHA` (1000 pradha = 0,00001 CEN)
//! bila peserta sedikit, menyusut proporsional ketika peserta bertambah karena
//! anggaran harian dibagi rata ke jumlah peserta yang terukur. Saldo tercatat
//! off-chain sampai user menariknya on-chain. Ini "rewards partisipasi", bukan
//! mining — lihat `docs/faucet-reservoir-spec.md`.
//!
//! Akuntansi memakai pradha (1 CEN = 1e8 pradha) agar integer, tanpa float.

use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Hadiah partisipasi **maksimum** per klaim harian (pradha; 1 CEN = 1e8 pradha).
///
/// 1000 pradha = 0,00001 CEN per perangkat per hari. Ini plafon, bukan janji:
/// reward yang benar-benar dibayar menyusut ketika peserta bertambah, karena
/// anggaran harian dibagi rata ke jumlah peserta yang terukur
/// (lihat [`FaucetState::reward_pradha_today`]).
pub const FAUCET_MAX_REWARD_PRADHA: u64 = 1_000;

/// Batas total kewajiban ledger (semua saldo yang belum ditarik), dalam pradha.
/// Cap harian hanya membatasi emisi per hari; tanpa batas akumulatif, kewajiban
/// menumpuk selamanya dan faucet berutang lebih besar daripada reservoir yang
/// menopangnya. Nilai ini = 2.500.000 CEN, sama dengan target reservoir: klaim
/// baru berhenti begitu total saldo tercatat mencapai titik di mana reservoir
/// seharusnya sudah penuh.
///
/// Ini plafon keras keselamatan. Batas operasional sehari-hari adalah floor
/// dinamis ([`crate::emission_policy::floor_pradha`]), yang menyusut mengikuti
/// sisa emisi sehingga kewajiban tidak pernah menumpuk melebihi dana yang bisa
/// datang.
pub const FAUCET_MAX_OUTSTANDING_PRADHA: u64 = 250_000_000_000_000; // 2.500.000 CEN

/// File tempat ledger faucet disimpan (relatif ke working directory bridge).
pub const FAUCET_LEDGER_PATH: &str = "faucet_ledger.json";

/// Maksimal address wallet berbeda per device (Android ID) — anti-sybil lunak.
pub const FAUCET_MAX_ACCOUNTS_PER_DEVICE: usize = 5;

/// Jeda minimum antar-withdraw untuk satu alamat, dalam detik.
/// Withdraw tanpa minimum nominal berarti tiap penarikan tetap memakan satu fee
/// on-chain dari reservoir. Tanpa jeda, saldo kecil bisa ditarik berulang kali
/// dan biayanya jauh melampaui nilai yang dikirim.
pub const FAUCET_WITHDRAW_COOLDOWN_SECS: u64 = 3_600;

/// Anggaran fee on-chain per hari, dalam pradha (2,5 CEN).
/// Ini backstop terakhir: berapa pun jumlah alamat yang dipakai, reservoir tidak
/// pernah membayar lebih dari plafon ini untuk biaya transaksi per hari.
pub const FAUCET_DAILY_FEE_BUDGET_PRADHA: u64 = 250_000_000;

/// Nominal withdraw minimum, dalam pradha (0,0005 CEN).
///
/// Fee transfer on-chain yang dibayar reservoir ≈ 25.000 pradha, jadi ambang ini
/// menjaga nilai yang dikirim tetap dua kali biayanya. Dengan reward maksimum
/// 1000 pradha/hari, saldo ini terkumpul dalam ~50 hari partisipasi — cukup
/// jarang untuk tidak membanjiri jaringan dengan transaksi receh.
pub const FAUCET_MIN_WITHDRAW_PRADHA: u64 = 50_000;

/// Reward minimum per klaim, dalam pradha.
///
/// Pradha adalah satuan terkecil, jadi 1 adalah lantai mutlak. Nilai ini hanya
/// terpakai ketika anggaran harian lebih kecil daripada jumlah peserta: pada
/// titik itu yang dibatasi adalah **jumlah klaim terlayani**, bukan nilai per
/// klaim.
pub const FAUCET_MIN_REWARD_PRADHA: u64 = 1;

/// Detik per hari (hari Unix untuk throttling klaim).
const DAY_SECS: u64 = 86_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct FaucetEntry {
    pub balance_pradha: u64,
    pub last_claim_day: u64,
    pub total_claimed_pradha: u64,
    /// Reward yang benar-benar diberikan pada klaim hari `last_claim_day`
    /// (pradha). Dinamis: bisa menyusut saat peserta banyak.
    #[serde(default)]
    pub last_claim_reward_pradha: u64,
    /// Detik Unix penarikan on-chain terakhir (0 bila belum pernah).
    #[serde(default)]
    pub last_withdraw_secs: u64,
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
    /// Jumlah klaim berhasil pada `claim_day`.
    #[serde(default)]
    claimants_today: u64,
    /// Jumlah klaim pada hari aktif terakhir sebelum `claim_day`.
    ///
    /// Inilah pengukur permintaan yang dipakai untuk membagi anggaran harian.
    /// Memakai angka hari sebelumnya (bukan maksimum sepanjang masa) membuat
    /// reward pulih sendiri ketika peserta berkurang.
    #[serde(default)]
    last_day_claimants: u64,
    /// Total fee on-chain yang sudah dibayar reservoir pada `fee_day`.
    #[serde(default)]
    fees_today_pradha: u64,
    /// Hari Unix yang sedang dihitung untuk `fees_today_pradha`.
    #[serde(default)]
    fee_day: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimError {
    AlreadyClaimed,
    DailyCapReached,
    ReservoirFull,
    InvalidAddress,
    EmptyAddress,
    TooManyAccounts,
}

impl std::fmt::Display for ClaimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ClaimError::AlreadyClaimed => "already claimed today",
            ClaimError::DailyCapReached => "daily faucet cap reached",
            ClaimError::ReservoirFull => "faucet liability cap reached",
            ClaimError::InvalidAddress => "address is not a valid Celenium address",
            ClaimError::EmptyAddress => "address is empty",
            ClaimError::TooManyAccounts => "device has too many accounts",
        };
        f.write_str(s)
    }
}

impl std::error::Error for ClaimError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WithdrawError {
    /// Alamat menarik lagi sebelum jeda antar-withdraw lewat.
    Cooldown { remaining_secs: u64 },
    /// Anggaran fee on-chain hari ini sudah habis.
    FeeBudgetExhausted { used_pradha: u64, budget_pradha: u64 },
}

impl std::fmt::Display for WithdrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WithdrawError::Cooldown { remaining_secs } => {
                write!(f, "please wait {remaining_secs} seconds before withdrawing again")
            }
            WithdrawError::FeeBudgetExhausted { used_pradha, budget_pradha } => write!(
                f,
                "daily faucet fee budget exhausted ({used_pradha}/{budget_pradha} pradha); try again after 00:00 UTC"
            ),
        }
    }
}

impl std::error::Error for WithdrawError {}

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
            // Hari berganti: jumlah klaim hari yang baru selesai menjadi
            // pengukur permintaan untuk hari ini. Menyimpannya (bukan
            // mempertahankan maksimum sepanjang masa) membuat reward pulih
            // sendiri ketika peserta berkurang.
            if self.claim_day != 0 {
                self.last_day_claimants = self.claimants_today;
            }
            self.claim_day = day;
            self.claimed_today = 0;
            self.claimants_today = 0;
        }
    }

    /// Jumlah klaim berhasil yang sudah tercatat pada hari ini.
    pub fn claimants_today(&self) -> u64 {
        if Self::current_day() == self.claim_day {
            self.claimants_today
        } else {
            0
        }
    }

    /// Perkiraan jumlah klaim hari ini.
    ///
    /// Maksimum antara klaim yang sudah masuk dan jumlah klaim hari sebelumnya,
    /// dengan lantai 1. Angka hari sebelumnya adalah basis pembagian, sehingga
    /// reward menyusut **sebelum** anggaran habis ketika peserta bertambah.
    /// Angka hari ini ikut dimasukkan supaya lonjakan permintaan langsung
    /// memotong reward, bukan baru terasa setelah anggaran terkuras.
    pub fn expected_claimants_today(&self) -> u64 {
        let today = self.claimants_today();
        today.max(self.last_day_claimants).max(1)
    }

    /// Reward untuk klaim **berikutnya** hari ini, dalam pradha.
    ///
    /// Aturan bagi: `sisa_anggaran ÷ sisa_slot_yang_diperkirakan`, dibatasi
    /// `FAUCET_MAX_REWARD_PRADHA`. Slot yang diperkirakan adalah jumlah peserta
    /// hari ini dikurangi yang sudah dilayani, jadi reward **menyusut proporsional
    /// saat peserta bertambah** — inilah yang membuat 1000 pradha sebuah plafon,
    /// bukan janji.
    ///
    /// Invarian: `sisa_anggaran >= reward × sisa_slot` tetap berlaku, sehingga
    /// total pembayaran sehari tidak pernah melewati
    /// [`crate::emission_policy::daily_cap_pradha`] berapa pun jumlah peserta —
    /// termasuk 2 miliar perangkat. Batasnya adalah **jumlah klaim terlayani**,
    /// bukan nilai per klaim.
    ///
    /// Kalau pembagiannya rounds ke bawah nol (anggaran lebih kecil daripada
    /// jumlah peserta), reward ditahan di `FAUCET_MIN_REWARD_PRADHA` — satuan
    /// terkecil yang masih bisa dicatat di ledger.
    pub fn reward_pradha_today(&self) -> u64 {
        let month = crate::emission_policy::month_since_genesis(Self::now_secs());
        let budget = crate::emission_policy::daily_cap_pradha(month);
        let today = Self::current_day();
        let used = if today == self.claim_day { self.claimed_today } else { 0 };
        let claimants = if today == self.claim_day { self.claimants_today } else { 0 };
        let remaining = budget.saturating_sub(used);
        if remaining == 0 {
            return 0;
        }
        let slots_left = self.expected_claimants_today().saturating_sub(claimants).max(1);
        let share = remaining / slots_left;
        if share == 0 {
            FAUCET_MIN_REWARD_PRADHA
        } else {
            share.min(FAUCET_MAX_REWARD_PRADHA)
        }
    }

    /// Klaim harian untuk `address`. Kembalikan saldo baru bila berhasil.
    pub fn claim(&mut self, address: &str, device_id: &str) -> Result<FaucetEntry, ClaimError> {
        if address.trim().is_empty() {
            return Err(ClaimError::EmptyAddress);
        }
        if celenium_addresses::Address::try_from(address.trim()).is_err() {
            return Err(ClaimError::InvalidAddress);
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
        // Cap dinamis dari kebijakan emisi (lihat `emission_policy`): floor dan
        // cap harian menyusut mengikuti sisa emisi, sehingga kewajiban ledger
        // tidak pernah menumpuk melebihi dana yang mungkin datang.
        //
        // Plafon kewajiban memakai **bucket klaim**, bukan floor penuh: porsi
        // `MINER_SUBSIDY_SHARE` dari reservoir sudah dialokasikan untuk membiayai
        // miner Android. Memakai floor penuh akan menjanjikan dana yang bukan
        // milik klaim, dan subsidinya baru ketahuan kurang saat penarikan gagal.
        let now = Self::now_secs();
        let month = crate::emission_policy::month_since_genesis(now);
        let outstanding_cap =
            crate::emission_policy::claim_bucket_pradha(month).min(FAUCET_MAX_OUTSTANDING_PRADHA);
        let daily_cap = crate::emission_policy::daily_cap_pradha(month);
        // Reward yang berlaku untuk klaim ini, dihitung dari sisa anggaran dan
        // sisa slot yang diperkirakan. NIlainya bisa turun sampai 1 pradha saat
        // peserta sangat banyak — lihat `reward_pradha_today`.
        let reward = self.reward_pradha_today();
        if reward == 0 {
            return Err(ClaimError::DailyCapReached);
        }
        // Cap kewajiban total: tidak menerbitkan klaim baru jika seluruh saldo
        // tercatat (termasuk klaim ini) akan melampaui batas reservoir. Diperiksa
        // sebelum `entry` dipinjam, karena butuh peminjaman immutable `self`.
        if self.total_outstanding().saturating_add(reward) > outstanding_cap {
            return Err(ClaimError::ReservoirFull);
        }
        let entry = self.ledger.entry(address.to_string()).or_default();
        if entry.last_claim_day == day {
            return Err(ClaimError::AlreadyClaimed);
        }
        if self.claimed_today.saturating_add(reward) > daily_cap {
            return Err(ClaimError::DailyCapReached);
        }
        entry.balance_pradha = entry.balance_pradha.saturating_add(reward);
        entry.total_claimed_pradha = entry.total_claimed_pradha.saturating_add(reward);
        entry.last_claim_day = day;
        entry.last_claim_reward_pradha = reward;
        self.claimed_today = self.claimed_today.saturating_add(reward);
        self.claimants_today = self.claimants_today.saturating_add(1);
        Ok(*entry)
    }

    /// Saldo tercatat untuk `address`, bila ada.
    pub fn balance(&self, address: &str) -> Option<&FaucetEntry> {
        self.ledger.get(address)
    }

    /// Reward terakhir yang diberikan ke `address` pada hari ini, 0 bila belum klaim.
    ///
    /// Dipakai API untuk melaporkan jumlah yang benar-benar diterima (reward
    /// dinamis, bukan konstanta).
    pub fn claim_reward_for(&self, address: &str) -> u64 {
        if Self::current_day() != self.claim_day {
            return 0;
        }
        self.ledger
            .get(address)
            .filter(|e| e.last_claim_day == self.claim_day)
            .map(|e| e.last_claim_reward_pradha)
            .unwrap_or(0)
    }

    fn now_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn roll_fee_day(&mut self) {
        let day = Self::current_day();
        if day != self.fee_day {
            self.fee_day = day;
            self.fees_today_pradha = 0;
        }
    }

    /// Periksa apakah `address` boleh menarik sekarang.
    ///
    /// Dua pengaman, keduanya menahan biaya on-chain yang dibayar reservoir:
    /// plafon fee harian global (dicek dulu: ini batas keras) dan jeda per
    /// alamat (satu penarikan per jam).
    pub fn check_withdraw(&self, address: &str) -> Result<(), WithdrawError> {
        let used = if self.fee_day == Self::current_day() { self.fees_today_pradha } else { 0 };
        if used >= FAUCET_DAILY_FEE_BUDGET_PRADHA {
            return Err(WithdrawError::FeeBudgetExhausted {
                used_pradha: used,
                budget_pradha: FAUCET_DAILY_FEE_BUDGET_PRADHA,
            });
        }
        let now = Self::now_secs();
        if let Some(e) = self.ledger.get(address)
            && e.last_withdraw_secs > 0
        {
            let elapsed = now.saturating_sub(e.last_withdraw_secs);
            if elapsed < FAUCET_WITHDRAW_COOLDOWN_SECS {
                return Err(WithdrawError::Cooldown {
                    remaining_secs: FAUCET_WITHDRAW_COOLDOWN_SECS.saturating_sub(elapsed),
                });
            }
        }
        Ok(())
    }

    /// Catat penarikan yang berhasil: debit saldo, set waktu, dan tambah pemakaian fee.
    pub fn record_withdraw(&mut self, address: &str, amount_pradha: u64, fee_pradha: u64) -> bool {
        self.roll_fee_day();
        let now = Self::now_secs();
        let Some(e) = self.ledger.get_mut(address) else {
            return false;
        };
        if e.balance_pradha < amount_pradha {
            return false;
        }
        e.balance_pradha -= amount_pradha;
        e.last_withdraw_secs = now;
        self.fees_today_pradha = self.fees_today_pradha.saturating_add(fee_pradha);
        true
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

    /// Cap harian dinamis saat ini (pradha), mengikuti kebijakan emisi.
    pub fn daily_cap_now(&self) -> u64 {
        let month = crate::emission_policy::month_since_genesis(Self::now_secs());
        crate::emission_policy::daily_cap_pradha(month)
    }

    /// Floor/kewajiban maksimum dinamis saat ini (pradha), dibatasi plafon keras.
    ///
    /// Memakai bucket klaim, bukan floor penuh — porsi miner Android bukan
    /// kewajiban faucet.
    pub fn outstanding_cap_now(&self) -> u64 {
        let month = crate::emission_policy::month_since_genesis(Self::now_secs());
        crate::emission_policy::claim_bucket_pradha(month).min(FAUCET_MAX_OUTSTANDING_PRADHA)
    }

    /// Fee on-chain yang sudah dibayar hari ini (0 bila sudah ganti hari).
    pub fn fees_today(&self) -> u64 {
        if Self::current_day() != self.fee_day { 0 } else { self.fees_today_pradha }
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

    const ADDR: &str = "cen69:qqh6qzvlczgp7nf7v7a0cwa9xqdskejse2575n7kwxa4vz778sarcqr5g90e6";

    #[test]
    fn claims_once_per_day() {
        let mut s = FaucetState::default();
        let day = FaucetState::current_day();
        s.ledger.insert(
            ADDR.to_string(),
            FaucetEntry {
                balance_pradha: 0,
                last_claim_day: day.saturating_sub(1),
                total_claimed_pradha: 0, last_claim_reward_pradha: 0, last_withdraw_secs: 0 },
        );

        assert_eq!(
            s.claim(ADDR, "").unwrap().balance_pradha,
            FAUCET_MAX_REWARD_PRADHA,
            "peserta tunggal mendapat plafon penuh"
        );
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
                total_claimed_pradha: 5000, last_claim_reward_pradha: 0, last_withdraw_secs: 0 },
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
                total_claimed_pradha: 1000, last_claim_reward_pradha: 0, last_withdraw_secs: 0 },
        );
        s.ledger.insert(
            "b".into(),
            FaucetEntry {
                balance_pradha: 2000,
                last_claim_day: 0,
                total_claimed_pradha: 2000, last_claim_reward_pradha: 0, last_withdraw_secs: 0 },
        );
        assert_eq!(s.total_outstanding(), 3000);
    }

    #[test]
    fn daily_cap_stops_issuance() {
        let mut s = FaucetState::default();
        s.claim_day = FaucetState::current_day();
        // Cap harian dinamis: isi tepat sampai cap, klaim berikutnya harus ditolak.
        s.claimed_today = s.daily_cap_now();
        assert!(s.daily_cap_now() > 0, "cap dinamis harus > 0 di fase emisi");
        assert_eq!(s.claim(ADDR, ""), Err(ClaimError::DailyCapReached));
    }

    #[test]
    fn device_cap_limits_accounts() {
        let mut s = FaucetState::default();
        // Daftarkan 5 address berbeda pada device yang sama.
        for i in 0..FAUCET_MAX_ACCOUNTS_PER_DEVICE {
            let a = [
                "cen69:qr7nlqwkawksc5h43xglnrr40w36a0lu4fr5fd7swqerxee02yddz6kq5cpxa",
                "cen69:qzczdkrh98g65gl7u2h0vcjxtszzlyk3rlykn0ejr3zkvq7dg247qk4slnyrz",
                "cen69:qr65k3j4yrnajp4kus7duvkwwp9m4wddqwpp307e029ek0g840hmw5qdrs424",
                "cen69:qzlwk2rtjvp5skpj7ywmk9e9qw7gq4hnpmxeu2d9jff7zwz278k55qxmdklnt",
                "cen69:qptdhd3arqvleucg3rtrswq9grpznk7cqnz3nv39467pr4xt7g2kkwx6fcz42",
            ][i];
            s.ledger.insert(
                a.to_string(),
                FaucetEntry { balance_pradha: 0, last_claim_day: 0, total_claimed_pradha: 0, last_claim_reward_pradha: 0, last_withdraw_secs: 0 },
            );
            s.devices.entry("dev-1".to_string()).or_default().insert(a.to_string());
        }
        // Address ke-6 ditolak.
        let sixth = "cen69:qqjc2n5jqed3z5s3nsk4cmeqf3zf2adt8l9my3jdn3kguk3pwle35ue4hkr67";
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
                total_claimed_pradha: 7000, last_claim_reward_pradha: 0, last_withdraw_secs: 0 },
        );
        let p = std::env::temp_dir().join("faucet_test_roundtrip.json");
        s.save(&p).unwrap();
        let loaded = FaucetState::load(&p).unwrap();
        assert_eq!(loaded.balance(ADDR).unwrap().balance_pradha, 7000);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn rejects_invalid_address() {
        let mut s = FaucetState::default();
        assert_eq!(s.claim("not-an-address", "dev-x"), Err(ClaimError::InvalidAddress));
        assert!(s.claim("cen69:qzsgej828lu39wkvqfq8d2hwnul9ru79g3ja4ulj4y5cer25reym2unh2sjmw", "dev-x").is_ok());
    }

    #[test]
    fn outstanding_cap_stops_issuance() {
        let mut s = FaucetState::default();
        let cap = s.outstanding_cap_now();
        s.ledger.insert(
            "cen69:qzsgej828lu39wkvqfq8d2hwnul9ru79g3ja4ulj4y5cer25reym2unh2sjmw".into(),
            FaucetEntry {
                balance_pradha: cap - FAUCET_MAX_REWARD_PRADHA + 1,
                last_claim_day: 0,
                total_claimed_pradha: cap - FAUCET_MAX_REWARD_PRADHA + 1,
                last_claim_reward_pradha: 0,
                last_withdraw_secs: 0 },
        );
        assert_eq!(s.claim(ADDR, ""), Err(ClaimError::ReservoirFull));
    }

    #[test]
    fn withdraw_cooldown_blocks_repeat() {
        let mut s = FaucetState::default();
        s.ledger.insert(
            ADDR.to_string(),
            FaucetEntry { balance_pradha: 1000, last_claim_day: 0, total_claimed_pradha: 1000, last_claim_reward_pradha: 0, last_withdraw_secs: 0 },
        );
        assert!(s.check_withdraw(ADDR).is_ok());
        s.record_withdraw(ADDR, 100, 25).then_some(()).expect("record should succeed");
        assert!(matches!(s.check_withdraw(ADDR), Err(WithdrawError::Cooldown { .. })));
        // Saldo terdebit, last_withdraw_secs terisi, fee tercatat.
        assert_eq!(s.balance(ADDR).unwrap().balance_pradha, 900);
        assert_eq!(s.balance(ADDR).unwrap().last_withdraw_secs, FaucetState::now_secs());
    }

    #[test]
    fn fee_budget_exhausts_after_cap() {
        let mut s = FaucetState::default();
        s.ledger.insert(
            ADDR.to_string(),
            FaucetEntry { balance_pradha: u64::MAX >> 1, last_claim_day: 0, total_claimed_pradha: 0, last_claim_reward_pradha: 0, last_withdraw_secs: 0 },
        );
        for _ in 0..(FAUCET_DAILY_FEE_BUDGET_PRADHA / 25) {
            s.record_withdraw(ADDR, 1, 25);
        }
        assert!(matches!(s.check_withdraw(ADDR), Err(WithdrawError::FeeBudgetExhausted { .. })));
    }

    /// Reward maximum 1000 pradha hanya berlaku saat peserta sedikit.
    /// Pada 2.025.000.000 peserta (seperempat penduduk bumi) yang memakai
    /// Android, reward harus menyusut proporsional — inilah yang membuat angka
    /// 1000 pradha sebuah *plafon*, bukan janji.
    #[test]
    fn reward_shrinks_as_participants_grow() {
        let mut s = FaucetState::default();
        s.claim_day = FaucetState::current_day();
        s.last_day_claimants = 0;
        assert_eq!(s.reward_pradha_today(), FAUCET_MAX_REWARD_PRADHA);

        // Cap inflow berkelanjutan ~1.078e8 pradha/hari, jadi plafon 1000 pradha
        // masih penuh sampai ~107.8 juta peserta/hari. Di atas ambang itu reward
        // harus menyusut proporsional agar total payout tidak melewati cap.
        s.last_day_claimants = 200_000_000;
        let r = s.reward_pradha_today();
        assert!(r < FAUCET_MAX_REWARD_PRADHA, "reward = {r}, harus menyusut");
        assert!(r >= FAUCET_MIN_REWARD_PRADHA, "reward = {r}");

        // 2,025 miliar peserta (seperempat penduduk bumi): reward = cap / N.
        // Dengan cap inflow bulan ini, ini jauh di bawah plafon tetapi (selama
        // cap >= N) masih > 1 pradha — artinya setiap peserta mendapat bagian
        // proporsional dan total payout tetap <= cap. "Lantai 1 pradha" hanya
        // tercapai ketika peserta melebihi sisa anggaran dalam satuan pradha.
        s.last_day_claimants = 2_025_000_000;
        let cap = s.daily_cap_now();
        let r = s.reward_pradha_today();
        let expected = (cap / 2_025_000_000)
            .max(FAUCET_MIN_REWARD_PRADHA)
            .min(FAUCET_MAX_REWARD_PRADHA);
        assert_eq!(r, expected, "reward harus proporsional: cap={cap}");
        assert!(r < FAUCET_MAX_REWARD_PRADHA);
        assert!(r >= FAUCET_MIN_REWARD_PRADHA);
    }

    /// Bukti struktural: berapa pun jumlah peserta, total payout satu hari
    /// tidak pernah melewati cap harian. Inilah yang menjaga saldo coin tidak
    /// "lenyap" ketika seperempat penduduk bumi ikut menambang.
    #[test]
    fn daily_payout_never_exceeds_cap_under_stress() {
        let mut s = FaucetState::default();
        s.claim_day = FaucetState::current_day();
        let cap = s.daily_cap_now();
        assert!(cap > 0, "cap harus > 0 di fase emisi");

        // Peserta melonjak: demand 2 miliar, tapi hanya sebagian kecil yang
        // sempat klaim sebelum anggaran habis.
        s.last_day_claimants = 2_025_000_000;
        let mut total = 0u64;
        for i in 0..50_000u64 {
            let addr = format!("cen69:qqh6qzvlczgp7nf7v7a0cwa9xqdskejse2575n7kwxa4vz778sarcqr5g9{}", i % 100);
            if s.claim(&addr, "").is_err() {
                break;
            }
            total = s.claimed_today();
            if total >= cap {
                break;
            }
        }
        assert!(total <= cap, "payout {total} > cap {cap}");
    }

    /// Setelah anggaran harian habis, klaim baru harus ditolak — bukan
    /// membayar dengan nilai sembarang.
    #[test]
    fn exhausted_budget_rejects_further_claims() {
        let mut s = FaucetState::default();
        s.claim_day = FaucetState::current_day();
        s.claimed_today = s.daily_cap_now();
        assert_eq!(s.reward_pradha_today(), 0);
        assert_eq!(s.claim(ADDR, ""), Err(ClaimError::DailyCapReached));
    }

    /// Reward yang dilaporkan API harus sama dengan yang benar-benar dibayar.
    #[test]
    fn reported_reward_matches_actual_payout() {
        let mut s = FaucetState::default();
        s.claim_day = FaucetState::current_day();
        s.last_day_claimants = 500_000;
        let before = s.reward_pradha_today();
        let entry = s.claim(ADDR, "").expect("klaim pertama harus berhasil");
        assert_eq!(entry.last_claim_reward_pradha, before);
        assert_eq!(s.claim_reward_for(ADDR), before);
    }
}