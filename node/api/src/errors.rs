// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Error;
use starcoin_genesis::GenesisError;
use starcoin_storage::errors::StorageInitError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeStartError {
    #[error("Load config error, cause: {0:?}")]
    LoadConfigError(Error),
    #[error("Node start failed for genesis: {0:?}")]
    GenesisError(GenesisError),
    #[error("Node start failed for storage: {0:?}")]
    StorageInitError(StorageInitError),
    #[error("Node start failed, cause: {0:?}")]
    Other(Error),
}
