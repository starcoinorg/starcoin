// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::BlockView;
use starcoin_types::block::BlockNumber;

/// Get block by hash or number.
#[derive(Debug, Parser)]
#[clap(name = "get-height-blocks", alias = "get_height_blocks")]
pub struct GetHeightBlocksOpt {
    #[clap(name = "number")]
    number: BlockNumber,
}

pub struct GetHeightBlocksCommand;

impl CommandAction for GetHeightBlocksCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetHeightBlocksOpt;
    type ReturnItem = Vec<BlockView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let blocks = client.chain_get_height_blocks(opt.number)?;
        Ok(blocks)
    }
}
