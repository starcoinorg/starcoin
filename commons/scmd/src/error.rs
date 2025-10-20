// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::error::ErrorKind;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CmdError {
    #[error(transparent)]
    ClapError(clap::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl CmdError {
    pub fn need_help(help: String) -> Self {
        Self::ClapError(clap::Error::raw(ErrorKind::InvalidSubcommand, help))
    }
}

impl From<clap::Error> for CmdError {
    fn from(clap_error: clap::Error) -> Self {
        Self::ClapError(clap_error)
    }
}
