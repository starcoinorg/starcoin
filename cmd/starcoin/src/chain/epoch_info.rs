// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_vm_types::on_chain_resource::EpochInfo;
use structopt::StructOpt;

/// Show epoch info.
#[derive(Debug, StructOpt)]
#[structopt(name = "epoch-info", alias = "epoch_info")]
pub struct EpochInfoOpt {
    /// The block number for get epoch info, if absent, show latest block epoch info.
    #[structopt(name = "block-number", long, short = "n")]
    block_number: Option<u64>,
}

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
        let opt = ctx.opt();
        if let Some(block_number) = opt.block_number {
            client.get_epoch_info_by_number(block_number)
        } else {
            client.epoch_info()
        }
    }
}
