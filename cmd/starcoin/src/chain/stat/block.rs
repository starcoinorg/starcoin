// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use serde::Serialize;
use structopt::StructOpt;

/// Get stat of gas for blocks.
#[derive(Debug, StructOpt)]
#[structopt(name = "block")]
pub struct BlockOpt {
    #[structopt(name = "begin", long, short = "b", default_value = "1")]
    begin_number: u64,
    #[structopt(name = "end", long, short = "e", default_value = "0")]
    end_number: u64,
}

#[derive(Serialize, Debug, Clone, Copy)]
pub struct BlockStatView {
    pub number: u64,
    pub gas_used: u64,
}
impl BlockStatView {
    fn new(number: u64, gas_used: u64) -> Self {
        Self { number, gas_used }
    }
}
pub struct StatBlockCommand;

impl CommandAction for StatBlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = BlockOpt;
    type ReturnItem = Vec<BlockStatView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let chain_info = client.chain_info().unwrap();
        let current_head_number = chain_info.head.number.0;
        let end_number = if opt.end_number >= 1 && opt.end_number < current_head_number {
            opt.end_number
        } else {
            current_head_number
        };
        // get block info
        let mut block_number = opt.begin_number;
        let mut vec_stat_block = vec![];
        while block_number < end_number {
            let block = client
                .chain_get_block_by_number(block_number)?
                .ok_or_else(|| anyhow::format_err!("block of height {} not found", block_number))?;
            let stat_view = BlockStatView::new(block.header.number.0, block.header.gas_used.0);
            println!("{:?}", stat_view);
            vec_stat_block.push(stat_view);
            block_number += 1;
        }
        Ok(vec_stat_block)
    }
}
