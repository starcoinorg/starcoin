// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub use anyhow::{anyhow, bail, format_err, Error, Result};
use starcoin_crypto::HashValue;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenesisError {
    #[error(
        "Genesis version mismatch expect: {expect:?}, real: {real:?}, please clean your data_dir."
    )]
    GenesisVersionMismatch { expect: HashValue, real: HashValue },
    #[error("Genesis load fail {0:?}, please check your data_dir.")]
    GenesisLoadFailure(Error),
    #[error("Genesis block not exist in {0}, please check your data_dir.")]
    GenesisNotExist(String),
}
