// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_types::sync_status::SyncStatus;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "status")]
pub struct StatusOpt {}

pub struct StatusCommand;

impl CommandAction for StatusCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = StatusOpt;
    type ReturnItem = SyncStatus;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.sync_status()
    }
}
