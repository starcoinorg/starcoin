// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "cancel")]
pub struct CancelOpt {}

pub struct CancelCommand;

impl CommandAction for CancelCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = CancelOpt;
    type ReturnItem = ();

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.sync_cancel()
    }
}
