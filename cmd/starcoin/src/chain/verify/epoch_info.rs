// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
// use starcoin_logger::prelude::*;
use std::cmp::min;
use structopt::StructOpt;

/// Verify epoch_info.
#[derive(Debug, StructOpt)]
#[structopt(name = "epoch")]
pub struct EpochOpt {
    #[structopt(name = "number", long, short = "b", default_value = "0")]
    block_number: u64,
}

pub struct VerifyEpochCommand;

impl CommandAction for VerifyEpochCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = EpochOpt;
    type ReturnItem = String;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let block_number = opt.block_number;

        let epoch_info = client.get_epoch_info_by_number(block_number)?;
        let head = client.chain_info()?;
        let start = epoch_info.start_block_number();
        let end = min(epoch_info.end_block_number(), head.head.number.0);

        //check uncles
        let uncles = epoch_info.uncles();
        let mut total_uncle = 0u64;
        for number in start..end {
            let block = client.chain_get_block_by_number(number)?.unwrap();
            total_uncle += block.uncles.len() as u64;
        }
        assert_eq!(uncles, total_uncle);

        Ok("verify ok!".parse()?)
    }
}
