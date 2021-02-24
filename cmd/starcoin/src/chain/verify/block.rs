// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use rand::Rng;
use scmd::{CommandAction, ExecContext};
use starcoin_consensus::difficulty::{get_next_target_helper, BlockDiffInfo};
use starcoin_consensus::{difficult_to_target, target_to_difficulty};
use starcoin_logger::prelude::*;
use std::cmp::min;
use std::collections::{HashMap, VecDeque};
use structopt::StructOpt;

/// Verify block.
#[derive(Debug, StructOpt)]
#[structopt(name = "block")]
pub struct BlockOpt {
    #[structopt(name = "number", long, short = "n", default_value = "0")]
    block_number: u64,
}

pub struct VerifyBlockCommand;

impl CommandAction for VerifyBlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = BlockOpt;
    type ReturnItem = String;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let block_number = opt.block_number;

        // time_target
        let epoch_info = client.get_epoch_info_by_number(block_number)?;
        let head = client.chain_info()?;
        let start = epoch_info.start_block_number();
        let last_epoch_time_target = if epoch_info.number() > 0 {
            client
                .get_epoch_info_by_number(start - 1)?
                .block_time_target()
        } else {
            epoch_info.block_time_target()
        };

        let end = min(epoch_info.end_block_number(), head.head.number.0);
        let block_time_target = epoch_info.block_time_target() * 6;

        let mut block_map = HashMap::new();
        let difficulty_window = epoch_info.block_difficulty_window();
        let load_start = if start > difficulty_window {
            start - difficulty_window
        } else {
            start
        };
        //load block
        let block_vec = client.chain_get_blocks_by_number(Some(end), end - load_start + 1)?;
        for block in block_vec {
            block_map.insert(block.header.number.0, block.clone());
        }

        let mut continue_large = 0u64;
        for index in start + 1..end {
            let block1 = block_map.get(&(index - 1)).unwrap();
            let block2 = block_map.get(&index).unwrap();
            let time_differ = block2.header.timestamp.0 - block1.header.timestamp.0;

            if time_differ > block_time_target {
                warn!(
                    "time larger than target:{:?} {:?} {:?}",
                    continue_large, index, time_differ
                );
                continue_large += 1;
                assert!(continue_large < 4u64);
            } else {
                continue_large = 0u64;
            }
            // assert!(time_differ <= block_time_target);
            info!("time verify ok: {:?}", index);
        }

        // difficulty
        let mut random = rand::thread_rng();
        let index: u64 = random.gen_range(start..end);
        let mut block_diff_vec = VecDeque::new();
        let min = if index > difficulty_window {
            index - difficulty_window
        } else {
            1
        };
        info!("verify difficulty : min: {:?}, index: {}", min, index);
        for i in min..index {
            let block = block_map.get(&i).unwrap();
            block_diff_vec.push_front(BlockDiffInfo::new(
                block.header.timestamp.0,
                difficult_to_target(block.header.difficulty),
            ));
        }
        if !block_diff_vec.is_empty() {
            let time_plan = if index == start {
                last_epoch_time_target
            } else {
                epoch_info.block_time_target()
            };
            let target = get_next_target_helper(Vec::from(block_diff_vec), time_plan).unwrap();
            let block = block_map.get(&index).unwrap();
            let difficulty = target_to_difficulty(target);
            assert_eq!(block.header.difficulty, difficulty);
            info!("difficulty verify ok: {:?}", index);
        } else {
            warn!("index err: {:?}, start: {}, end {:}", index, start, end);
        }

        //gas check
        let block = block_map.get(&index).unwrap();
        let block_gas = block.header.gas_used.0;
        let txn_vec = block.body.txn_hashes();
        let mut total_gas = 0u64;
        for txn in txn_vec {
            let gas = client
                .chain_get_transaction_info(txn)
                .unwrap()
                .unwrap()
                .gas_used
                .0;
            total_gas += gas;
        }
        assert_eq!(block_gas, total_gas);
        info!("gas verify ok: {:?}", index);

        Ok("verify ok!".parse()?)
    }
}
