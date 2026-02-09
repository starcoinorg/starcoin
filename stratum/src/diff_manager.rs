use crate::difficulty_to_target_hex;
use starcoin_logger::prelude::*;
use starcoin_types::U256;

pub const TARGET_SHARE_TIME_SECS: u64 = 10;
pub const MIN_UPDATE_PERIOD_SECS: u64 = 10;
pub const MIN_DIFFICULTY: u64 = 64;
pub const MAX_DIFFICULTY: u64 = 10_000_000_000;
pub const MAX_ADJUST_FACTOR_NUM: u64 = 2;
pub const MAX_ADJUST_FACTOR_DEN: u64 = 1;
pub const MIN_SAMPLES_PER_UPDATE: u32 = 1;
pub const EMA_ALPHA_NUM: u64 = 3;
pub const EMA_ALPHA_DEN: u64 = 10;
pub const DRIFT_LOWER_NUM: u64 = 7;
pub const DRIFT_UPPER_NUM: u64 = 13;

pub struct DifficultyManager {
    pub timestamp_since_last_update: u64,
    pub submits_since_last_update: u32,
    pub difficulty: U256,
    pub avg_share_time: f64,
    pub last_share_ts: u64,
    pub last_decay_ts: u64,
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
        let now = Self::current_timestamp();
        Self {
            timestamp_since_last_update: now,
            submits_since_last_update: 0,
            difficulty: U256::from(initial_difficulty),
            avg_share_time: TARGET_SHARE_TIME_SECS as f64,
            last_share_ts: now,
            last_decay_ts: now,
        }
    }

    pub fn find_seal(&mut self) {
        self.submits_since_last_update += 1;
        self.last_share_ts = Self::current_timestamp();
    }

    pub fn try_update(&mut self, worker: String) -> bool {
        self.find_seal();
        let current_timestamp = Self::current_timestamp();

        let pass_time = current_timestamp - self.timestamp_since_last_update;
        if pass_time < MIN_UPDATE_PERIOD_SECS {
            return false;
        }

        if self.submits_since_last_update < MIN_SAMPLES_PER_UPDATE {
            return false;
        }

        let avg_time = pass_time as f64 / self.submits_since_last_update as f64;
        let alpha = EMA_ALPHA_NUM as f64 / EMA_ALPHA_DEN as f64;
        self.avg_share_time = self.avg_share_time * (1.0 - alpha) + avg_time * alpha;

        let lower = TARGET_SHARE_TIME_SECS as f64 * (DRIFT_LOWER_NUM as f64 / 10.0);
        let upper = TARGET_SHARE_TIME_SECS as f64 * (DRIFT_UPPER_NUM as f64 / 10.0);
        if self.avg_share_time >= lower && self.avg_share_time <= upper {
            self.timestamp_since_last_update = current_timestamp;
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
            "Miner:{} avg_share_time:{:.2}s difficulty:{}",
            worker, self.avg_share_time, self.difficulty
        );
        self.timestamp_since_last_update = current_timestamp;
        self.submits_since_last_update = 0;
        true
    }

    pub fn maybe_decay(&mut self, worker: &str) -> bool {
        let now = Self::current_timestamp();
        let decay_window = TARGET_SHARE_TIME_SECS.saturating_mul(3);
        if now.saturating_sub(self.last_share_ts) < decay_window {
            return false;
        }
        if now.saturating_sub(self.last_decay_ts) < decay_window {
            return false;
        }
        let current = self.difficulty.as_u64();
        let mut new_diff = current.saturating_div(2).max(MIN_DIFFICULTY);
        if new_diff == 0 {
            new_diff = MIN_DIFFICULTY;
        }
        if new_diff == current {
            self.last_decay_ts = now;
            return false;
        }
        self.difficulty = U256::from(new_diff);
        self.last_decay_ts = now;
        info!(
            "Miner:{} no-share decay difficulty:{} -> {}",
            worker, current, new_diff
        );
        true
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs()
    }
}
