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
use std::collections::{BTreeMap, BTreeSet, HashSet};
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

    let mut blue_block_in_order: BTreeMap<u64, Vec<BlockDiffInfo2>> = BTreeMap::new();

    for blue_block in blue_block_set.iter() {
        blue_block_in_order
            .entry(blue_block.header().number())
            .or_insert_with(Vec::new)
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
    blocks: BTreeMap<u64, Vec<BlockDiffInfo2>>,
    time_plan: u64,
) -> Result<U256> {
    if blocks.is_empty() {
        bail!("block diff info is empty")
    }
    if blocks.len() == 1 {
        return Ok(blocks.first_key_value().unwrap().1.last().unwrap().target);
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
    let avg_target: U256 = total_target
        .checked_div(U512::from(block_n))
        .and_then(|avg_target| U256::try_from(&avg_target).ok())
        .ok_or_else(|| format_err!("calculate avg target overflow"))?;

    let mut avg_time = match block_n.cmp(&2) {
        Ordering::Less => {
            unreachable!()
        }
        Ordering::Equal => blocks
            .last_key_value()
            .unwrap()
            .1
            .last()
            .unwrap()
            .timestamp
            .saturating_sub(
                blocks
                    .first_key_value()
                    .unwrap()
                    .1
                    .last()
                    .unwrap()
                    .timestamp,
            ),
        Ordering::Greater => {
            let mut blocks_in_same_number = vec![];
            let mut v_blocks: usize = 0;
            for (_number, diff_infos) in blocks.iter() {
                blocks_in_same_number.push(diff_infos.last().unwrap().clone());
                v_blocks = v_blocks.saturating_add(diff_infos.len());
            }
            blocks_in_same_number.reverse();
            // let latest_timestamp = blocks_in_same_number.first().unwrap().timestamp;
            let mut total_v_block_time: u64 = blocks_in_same_number
                .first()
                .unwrap()
                .timestamp
                .saturating_sub(blocks_in_same_number.last().unwrap().timestamp);
            // for (idx, diff_info) in blocks_in_same_number.iter().enumerate() {
            //     if idx == 0 {
            //         continue;
            //     }
            //     total_v_block_time = total_v_block_time
            //         .saturating_add(latest_timestamp.saturating_sub(diff_info.timestamp));
            //     v_blocks = v_blocks.saturating_add(idx);
            // }

            // let total_v_block_time = blocks
            //     .first()
            //     .unwrap()
            //     .timestamp
            //     .saturating_sub(blocks.last().unwrap().timestamp);
            let total_transaction_time = blocks
                .iter()
                .flat_map(|(_number, diff)| {
                    diff.iter()
                        .map(|diff| diff.transaction_count)
                        .collect::<Vec<u64>>()
                })
                .sum::<u64>();
            total_v_block_time =
                total_v_block_time.saturating_sub(total_transaction_time.saturating_mul(2));

            let avg_time = total_v_block_time
                .checked_div(v_blocks as u64)
                .ok_or_else(|| format_err!("calculate avg time overflow"))?;
            info!("jacktest: total_v_block_time: {:?}, total_transaction_time: {:?}, v_blocks: {:?}, avg_time: {:?}, avg_target: {:?}, time plan: {:?}", total_v_block_time, total_transaction_time, v_blocks, avg_time, avg_target, time_plan);
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDiffInfo2 {
    pub timestamp: u64,
    pub target: U256,
    pub transaction_count: u64,
}

impl BlockDiffInfo2 {
    pub fn new(timestamp: u64, target: U256, transaction_count: u64) -> Self {
        Self {
            timestamp,
            target,
            transaction_count,
        }
    }
}

impl TryFrom<&Block> for BlockDiffInfo2 {
    type Error = anyhow::Error;
    fn try_from(block: &Block) -> Result<Self, Self::Error> {
        Ok(Self {
            timestamp: block.header().timestamp(),
            target: difficult_to_target(block.header().difficulty())?,
            transaction_count: block.body.transactions.len() as u64,
        })
    }
}
