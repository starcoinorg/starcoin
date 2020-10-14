// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageInitError {
    #[error("Storage check error {0:?}.")]
    StorageCheckError(Error),
}
