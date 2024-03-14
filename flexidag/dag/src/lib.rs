use consensusdb::prelude::StoreError;

pub mod blockdag;
pub mod consensusdb;
pub mod ghostdag;
pub mod reachability;
pub mod types;
mod tests;

pub fn process_key_already_error(result: Result<(), StoreError>) -> Result<(), StoreError> {
    if let Err(StoreError::KeyAlreadyExists(_)) = result {
        Result::Ok(())
    } else {
        result
    }
}
