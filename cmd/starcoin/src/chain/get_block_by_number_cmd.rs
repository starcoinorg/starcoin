// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::BlockView;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "get_block_by_number")]
pub struct GetOpt {
    #[structopt(name = "number", long, default_value = "0")]
    number: usize,
}

pub struct GetBlockByNumberCommand;

impl CommandAction for GetBlockByNumberCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetOpt;
    type ReturnItem = BlockView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let block = client.chain_get_block_by_number(opt.number as u64)?;

        Ok(block.into())
    }
}
