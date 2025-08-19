// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::difficult_to_target;
use anyhow::{bail, format_err, Ok, Result};
use starcoin_chain_api::ChainReader;
use starcoin_crypto::HashValue;
use starcoin_dag::consensusdb::schemadb::GhostdagStoreReader;
use starcoin_logger::prelude::*;
use starcoin_types::block::BlockHeader;
use starcoin_types::{U256, U512};
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
    let selected_blocks = (start_window_num
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
            selected_blocks.len() == epoch.block_difficulty_window() as usize,
            "block difficulty count should eq block_difficulty_window"
        );
    }

    // Calculate time range from first and last selected parent's blues
    let first_id = selected_blocks
        .first()
        .ok_or_else(|| format_err!("selected_blocks is empty"))?;
    let last_id = selected_blocks
        .last()
        .ok_or_else(|| format_err!("selected_blocks is empty"))?;

    // Get min timestamp from first selected parent's blues
    let first_ghostdata = chain.dag().storage.ghost_dag_store.get_data(*first_id)?;
    if first_ghostdata.mergeset_blues.is_empty() {
        bail!("First ghostdata has no blue blocks");
    }

    let first_header = chain
        .get_header(first_ghostdata.mergeset_blues[0])?
        .ok_or_else(|| format_err!("failed to get the first block header"))?;
    let mut min_timestamp = first_header.timestamp();

    for blue_id in first_ghostdata.mergeset_blues.iter().skip(1) {
        let header = chain.get_header(*blue_id)?.ok_or_else(|| {
            format_err!("failed to get the block header when getting next work required")
        })?;
        min_timestamp = min_timestamp.min(header.timestamp());
    }

    // Get max timestamp from last selected parent's blues (reuse if same block)
    let last_ghostdata = if first_id == last_id {
        first_ghostdata
    } else {
        chain.dag().storage.ghost_dag_store.get_data(*last_id)?
    };
    if last_ghostdata.mergeset_blues.is_empty() {
        bail!("Last ghostdata has no blue blocks");
    }

    let last_header = chain
        .get_header(last_ghostdata.mergeset_blues[0])?
        .ok_or_else(|| format_err!("failed to get the last block header"))?;
    let mut max_timestamp = last_header.timestamp();

    for blue_id in last_ghostdata.mergeset_blues.iter().skip(1) {
        let header = chain.get_header(*blue_id)?.ok_or_else(|| {
            format_err!("failed to get the block header when getting next work required")
        })?;
        max_timestamp = max_timestamp.max(header.timestamp());
    }

    let time_used = max_timestamp.saturating_sub(min_timestamp);

    // Collect all blue blocks for target calculation
    let mut blue_blocks = Vec::new();
    for id in selected_blocks.iter() {
        let ghostdata = chain.dag().storage.ghost_dag_store.get_data(*id)?;

        for blue_id in ghostdata.mergeset_blues.iter() {
            let header = chain.get_header(*blue_id)?.ok_or_else(|| {
                format_err!("failed to get the block header when getting next work required")
            })?;
            blue_blocks.push(BlockDiffInfo::try_from(&header)?);
        }
    }

    let next_block_time_target = epoch.block_time_target();
    info!(
        "next_block_time_target: {:?}, blue block count: {:?}, time_used: {:?}, selected parent id: {:?}",
        next_block_time_target,
        blue_blocks.len(),
        time_used,
        current_header.id()
    );

    let target = get_next_target_helper(blue_blocks, time_used, next_block_time_target)?;

    debug!(
        "get_next_work_required current_number: {}, epoch: {:?}, target: {}",
        current_header.number(),
        epoch,
        target
    );
    Ok(target)
}

pub fn get_next_target_helper(
    blocks: Vec<BlockDiffInfo>,
    time_used: u64,
    time_plan: u64,
) -> Result<U256> {
    if blocks.is_empty() {
        bail!("block diff info is empty")
    }

    let block_n = blocks.len() as u64;

    if block_n == 1 {
        return Ok(blocks[0].target);
    }

    // Calculate average target
    let mut total_target = U512::zero();
    for block in blocks.iter() {
        total_target = total_target
            .checked_add(U512::from(&block.target))
            .ok_or_else(|| format_err!("calculate total target overflow"))?;
    }

    let avg_target: U256 = total_target
        .checked_div(U512::from(block_n))
        .and_then(|avg_target| U256::try_from(&avg_target).ok())
        .ok_or_else(|| format_err!("calculate avg target overflow"))?;

    // Calculate average time per block
    let mut avg_time = time_used
        .checked_div(block_n)
        .ok_or_else(|| format_err!("calculate avg time overflow"))?;

    info!(
        "[BlockProcess] time_used: {:?}, block_n: {:?}, avg_time: {:?}, avg_target: {:?}, time_plan: {:?}",
        time_used, block_n, avg_time, avg_target, time_plan
    );

    if avg_time == 0 {
        warn!(
            "Average time is 0, this should not happen! time_used: {}, block_n: {}. Setting to 1ms to avoid division by zero.",
            time_used, block_n
        );
        avg_time = 1;
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

impl TryFrom<&BlockHeader> for BlockDiffInfo {
    type Error = anyhow::Error;
    fn try_from(block: &BlockHeader) -> Result<Self, Self::Error> {
        Ok(Self {
            timestamp: block.timestamp(),
            target: difficult_to_target(block.difficulty())?,
        })
    }
}
