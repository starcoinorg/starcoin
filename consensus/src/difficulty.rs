// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_types::U256;

use crate::{difficult_1_target, difficult_to_target};
use anyhow::Result;
use logger::prelude::*;
use starcoin_traits::ChainReader;
use starcoin_vm_types::on_chain_config::EpochInfo;

/// Get the target of next pow work
pub fn get_next_work_required(chain: &dyn ChainReader, epoch: &EpochInfo) -> Result<U256> {
    let mut current_header = chain.current_header();
    if current_header.number <= 1 {
        return Ok(difficult_to_target(current_header.difficulty));
    }
    let blocks = {
        let mut blocks: Vec<BlockDiffInfo> = vec![];

        loop {
            if epoch.block_difficulty_window() == 0
                || epoch.start_number() > current_header.number()
                || epoch.end_number() <= current_header.number()
            {
                break;
            }
            blocks.push(BlockDiffInfo {
                timestamp: current_header.timestamp,
                target: difficult_to_target(current_header.difficulty),
            });

            if (blocks.len() as u64) >= epoch.block_difficulty_window() {
                break;
            }

            match chain.get_header(current_header.parent_hash)? {
                Some(header) => {
                    // Skip genesis
                    if header.number == 0 {
                        break;
                    }
                    current_header = header;
                }
                None => {
                    anyhow::bail!("Invalid block, header not exist");
                }
            }
        }
        blocks
    };

    let mut avg_time: u64 = 0;
    let mut avg_target = U256::zero();
    let mut latest_block_index = 0;
    if blocks.len() <= 1 {
        return Ok(difficult_to_target(current_header.difficulty));
    }
    let block_n = blocks.len() - 1;
    while latest_block_index < block_n {
        let solve_time =
            blocks[latest_block_index].timestamp - blocks[latest_block_index + 1].timestamp;
        avg_time += solve_time * (block_n - latest_block_index) as u64;
        debug!(
            "solve_time:{:?}, avg_time:{:?}, block_n:{:?}",
            solve_time, avg_time, block_n
        );
        avg_target = avg_target + blocks[latest_block_index].target / block_n;
        latest_block_index += 1
    }
    avg_time /= (block_n as u64) * ((block_n + 1) as u64) / 2;
    if avg_time == 0 {
        avg_time = 1
    }
    let time_plan = epoch.block_time_target();
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
