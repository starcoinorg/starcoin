// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::chain::GetBlockOption;
use starcoin_rpc_api::types::BlockView;
use std::str::FromStr;
use starcoin_types::block::BlockNumber;

/// Get block by hash or number.
#[derive(Debug, Parser)]
#[clap(name = "get-block", alias = "get_block")]
pub struct GetNumBlocksOpt {
    #[clap(name = "number")]
    number: BlockNumber,

    #[clap(name = "contains-raw-block")]
    raw: bool,
}

pub struct GetNumBlocksCommand;

impl CommandAction for GetNumBlocksCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetNumBlocksOpt;
    type ReturnItem = Vec<BlockView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let blocks = client
                .chain_get_num_blocks(
                    number,
                    Some(GetBlockOption {
                        decode: true,
                        raw: opt.raw,
                    }),
                )?
                .ok_or_else(|| anyhow::format_err!("block of height {} not found", number))?;
        Ok(blocks)
    }
}
