// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state::CliState;
use crate::view::BlockView;
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_types::block::Block;
use starcoin_types::startup_info::ChainInfo;
use std::ptr::hash;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "get_block")]
pub struct ShowOpt {
    #[structopt(short = "h")]
    hash: HashValue,
}

pub struct ShowCommand;

impl CommandAction for ShowCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ShowOpt;
    type ReturnItem = BlockView;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<Block> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let block = client
            .chain_get_block_by_hash(opt.hash)
            .ok_or(format_err!("get block by hash {:?} error.", opt.hash))?;

        Ok(block.into())
    }
}
