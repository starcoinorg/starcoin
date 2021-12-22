// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::BlockInfoView;
use starcoin_types::block::BlockNumber;
use structopt::StructOpt;

/// Get block info by number
#[derive(Debug, StructOpt)]
#[structopt(name = "get-block-info", alias = "get_block_info")]
pub struct GetBlockInfoOpt {
    #[structopt(name = "number")]
    number: BlockNumber,
}

pub struct GetBlockInfoCommand;

impl CommandAction for GetBlockInfoCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetBlockInfoOpt;
    type ReturnItem = BlockInfoView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let block_info = client
            .chain_get_block_info_by_number(opt.number)?
            .ok_or_else(|| anyhow::format_err!("block_info of height {} not found", opt.number))?;
        Ok(block_info)
    }
}
