use crate::blockhash::BlueWorkType;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue as Hash;
use std::cmp::Ordering;

#[derive(Eq, Clone, Debug, Serialize, Deserialize)]
pub struct SortableBlock {
    pub hash: Hash,
    pub blue_work: BlueWorkType,
}

impl SortableBlock {
    pub fn new(hash: Hash, blue_work: BlueWorkType) -> Self {
        Self { hash, blue_work }
    }
}

impl PartialEq for SortableBlock {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl PartialOrd for SortableBlock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SortableBlock {
    fn cmp(&self, other: &Self) -> Ordering {
        self.blue_work
            .cmp(&other.blue_work)
            .then_with(|| self.hash.cmp(&other.hash))
    }
}
