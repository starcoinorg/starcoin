use consensusdb::prelude::StoreError;
use starcoin_crypto::HashValue;

mod block_depth;
pub mod blockdag;
pub mod consensusdb;
pub mod ghostdag;
pub mod level;
pub mod prune;
pub mod reachability;
pub mod service;
pub mod types;

pub fn process_key_already_error(result: Result<(), StoreError>) -> Result<(), StoreError> {
    if let Err(StoreError::KeyAlreadyExists(_)) = result {
        Result::Ok(())
    } else {
        result
    }
}

pub struct GetAbsentBlock {
    pub absent_id: Vec<HashValue>,
    pub exp: u64,
}

pub struct GetAbsentBlockResult {
    pub absent_blocks: Vec<HashValue>,
}
