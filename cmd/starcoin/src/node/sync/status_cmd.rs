// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::SyncStatusView;

#[derive(Debug, Parser, Default)]
#[clap(name = "status")]
pub struct StatusOpt {}

pub struct StatusCommand;

impl CommandAction for StatusCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = StatusOpt;
    type ReturnItem = SyncStatusView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.sync_status()
    }
}
