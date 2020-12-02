// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::ChainInfoView;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "info")]
pub struct InfoOpt {}

pub struct InfoCommand;

impl CommandAction for InfoCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = InfoOpt;
    type ReturnItem = ChainInfoView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.chain_info()
    }
}
