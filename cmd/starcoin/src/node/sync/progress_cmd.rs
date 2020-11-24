// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_sync_api::SyncProgressReport;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "progress")]
pub struct ProgressOpt {}

pub struct ProgressCommand;

impl CommandAction for ProgressCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ProgressOpt;
    type ReturnItem = SyncProgressReport;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client
            .sync_progress()?
            .ok_or_else(|| format_err!("There are no running sync tasks."))
    }
}
