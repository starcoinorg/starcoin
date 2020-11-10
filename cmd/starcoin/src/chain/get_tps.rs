// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use structopt::StructOpt;

/// Get tps of master.
#[derive(Debug, StructOpt)]
#[structopt(name = "get_tps")]
pub struct GetTPSOpt {
    #[structopt(name = "number", long, short = "n", default_value = "0")]
    number: u64,
}

pub struct GetTPSCommand;

impl CommandAction for GetTPSCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetTPSOpt;
    type ReturnItem = u64;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.get_tps(ctx.opt().number)
    }
}
