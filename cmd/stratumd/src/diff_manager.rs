use crate::difficulty_to_target_hex;
use starcoin_logger::prelude::*;
use starcoin_types::U256;
use std::time::{Duration, Instant};

pub const TARGET_SHARE_TIME_SECS: u64 = 3;
pub const MIN_UPDATE_PERIOD_SECS: u64 = 10;
pub const MIN_DIFFICULTY: u64 = 1;
pub const INITIAL_DIFFICULTY: u64 = 2_000;
pub const MAX_DIFFICULTY: u64 = 10_000_000_000;
pub const MAX_ADJUST_UP_NUM: u64 = 5;
pub const MAX_ADJUST_UP_DEN: u64 = 4;
pub const MAX_ADJUST_DOWN_NUM: u64 = 4;
pub const MAX_ADJUST_DOWN_DEN: u64 = 5;
pub const MIN_SAMPLES_PER_UPDATE: u32 = 6;
pub const EMA_ALPHA_NUM: u64 = 3;
pub const EMA_ALPHA_DEN: u64 = 10;
pub const STABLE_BAND_LOWER_NUM: u64 = 85;
pub const STABLE_BAND_UPPER_NUM: u64 = 115;

pub struct DifficultyManager {
    pub last_update: Instant,
    pub submits_since_last_update: u32,
    pub difficulty: U256,
    pub avg_share_time: f64,
    pub desired_diff_ema: f64,
    pub work_since_last_update: f64,
    pub last_share: Option<Instant>,
    pub last_decay: Instant,
}
impl Default for DifficultyManager {
    fn default() -> Self {
        Self::new()
    }
}
impl DifficultyManager {
    pub fn get_target(&self) -> String {
        difficulty_to_target_hex(self.difficulty)
    }

    pub fn new() -> Self {
        let initial_difficulty = INITIAL_DIFFICULTY.max(MIN_DIFFICULTY);
        let now = Instant::now();
        Self {
            last_update: now,
            submits_since_last_update: 0,
            difficulty: U256::from(initial_difficulty),
            avg_share_time: TARGET_SHARE_TIME_SECS as f64,
            desired_diff_ema: initial_difficulty as f64,
            work_since_last_update: 0.0,
            last_share: None,
            last_decay: now,
        }
    }

    fn reset_update_window(&mut self, now: Instant) {
        self.last_update = now;
        self.submits_since_last_update = 0;
        self.work_since_last_update = 0.0;
    }

    fn observe_share(&mut self, now: Instant, observed_diff: U256) {
        if let Some(last_share) = self.last_share {
            let share_interval = now
                .saturating_duration_since(last_share)
                .as_secs_f64()
                .max(0.001);
            let alpha = EMA_ALPHA_NUM as f64 / EMA_ALPHA_DEN as f64;
            self.avg_share_time = self.avg_share_time * (1.0 - alpha) + share_interval * alpha;
        }
        self.last_share = Some(now);
        self.submits_since_last_update += 1;
        self.work_since_last_update += observed_diff.as_u64().max(1) as f64;
    }

    pub fn try_update(&mut self, worker: &str, observed_diff: U256, network_diff: U256) -> bool {
        let now = Instant::now();
        self.try_update_at(worker, now, observed_diff, network_diff)
    }

    fn try_update_at(
        &mut self,
        worker: &str,
        now: Instant,
        observed_diff: U256,
        network_diff: U256,
    ) -> bool {
        self.observe_share(now, observed_diff);

        if now.saturating_duration_since(self.last_update)
            < Duration::from_secs(MIN_UPDATE_PERIOD_SECS)
        {
            debug!(
                target: "stratum_server",
                "Miner:{} diff update skipped: interval < {}s",
                worker, MIN_UPDATE_PERIOD_SECS
            );
            return false;
        }

        if self.submits_since_last_update < MIN_SAMPLES_PER_UPDATE {
            debug!(
                target: "stratum_server",
                "Miner:{} diff update skipped: samples {} < {}",
                worker, self.submits_since_last_update, MIN_SAMPLES_PER_UPDATE
            );
            return false;
        }

        let elapsed_secs = now
            .saturating_duration_since(self.last_update)
            .as_secs_f64()
            .max(0.001);
        let work_rate = self.work_since_last_update / elapsed_secs;
        if work_rate <= 0.0 {
            self.reset_update_window(now);
            return false;
        }

        let sampled_desired = (work_rate * TARGET_SHARE_TIME_SECS as f64)
            .max(MIN_DIFFICULTY as f64)
            .min(MAX_DIFFICULTY as f64);
        let alpha = EMA_ALPHA_NUM as f64 / EMA_ALPHA_DEN as f64;
        self.desired_diff_ema = self.desired_diff_ema * (1.0 - alpha) + sampled_desired * alpha;

        let current = self.difficulty.as_u64().max(1) as f64;
        let is_network_capped = observed_diff >= network_diff;
        let desired = if is_network_capped {
            self.desired_diff_ema
                .min(network_diff.as_u64().max(1) as f64)
        } else {
            self.desired_diff_ema
        };
        let ratio = desired / current.max(1.0);
        let stable_low = STABLE_BAND_LOWER_NUM as f64 / 100.0;
        let stable_high = STABLE_BAND_UPPER_NUM as f64 / 100.0;
        if ratio >= stable_low && ratio <= stable_high {
            debug!(
                target: "stratum_server",
                "Miner:{} diff update skipped: desired/current ratio {:.3} in [{:.2}, {:.2}]",
                worker, ratio, stable_low, stable_high
            );
            self.reset_update_window(now);
            return false;
        }

        let max_up = MAX_ADJUST_UP_NUM as f64 / MAX_ADJUST_UP_DEN as f64;
        let max_down = MAX_ADJUST_DOWN_NUM as f64 / MAX_ADJUST_DOWN_DEN as f64;
        let step = ratio.min(max_up).max(max_down);

        let mut clamped = (current * step)
            .round()
            .max(MIN_DIFFICULTY as f64)
            .min(MAX_DIFFICULTY as f64) as u64;
        if is_network_capped {
            clamped = clamped.min(network_diff.as_u64().max(1));
        }

        self.difficulty = U256::from(clamped);
        info!(
            target: "stratum_server",
            "Miner:{} avg_share_time:{:.2}s desired_diff:{:.0} difficulty:{} capped:{}",
            worker, self.avg_share_time, desired, self.difficulty, is_network_capped
        );
        self.reset_update_window(now);
        true
    }

    pub fn maybe_decay(&mut self, worker: &str) -> bool {
        let now = Instant::now();
        self.maybe_decay_at(worker, now)
    }

    fn maybe_decay_at(&mut self, worker: &str, now: Instant) -> bool {
        let Some(last_share) = self.last_share else {
            return false;
        };
        let decay_window = Duration::from_secs(TARGET_SHARE_TIME_SECS.saturating_mul(3));
        if now.saturating_duration_since(last_share) < decay_window {
            return false;
        }
        if now.saturating_duration_since(self.last_decay) < decay_window {
            return false;
        }
        let current = self.difficulty.as_u64();
        let mut new_diff = current.saturating_div(2).max(MIN_DIFFICULTY);
        if new_diff == 0 {
            new_diff = MIN_DIFFICULTY;
        }
        if new_diff == current {
            self.last_decay = now;
            return false;
        }
        self.difficulty = U256::from(new_diff);
        self.desired_diff_ema = self.desired_diff_ema.min(new_diff as f64);
        self.avg_share_time = TARGET_SHARE_TIME_SECS as f64;
        self.last_decay = now;
        self.reset_update_window(now);
        info!(
            target: "stratum_server",
            "Miner:{} no-share decay difficulty:{} -> {}",
            worker, current, new_diff
        );
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulty_increases_for_fast_shares() {
        let mut manager = DifficultyManager::new();
        let start = Instant::now();
        for i in 0..40 {
            let now = start + Duration::from_secs(i);
            let _ =
                manager.try_update_at("alice", now, U256::from(2_000u64), U256::from(100_000u64));
        }
        assert!(manager.difficulty.as_u64() > MIN_DIFFICULTY);
    }

    #[test]
    fn difficulty_decreases_for_slow_shares() {
        let mut manager = DifficultyManager::new();
        manager.difficulty = U256::from(20_000u64);
        manager.desired_diff_ema = 20_000.0;
        let start = Instant::now();
        for i in 0..8 {
            let now = start + Duration::from_secs(i * 8);
            let _ =
                manager.try_update_at("alice", now, U256::from(20_000u64), U256::from(100_000u64));
        }
        assert!(manager.difficulty.as_u64() < 20_000);
    }

    #[test]
    fn difficulty_decays_when_idle() {
        let mut manager = DifficultyManager::new();
        manager.difficulty = U256::from(32_000u64);
        manager.desired_diff_ema = 32_000.0;
        let start = Instant::now();
        let _ = manager.try_update_at(
            "alice",
            start,
            U256::from(32_000u64),
            U256::from(100_000u64),
        );
        let decayed = manager.maybe_decay_at(
            "alice",
            start + Duration::from_secs(TARGET_SHARE_TIME_SECS * 4),
        );
        assert!(decayed);
        assert_eq!(manager.difficulty.as_u64(), 16_000);
    }

    #[test]
    fn difficulty_stays_stable_inside_deadband() {
        let mut manager = DifficultyManager::new();
        manager.difficulty = U256::from(16_000u64);
        manager.desired_diff_ema = 16_000.0;
        let start = Instant::now();
        for i in 0..20 {
            let now = start + Duration::from_secs(i * 3);
            let _ =
                manager.try_update_at("alice", now, U256::from(16_000u64), U256::from(100_000u64));
        }
        assert_eq!(manager.difficulty.as_u64(), 16_000);
    }

    #[test]
    fn network_capped_worker_diff_does_not_stay_above_network() {
        let mut manager = DifficultyManager::new();
        manager.difficulty = U256::from(50_000u64);
        manager.desired_diff_ema = 50_000.0;
        let start = Instant::now();
        for i in 0..40 {
            let now = start + Duration::from_secs(i);
            let _ =
                manager.try_update_at("alice", now, U256::from(13_000u64), U256::from(13_000u64));
        }
        assert!(manager.difficulty.as_u64() <= 13_000);
    }
}
