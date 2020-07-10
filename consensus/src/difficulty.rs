// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use types::U256;

use anyhow::{format_err, Result};
use logger::prelude::*;
use starcoin_state_api::{AccountStateReader, StateNodeStore};
use starcoin_statedb::ChainStateDB;
use starcoin_vm_types::{account_config::CORE_CODE_ADDRESS, on_chain_config::EpochResource};
use std::sync::Arc;
use traits::ChainReader;

pub fn difficult_1_target() -> U256 {
    U256::max_value()
}

// pub fn current_hash_rate(target: &[u8]) -> u64 {
//     // current_hash_rate = (difficult_1_target/target_current) * difficult_1_hash/block_per_esc
//     let target_u256: U256 = target.into();
//     (difficult_1_target() / target_u256).low_u64() / (BLOCK_TIME_SEC as u64)
// }

/// Get the target of next pow work
pub fn get_next_work_required(
    chain: &dyn ChainReader,
    store: Arc<dyn StateNodeStore>,
) -> Result<U256> {
    let mut current_header = chain.current_header();
    if current_header.number <= 1 {
        return Ok(difficult_to_target(current_header.difficulty));
    }

    let chain_state_reader = ChainStateDB::new(store, Some(current_header.state_root));
    let account_reader = AccountStateReader::new(&chain_state_reader);
    if let Some(epoch) = account_reader.get_resource::<EpochResource>(CORE_CODE_ADDRESS)? {
        let blocks = {
            let mut blocks: Vec<BlockDiffInfo> = vec![];
            let calculate_window = if current_header.number < epoch.window() {
                current_header.number
            } else {
                epoch.window()
            };
            blocks.push(BlockDiffInfo {
                timestamp: current_header.timestamp,
                target: difficult_to_target(current_header.difficulty),
            });
            for _ in 1..calculate_window {
                match chain.get_header(current_header.parent_hash)? {
                    Some(header) => {
                        // Skip genesis
                        if header.number == 0 {
                            break;
                        }
                        blocks.push(BlockDiffInfo {
                            timestamp: header.timestamp,
                            target: difficult_to_target(header.difficulty),
                        });
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
        let block_n = blocks.len() - 1;
        while latest_block_index < block_n {
            let solve_time =
                blocks[latest_block_index].timestamp - blocks[latest_block_index + 1].timestamp;
            avg_time += solve_time * (block_n - latest_block_index) as u64;
            debug!(
                "solve_time:{:?}, avg_time:{:?}, block_n:{:?}",
                solve_time, avg_time, block_n
            );
            avg_target = avg_target + blocks[latest_block_index].target / block_n.into();
            latest_block_index += 1
        }
        avg_time /= (block_n as u64) * ((block_n + 1) as u64) / 2;
        if avg_time == 0 {
            avg_time = 1
        }
        let time_plan = epoch.time_target();
        // new_target = avg_target * avg_time_used/time_plan
        // avoid the target increase or reduce too fast.
        let new_target = if let Some(new_target) =
            (avg_target / time_plan.into()).checked_mul(avg_time.into())
        {
            if new_target / 2.into() > avg_target {
                debug!("target increase too fast, limit to 2 times");
                avg_target * 2
            } else if new_target < avg_target / 2.into() {
                debug!("target reduce too fase, limit to 2 times");
                avg_target / 2.into()
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
    } else {
        Err(format_err!("Epoch is none."))
    }
}

pub fn target_to_difficulty(target: U256) -> U256 {
    difficult_1_target() / target
}

pub fn difficult_to_target(difficulty: U256) -> U256 {
    difficult_1_target() / difficulty
}

#[derive(Clone)]
pub struct BlockDiffInfo {
    pub timestamp: u64,
    pub target: U256,
}
