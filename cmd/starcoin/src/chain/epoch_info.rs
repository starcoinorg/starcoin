// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use structopt::StructOpt;
use starcoin_vm_types::on_chain_config::EpochInfo;

/// Epoch info of master.
#[derive(Debug, StructOpt)]
#[structopt(name = "epoch_info")]
pub struct EpochInfoOpt {}

pub struct EpochInfoCommand;

impl CommandAction for EpochInfoCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = EpochInfoOpt;
    type ReturnItem = EpochInfo;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.epoch_info()
    }
}
