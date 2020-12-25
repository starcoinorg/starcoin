// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

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
        CmdError::ClapError(clap::Error {
            message: help,
            kind: clap::ErrorKind::HelpDisplayed,
            info: None,
        })
    }
}

impl From<clap::Error> for CmdError {
    fn from(clap_error: clap::Error) -> Self {
        CmdError::ClapError(clap_error)
    }
}
