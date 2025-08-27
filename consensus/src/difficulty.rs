// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::difficult_to_target;
use anyhow::{bail, format_err, Ok, Result};
use starcoin_chain_api::ChainReader;
use starcoin_crypto::HashValue;
use starcoin_dag::consensusdb::schemadb::GhostdagStoreReader;
use starcoin_logger::prelude::*;
use starcoin_types::block::{Block, BlockHeader};
use starcoin_types::{U256, U512};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
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

    let mut blue_block_set = HashSet::new();
    // let mut selected_chain = Vec::new();

    selected_blocks.iter().try_for_each(|id| {
        let ghostdata = chain.get_dag().storage.ghost_dag_store.get_data(*id)?;

        blue_block_set.extend(
            ghostdata
                .mergeset_blues
                .iter()
                .map(|id| {
                    chain.get_block(*id)?.ok_or_else(|| {
                        format_err!(
                            "failed to get the block header when getting next work required"
                        )
                    })
                })
                .collect::<Result<Vec<Block>>>()?,
        );
        // selected_chain.push(chain.get_block(*id)?.ok_or_else(|| {
        //     format_err!("failed to get the block header when getting next work required")
        // })?);
        Ok(())
    })?;
    // selected_chain.reverse();

    let mut blue_block_in_order: BTreeMap<u64, Vec<BlockDiffInfo>> = BTreeMap::new();

    for blue_block in blue_block_set.iter() {
        blue_block_in_order
            .entry(blue_block.header().number())
            .or_default()
            .push(blue_block.try_into()?);
    }

    for b in blue_block_in_order.values_mut() {
        b.sort_by_key(|b| b.timestamp);
    }

    let next_block_time_target = epoch.block_time_target();
    info!(
        "next_block_time_target: {:?}, blue block count: {:?}, selected parent id: {:?}",
        next_block_time_target,
        blue_block_in_order.len(),
        current_header.id()
    );

    let target = get_next_target_helper(blue_block_in_order, next_block_time_target)?;

    debug!(
        "get_next_work_required current_number: {}, epoch: {:?}, target: {}",
        current_header.number(),
        epoch,
        target
    );
    Ok(target)
}

pub fn get_next_target_helper(
    blocks: BTreeMap<u64, Vec<BlockDiffInfo>>,
    time_plan: u64,
) -> Result<U256> {
    if blocks.is_empty() {
        bail!("block diff info is empty")
    }
    if blocks.len() == 1 {
        return Ok(blocks
            .iter()
            .next()
            .expect("block diff info is empty")
            .1
            .last()
            .unwrap()
            .target);
    }
    // let block_n = blocks.iter().map(|(_number, diff)| diff.len()).sum::<usize>() as u64;
    let mut block_n: u64 = 0;

    let mut total_target = U512::zero();
    for (_number, diff_infos) in blocks.iter() {
        for diff_info in diff_infos.iter() {
            total_target = total_target
                .checked_add(U512::from(&diff_info.target))
                .ok_or_else(|| format_err!("calculate total target overflow"))?;
        }
        block_n = block_n.saturating_add(diff_infos.len() as u64);
    }
    let avg_target: U512 = total_target
        .checked_div(U512::from(block_n))
        // .and_then(|avg_target| U256::try_from(&avg_target).ok())
        .ok_or_else(|| format_err!("calculate avg target overflow"))?;

    let mut avg_time = match block_n.cmp(&2) {
        Ordering::Less => {
            unreachable!()
        }
        Ordering::Equal => blocks
            .iter()
            .last()
            .expect("block diff info is empty")
            .1
            .last()
            .unwrap()
            .timestamp
            .saturating_sub(
                blocks
                    .iter()
                    .next()
                    .expect("block diff info is empty")
                    .1
                    .last()
                    .unwrap()
                    .timestamp,
            ),
        Ordering::Greater => {
            let mut blocks_in_same_number = vec![];
            for (_number, diff_infos) in blocks.iter() {
                blocks_in_same_number.push(diff_infos.last().unwrap().clone());
            }
            let total_v_block_time: u64 = blocks_in_same_number
                .last()
                .expect("cannot find last block")
                .timestamp
                .saturating_sub(
                    blocks_in_same_number
                        .first()
                        .expect("cannot find first block")
                        .timestamp,
                );

            let avg_time = total_v_block_time
                .checked_div(block_n)
                .ok_or_else(|| format_err!("calculate avg time overflow"))?;
            info!("[BlockProcess] total_v_block_time: {:?}, block_n: {:?}, avg_time: {:?}, avg_target: {:?}, time plan: {:?}", total_v_block_time, block_n, avg_time, avg_target, time_plan);
            avg_time
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
        let new_target = if new_target.checked_div(2.into()).unwrap() > avg_target {
            debug!("target increase too fast, limit to 2 times");
            avg_target.saturating_mul(2.into())
        } else if new_target < avg_target.checked_div(2.into()).unwrap() {
            debug!("target reduce too fast, limit to 2 times");
            avg_target.checked_div(2.into()).unwrap()
        } else {
            new_target
        };
        if let std::result::Result::Ok(new_target) = U256::try_from(&new_target) {
            new_target
        } else {
            warn!("target large than max value, set to 1_difficulty");
            U256::MAX
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

impl TryFrom<&Block> for BlockDiffInfo {
    type Error = anyhow::Error;
    fn try_from(block: &Block) -> Result<Self, Self::Error> {
        Ok(Self {
            timestamp: block.header().timestamp(),
            target: difficult_to_target(block.header().difficulty())?,
        })
    }
}
