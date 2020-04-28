// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_types::startup_info::ChainInfo;
use structopt::StructOpt;

/// List branches of current chain, first is master.
#[derive(Debug, StructOpt)]
#[structopt(name = "branches")]
pub struct BranchesOpt {}

pub struct BranchesCommand;

impl CommandAction for BranchesCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = BranchesOpt;
    type ReturnItem = Vec<ChainInfo>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.chain_branches()
    }
}
