// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::difficult_to_target;
use anyhow::{bail, format_err, Result};
use starcoin_chain_api::ChainReader;
use starcoin_logger::prelude::*;
use starcoin_types::block::BlockHeader;
use starcoin_types::{U256, U512};
use std::cmp::Ordering;
use std::convert::TryFrom;

/// Get the target of next pow work
pub fn get_next_work_required(chain: &dyn ChainReader) -> Result<U256> {
    let epoch = chain.epoch();
    let current_header = chain.current_header();
    if current_header.number() <= 1 {
        return Ok(difficult_to_target(current_header.difficulty()));
    }
    let start_window_num = if current_header.number() < epoch.block_difficulty_window() {
        0
    } else {
        current_header
            .number()
            .saturating_sub(epoch.block_difficulty_window())
            .checked_add(1)
            .ok_or_else(|| format_err!("block number overflow"))?
    };
    let blocks = (start_window_num
        ..current_header
            .number()
            .checked_add(1)
            .ok_or_else(|| format_err!("block number overflow"))?)
        .rev()
        .map(|n| {
            chain
                .get_header_by_number(n)?
                .ok_or_else(|| format_err!("Can not find header by number {}", n))
                .map(|header| header.into())
        })
        .collect::<Result<Vec<BlockDiffInfo>>>()?;
    if start_window_num != 0 {
        debug_assert!(
            blocks.len() == epoch.block_difficulty_window() as usize,
            "block difficulty count should eq block_difficulty_window"
        );
    }
    let target = get_next_target_helper(blocks, epoch.block_time_target())?;
    debug!(
        "get_next_work_required current_number: {}, epoch: {:?}, target: {}",
        current_header.number(),
        epoch,
        target
    );
    Ok(target)
}

pub fn get_next_target_helper(blocks: Vec<BlockDiffInfo>, time_plan: u64) -> Result<U256> {
    if blocks.is_empty() {
        bail!("block diff info is empty")
    }
    if blocks.len() == 1 {
        return Ok(blocks[0].target);
    }
    let block_n = blocks.len();

    let mut total_target = U512::zero();
    for diff_info in blocks.iter() {
        total_target = total_target
            .checked_add(U512::from(&diff_info.target))
            .ok_or_else(|| format_err!("calculate total target overflow"))?;
    }
    let avg_target: U256 = total_target
        .checked_div(U512::from(block_n))
        .and_then(|avg_target| U256::try_from(&avg_target).ok())
        .ok_or_else(|| format_err!("calculate avg target overflow"))?;

    let mut avg_time = match block_n.cmp(&2) {
        Ordering::Less => {
            unreachable!()
        }
        Ordering::Equal => blocks[0].timestamp.saturating_sub(blocks[1].timestamp),
        Ordering::Greater => {
            let latest_timestamp = blocks[0].timestamp;
            let mut total_v_block_time: u64 = 0;
            let mut v_blocks: usize = 0;
            for (idx, diff_info) in blocks.iter().enumerate() {
                if idx == 0 {
                    continue;
                }
                total_v_block_time = total_v_block_time
                    .saturating_add(latest_timestamp.saturating_sub(diff_info.timestamp));
                v_blocks = v_blocks.saturating_add(idx);
            }
            total_v_block_time
                .checked_div(v_blocks as u64)
                .ok_or_else(|| format_err!("calculate avg time overflow"))?
        }
    };

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
        warn!("target large than max value, set to 1_difficulty");
        U256::max_value()
    };
    debug!(
        "avg_time:{:?} mills, time_plan:{:?} mills, target: {:?}",
        avg_time, time_plan, new_target
    );
    Ok(new_target)
}

#[derive(Clone)]
pub struct BlockDiffInfo {
    pub timestamp: u64,
    pub target: U256,
}

impl BlockDiffInfo {
    pub fn new(timestamp: u64, target: U256) -> Self {
        Self { timestamp, target }
    }
}

impl From<BlockHeader> for BlockDiffInfo {
    fn from(block_header: BlockHeader) -> Self {
        Self {
            timestamp: block_header.timestamp(),
            target: difficult_to_target(block_header.difficulty()),
        }
    }
}
