use super::interval::Interval;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue as Hash;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ReachabilityData {
    pub parent: Hash,
    pub interval: Interval,
    pub height: u64,
}

impl ReachabilityData {
    pub fn new(parent: Hash, interval: Interval, height: u64) -> Self {
        Self {
            parent,
            interval,
            height,
        }
    }
}
