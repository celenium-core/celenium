//! PPLNS (Pay Per Last N Shares) share tracker.
//!
//! Mencatat jendela share terbaru (bobot = difficulty-weighted "work") untuk
//! menghitung porsi tiap worker atas reward blok. Bobot adalah ukuran kerja,
//! bukan uang; konversi ke integer pradha dilakukan di `distribute`.

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
struct ShareEntry {
    worker: String,
    weight: f64,
}

#[derive(Debug, Clone)]
pub struct PplnsTracker {
    window: VecDeque<ShareEntry>,
    capacity: usize,
}

impl PplnsTracker {
    pub fn new(capacity: usize) -> Self {
        Self { window: VecDeque::new(), capacity: capacity.max(1) }
    }

    /// Catat share valid (bobot = hash_value difficulty-weighted). Share dengan
    /// bobot non-positif diabaikan (tidak merusak distribusi).
    pub fn record_share(&mut self, worker: &str, weight: f64) {
        if weight <= 0.0 || !weight.is_finite() {
            return;
        }
        self.window.push_back(ShareEntry { worker: worker.to_string(), weight });
        while self.window.len() > self.capacity {
            self.window.pop_front();
        }
    }

    fn worker_totals(&self) -> (HashMap<String, f64>, f64) {
        let mut totals: HashMap<String, f64> = HashMap::new();
        let mut total = 0.0;
        for e in &self.window {
            *totals.entry(e.worker.clone()).or_insert(0.0) += e.weight;
            total += e.weight;
        }
        (totals, total)
    }

    /// Porsi tiap worker (0..1) dalam jendela saat ini.
    pub fn worker_fractions(&self) -> HashMap<String, f64> {
        let (totals, total) = self.worker_totals();
        if total <= 0.0 {
            totals.into_iter().map(|(w, _)| (w, 0.0)).collect()
        } else {
            totals.into_iter().map(|(w, t)| (w, t / total)).collect()
        }
    }

    /// Distribusi reward blok (pradha) ke worker proporsional bobot. Pembulatan
    /// floor diterapkan per worker; sisa (dust) diberikan ke worker porsi terbesar.
    pub fn distribute(&self, reward_pradha: u64) -> HashMap<String, u64> {
        let (totals, total) = self.worker_totals();
        if total <= 0.0 || reward_pradha == 0 {
            return HashMap::new();
        }

        let mut result: HashMap<String, u64> = HashMap::with_capacity(totals.len());
        let mut sum: u64 = 0;
        for (worker, weight) in &totals {
            let frac = weight / total;
            let amt = ((reward_pradha as f64) * frac).floor() as u64;
            result.insert(worker.clone(), amt);
            sum = sum.saturating_add(amt);
        }

        let dust = reward_pradha.saturating_sub(sum);
        if dust > 0 {
            if let Some((w, _)) = totals.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal)) {
                if let Some(amt) = result.get_mut(w) {
                    *amt = amt.saturating_add(dust);
                }
            }
        }
        result
    }

    pub fn len(&self) -> usize {
        self.window.len()
    }

    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_worker_gets_entire_reward() {
        let mut t = PplnsTracker::new(100);
        t.record_share("w1", 1.0);
        let d = t.distribute(1000);
        assert_eq!(d.get("w1"), Some(&1000));
    }

    #[test]
    fn equal_weight_splits_evenly() {
        let mut t = PplnsTracker::new(100);
        t.record_share("a", 1.0);
        t.record_share("b", 1.0);
        let d = t.distribute(1000);
        assert_eq!(d.get("a"), Some(&500));
        assert_eq!(d.get("b"), Some(&500));
    }

    #[test]
    fn weighted_shares_are_proportional() {
        let mut t = PplnsTracker::new(100);
        // a total 3.0, b total 1.0 → a 75%, b 25%
        t.record_share("a", 1.0);
        t.record_share("a", 2.0);
        t.record_share("b", 1.0);
        let d = t.distribute(1000);
        assert_eq!(d.get("a"), Some(&750));
        assert_eq!(d.get("b"), Some(&250));
    }

    #[test]
    fn dust_is_assigned_to_largest_worker() {
        let mut t = PplnsTracker::new(100);
        t.record_share("a", 1.0);
        t.record_share("b", 1.0);
        t.record_share("b", 1.0);
        // a 1/3, b 2/3; reward 1000 → a 333, b 666 + dust 1 → b 667
        let d = t.distribute(1000);
        assert_eq!(d.get("a"), Some(&333));
        assert_eq!(d.get("b"), Some(&667));
        assert_eq!(d.values().sum::<u64>(), 1000);
    }

    #[test]
    fn window_evicts_oldest_shares() {
        let mut t = PplnsTracker::new(2);
        t.record_share("old", 10.0);
        t.record_share("a", 1.0);
        t.record_share("b", 1.0); // evicts "old"
        assert_eq!(t.len(), 2);
        let d = t.distribute(1000);
        assert_eq!(d.get("old"), None);
        assert_eq!(d.get("a"), Some(&500));
        assert_eq!(d.get("b"), Some(&500));
    }

    #[test]
    fn empty_tracker_distributes_nothing() {
        let t = PplnsTracker::new(10);
        assert_eq!(t.distribute(1000).len(), 0);
    }
}