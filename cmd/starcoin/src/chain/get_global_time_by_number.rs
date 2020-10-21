// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_vm_types::on_chain_resource::GlobalTimeOnChain;
use structopt::StructOpt;

/// Get global time of master.
#[derive(Debug, StructOpt)]
#[structopt(name = "get_global_time_by_number")]
pub struct GetGlobalTimeByNumberOpt {
    #[structopt(name = "number", long, short = "n", default_value = "0")]
    number: u64,
}

pub struct GetGlobalTimeByNumberCommand;

impl CommandAction for GetGlobalTimeByNumberCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetGlobalTimeByNumberOpt;
    type ReturnItem = GlobalTimeOnChain;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.get_global_time_by_number(ctx.opt().number)
    }
}
