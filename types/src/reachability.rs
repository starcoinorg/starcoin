use crate::{blockhash::BlockHashes, interval::Interval};
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue as Hash;
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]
pub struct ReachabilityData {
    pub children: BlockHashes,
    pub parent: Hash,
    pub interval: Interval,
    pub height: u64,
    pub future_covering_set: BlockHashes,
}

impl ReachabilityData {
    pub fn new(parent: Hash, interval: Interval, height: u64) -> Self {
        Self {
            children: Arc::new(vec![]),
            parent,
            interval,
            height,
            future_covering_set: Arc::new(vec![]),
        }
    }
}
