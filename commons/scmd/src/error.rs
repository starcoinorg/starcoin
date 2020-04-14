// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CmdError {
    #[error("Can not find command {cmd:?} \n {help:?})")]
    InvalidCommand { cmd: String, help: String },
    #[error("{help:?})")]
    NeedHelp { help: String },
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
