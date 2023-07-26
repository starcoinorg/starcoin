mod extensions;
pub mod inquirer;
pub mod reachability_service;
mod reindex;
pub mod relations_service;

#[cfg(test)]
mod tests;
mod tree;

use crate::consensusdb::prelude::StoreError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReachabilityError {
    #[error("data store error")]
    StoreError(#[from] StoreError),

    #[error("data overflow error")]
    DataOverflow(String),

    #[error("data inconsistency error")]
    DataInconsistency,

    #[error("query is inconsistent")]
    BadQuery,
}

impl ReachabilityError {
    pub fn is_key_not_found(&self) -> bool {
        matches!(self, ReachabilityError::StoreError(e) if matches!(e, StoreError::KeyNotFound(_)))
    }
}

pub type Result<T> = std::result::Result<T, ReachabilityError>;

pub trait ReachabilityResultExtensions<T> {
    /// Unwraps the error into `None` if the internal error is `StoreError::KeyNotFound` or panics otherwise
    fn unwrap_option(self) -> Option<T>;
}

impl<T> ReachabilityResultExtensions<T> for Result<T> {
    fn unwrap_option(self) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(err) if err.is_key_not_found() => None,
            Err(err) => panic!("Unexpected reachability error: {err:?}"),
        }
    }
}
