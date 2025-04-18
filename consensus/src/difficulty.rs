// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::difficult_to_target;
use anyhow::{bail, format_err, Result};
use starcoin_chain_api::ChainReader;
use starcoin_crypto::HashValue;
use starcoin_dag::consensusdb::schemadb::GhostdagStoreReader;
use starcoin_logger::prelude::*;
use starcoin_types::block::BlockHeader;
use starcoin_types::{U256, U512};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::convert::TryFrom;

/// Get the target of next pow work
pub fn get_next_work_required(chain: &dyn ChainReader) -> Result<U256> {
    let epoch = chain.epoch();
    let current_header = chain.current_header();
    if current_header.number() <= 1 {
        return difficult_to_target(current_header.difficulty());
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
                .map(|header| header.id())
        })
        .collect::<Result<Vec<HashValue>>>()?;

    if start_window_num != 0 {
        debug_assert!(
            blocks.len() == epoch.block_difficulty_window() as usize,
            "block difficulty count should eq block_difficulty_window"
        );
    }

    let mut block_set = HashSet::new();
    block_set.extend(
        blocks
            .into_iter()
            .flat_map(|id| {
                chain
                    .get_dag()
                    .storage
                    .ghost_dag_store
                    .get_mergeset_blues(id)
                    .into_iter()
                    .flat_map(|blue_id| blue_id.as_ref().clone())
            })
            .map(|id| {
                chain.get_header(id)?.ok_or_else(|| {
                    format_err!("failed to get the block header when getting next work required")
                })
            })
            .collect::<Result<Vec<BlockHeader>>>()?,
    );

    let target = get_next_target_helper(
        block_set
            .into_iter()
            .map(|header| header.try_into())
            .collect::<Result<Vec<BlockDiffInfo>>>()?,
        2_00,
    )?;
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
    let new_target = if let Some(new_target) = avg_target
        .checked_div(time_plan.into())
        .and_then(|r| r.checked_mul(avg_time.into()))
    {
        // the divisor is `2` and never be `0`
        if new_target.checked_div(2.into()).unwrap() > avg_target {
            debug!("target increase too fast, limit to 2 times");
            avg_target.saturating_mul(2.into())
        } else if new_target < avg_target.checked_div(2.into()).unwrap() {
            debug!("target reduce too fast, limit to 2 times");
            avg_target.checked_div(2.into()).unwrap()
        } else {
            new_target
        }
    } else {
        warn!("target large than max value, set to 1_difficulty");
        U256::MAX
    };
    debug!(
        "avg_time:{:?} mills, time_plan:{:?} mills, target: {:?}",
        avg_time, time_plan, new_target
    );
    Ok(new_target)
}

#[derive(Debug, Clone)]
pub struct BlockDiffInfo {
    pub timestamp: u64,
    pub target: U256,
}

impl BlockDiffInfo {
    pub fn new(timestamp: u64, target: U256) -> Self {
        Self { timestamp, target }
    }
}

impl TryFrom<BlockHeader> for BlockDiffInfo {
    type Error = anyhow::Error;
    fn try_from(block_header: BlockHeader) -> Result<Self, Self::Error> {
        Ok(Self {
            timestamp: block_header.timestamp(),
            target: difficult_to_target(block_header.difficulty())?,
        })
    }
}
