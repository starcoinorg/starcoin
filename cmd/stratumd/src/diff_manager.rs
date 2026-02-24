use crate::difficulty_to_target_hex;
use starcoin_logger::prelude::*;
use starcoin_types::U256;
use std::time::{Duration, Instant};

pub const TARGET_SHARE_TIME_SECS: u64 = 10;
pub const MIN_UPDATE_PERIOD_SECS: u64 = 30;
pub const MIN_DIFFICULTY: u64 = 2_000;
pub const MAX_DIFFICULTY: u64 = 10_000_000_000;
pub const MAX_ADJUST_FACTOR_NUM: u64 = 2;
pub const MAX_ADJUST_FACTOR_DEN: u64 = 1;
pub const MIN_SAMPLES_PER_UPDATE: u32 = 3;
pub const EMA_ALPHA_NUM: u64 = 3;
pub const EMA_ALPHA_DEN: u64 = 10;
pub const DRIFT_LOWER_NUM: u64 = 7;
pub const DRIFT_UPPER_NUM: u64 = 13;

pub struct DifficultyManager {
    pub last_update: Instant,
    pub submits_since_last_update: u32,
    pub difficulty: U256,
    pub avg_share_time: f64,
    pub last_share: Instant,
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
        let initial_difficulty = MIN_DIFFICULTY.max(1);
        let now = Instant::now();
        Self {
            last_update: now,
            submits_since_last_update: 0,
            difficulty: U256::from(initial_difficulty),
            avg_share_time: TARGET_SHARE_TIME_SECS as f64,
            last_share: now,
            last_decay: now,
        }
    }

    pub fn find_seal(&mut self) {
        self.submits_since_last_update += 1;
        self.last_share = Instant::now();
    }

    pub fn try_update(&mut self, worker: String) -> bool {
        self.find_seal();
        let now = Instant::now();

        let pass_time = now.duration_since(self.last_update).as_secs();
        if pass_time < MIN_UPDATE_PERIOD_SECS {
            debug!(
                target: "stratum_server",
                "Miner:{} diff update skipped: pass_time {}s < {}s",
                worker, pass_time, MIN_UPDATE_PERIOD_SECS
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

        let avg_time = pass_time as f64 / self.submits_since_last_update as f64;
        let alpha = EMA_ALPHA_NUM as f64 / EMA_ALPHA_DEN as f64;
        self.avg_share_time = self.avg_share_time * (1.0 - alpha) + avg_time * alpha;

        let lower = TARGET_SHARE_TIME_SECS as f64 * (DRIFT_LOWER_NUM as f64 / 10.0);
        let upper = TARGET_SHARE_TIME_SECS as f64 * (DRIFT_UPPER_NUM as f64 / 10.0);
        if self.avg_share_time >= lower && self.avg_share_time <= upper {
            debug!(
                target: "stratum_server",
                "Miner:{} diff update skipped: avg_share_time {:.2}s in [{:.2}, {:.2}]",
                worker, self.avg_share_time, lower, upper
            );
            self.last_update = now;
            self.submits_since_last_update = 0;
            return false;
        }

        let mut new_diff = (self.difficulty.as_u64() as f64)
            * (TARGET_SHARE_TIME_SECS as f64 / self.avg_share_time);
        let max_up = self
            .difficulty
            .as_u64()
            .saturating_mul(MAX_ADJUST_FACTOR_NUM);
        let max_down = self
            .difficulty
            .as_u64()
            .saturating_div(MAX_ADJUST_FACTOR_NUM.max(MAX_ADJUST_FACTOR_DEN));
        if new_diff > max_up as f64 {
            new_diff = max_up as f64;
        } else if new_diff < max_down as f64 {
            new_diff = max_down as f64;
        }

        let clamped = new_diff
            .max(MIN_DIFFICULTY as f64)
            .min(MAX_DIFFICULTY as f64) as u64;

        self.difficulty = U256::from(clamped);
        info!(
            target: "stratum_server",
            "Miner:{} avg_share_time:{:.2}s difficulty:{}",
            worker, self.avg_share_time, self.difficulty
        );
        self.last_update = now;
        self.submits_since_last_update = 0;
        true
    }

    pub fn maybe_decay(&mut self, worker: &str) -> bool {
        let now = Instant::now();
        let decay_window = Duration::from_secs(TARGET_SHARE_TIME_SECS.saturating_mul(3));
        if now.duration_since(self.last_share) < decay_window {
            return false;
        }
        if now.duration_since(self.last_decay) < decay_window {
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
        self.last_decay = now;
        info!(
            target: "stratum_server",
            "Miner:{} no-share decay difficulty:{} -> {}",
            worker, current, new_diff
        );
        true
    }
}
