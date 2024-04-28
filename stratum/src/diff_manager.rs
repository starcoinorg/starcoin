use crate::difficulty_to_target_hex;
use starcoin_logger::prelude::*;
use starcoin_types::U256;
pub const SHARE_SUBMIT_PERIOD: u64 = 2;
pub const INIT_HASH_RATE: u64 = 10000;
pub const MINI_UPDATE_PERIOD: u64 = 20;

pub struct DifficultyManager {
    pub timestamp_since_last_update: u64,
    pub submits_since_last_update: u32,
    pub hash_rate: u64,
    pub difficulty: U256,
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
        Self {
            timestamp_since_last_update: Self::current_timestamp(),
            submits_since_last_update: 0,
            hash_rate: INIT_HASH_RATE,
            difficulty: Self::get_difficulty_from_hashrate(INIT_HASH_RATE, SHARE_SUBMIT_PERIOD),
        }
    }

    pub fn find_seal(&mut self) {
        self.submits_since_last_update += 1;
    }

    pub fn try_update(&mut self, worker: String) -> bool {
        self.find_seal();
        let current_timestamp = Self::current_timestamp();

        let pass_time = current_timestamp - self.timestamp_since_last_update;
        if pass_time < MINI_UPDATE_PERIOD {
            return false;
        }

        if self.submits_since_last_update == 0 {
            self.hash_rate /= 2
        } else {
            // hash_rate = difficulty / avg_time = difficulty / (pass_time / submits_of_share)
            self.hash_rate =
                (self.difficulty * self.submits_since_last_update / pass_time).as_u64();
        }
        info!("Miner:{} hash rate is:{}", worker, self.hash_rate);
        self.timestamp_since_last_update = current_timestamp;
        self.difficulty = Self::get_difficulty_from_hashrate(self.hash_rate, SHARE_SUBMIT_PERIOD);
        self.submits_since_last_update = 0;
        true
    }

    fn get_difficulty_from_hashrate(hash_rate: u64, share_submit_period: u64) -> U256 {
        if hash_rate == 0 {
            return 1.into();
        }
        U256::from(hash_rate * share_submit_period)
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time went backwards")
            .as_secs()
    }
}
