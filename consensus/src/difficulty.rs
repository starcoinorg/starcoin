// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_types::{U256, U512};

use crate::{difficult_1_target, difficult_to_target, target_to_difficulty};
use anyhow::{bail, Result};
use logger::prelude::*;
use starcoin_traits::ChainReader;
use starcoin_types::block::Block;
use starcoin_vm_types::on_chain_config::EpochInfo;
use std::convert::TryFrom;

/// Get the target of next pow work
pub fn get_next_work_required(chain: &dyn ChainReader, epoch: &EpochInfo) -> Result<U256> {
    let current_header = chain.current_header();
    if current_header.number <= 1 {
        return Ok(difficult_to_target(current_header.difficulty));
    }
    let start_window_num = if current_header.number < epoch.block_difficulty_window() {
        1
    } else {
        current_header.number - epoch.block_difficulty_window() + 1
    };
    let blocks: Vec<BlockDiffInfo> = (start_window_num..current_header.number + 1)
        .rev()
        .filter(|&n| epoch.start_number() <= n && current_header.number <= epoch.end_number())
        .map(|n| chain.get_block_by_number(n))
        .filter_map(Result::ok)
        .filter_map(|x| x)
        .map(|b| b.into())
        .collect();
    get_next_target_helper(blocks, epoch.block_time_target())
}

pub fn get_next_target_helper(blocks: Vec<BlockDiffInfo>, time_plan: u64) -> Result<U256> {
    if blocks.len() < 1 {
        bail!("block diff info is empty")
    }
    if blocks.len() == 1 {
        return Ok(target_to_difficulty(blocks[0].target));
    }
    let mut avg_time: u64 = 0;
    let mut avg_target = U256::zero();
    let block_n = blocks.len() - 1;
    while latest_block_index < block_n {
        avg_time += blocks[latest_block_index].timestamp;
        avg_target += U512::from(&blocks[latest_block_index].target);
        latest_block_index += 1
    }
    avg_time = if block_n <= 1 {
        blocks[0].timestamp - blocks[1].timestamp
    } else {
        (block_n + 1) as u64 * blocks[0].timestamp - avg_time - blocks[block_n].timestamp
    };
    let avg_target = U256::try_from(&(avg_target / U512::from(block_n)))
        .expect("avg_target must be transfer ok");

    avg_time /= (block_n as u64) * ((block_n + 1) as u64) / 2;
    if avg_time == 0 {
        avg_time = 1
    }
    // new_target = avg_target * avg_time_used/time_plan
    // avoid the target increase or reduce too fast.
    let new_target = if let Some(new_target) = (avg_target / time_plan).checked_mul(avg_time.into())
    {
        if new_target / 2 > avg_target {
            debug!("target increase too fast, limit to 2 times");
            avg_target * 2
        } else if new_target < avg_target / 2 {
            debug!("target reduce too fast, limit to 2 times");
            avg_target / 2
        } else {
            new_target
        }
    } else {
        debug!("target large than max value, set to 1_difficulty");
        difficult_1_target()
    };
    debug!(
        "avg_time:{:?}s, time_plan:{:?}s, target: {:?}",
        avg_time, time_plan, new_target
    );
    Ok(new_target)
}

#[derive(Clone)]
pub struct BlockDiffInfo {
    pub timestamp: u64,
    pub target: U256,
}

impl From<Block> for BlockDiffInfo {
    fn from(block: Block) -> Self {
        Self {
            timestamp: block.header.timestamp,
            target: difficult_to_target(block.header.difficulty),
        }
    }
}
