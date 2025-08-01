use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Deserialize, Serialize, Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct TPS {
    total_txns: u64,
    duration: u64,
    tps: u64,
}

impl TPS {
    pub fn new(total_txns: u64, duration: u64, tps: u64) -> Self {
        Self {
            total_txns,
            duration,
            tps,
        }
    }
}
