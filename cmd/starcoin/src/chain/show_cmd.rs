// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state::CliState;
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_types::startup_info::ChainInfo;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "show")]
pub struct ShowOpt {}

pub struct ShowCommand;

impl CommandAction for ShowCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ShowOpt;
    type ReturnItem = ChainInfo;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<ChainInfo> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let chain_info = client
            .chain_head()?
            .ok_or(format_err!("get chain head error."))?;

        Ok(chain_info)
    }
}
