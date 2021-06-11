// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::BlockHeaderView;
use starcoin_types::block::BlockNumber;
use structopt::StructOpt;

/// List latest `count` blocks before `number`. if `number` is absent, use head block number.
#[derive(Debug, StructOpt)]
#[structopt(name = "list-block", alias = "list_block")]
pub struct ListBlockOpt {
    #[structopt(name = "number", long, short = "n")]
    number: Option<BlockNumber>,
    #[structopt(name = "count", long, short = "c", default_value = "10")]
    count: u64,
}

pub struct ListBlockCommand;

impl CommandAction for ListBlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ListBlockOpt;
    type ReturnItem = Vec<BlockHeaderView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let blocks = client.chain_get_blocks_by_number(opt.number, opt.count)?;
        let block_view = blocks.into_iter().map(|block| block.header).collect();
        Ok(block_view)
    }
}
