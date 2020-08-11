// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_vm_types::on_chain_config::EpochInfo;
use structopt::StructOpt;

/// Get epoch info of master.
#[derive(Debug, StructOpt)]
#[structopt(name = "get_epoch_info_by_number")]
pub struct GetEpochInfoByNumberOpt {
    #[structopt(name = "number", long, short = "n", default_value = "0")]
    number: u64,
}

pub struct GetEpochInfoByNumberCommand;

impl CommandAction for GetEpochInfoByNumberCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetEpochInfoByNumberOpt;
    type ReturnItem = EpochInfo;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.get_epoch_info_by_number(ctx.opt().number)
    }
}
