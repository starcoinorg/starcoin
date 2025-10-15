use super::{GhostAdpter, ParentChoice};
use crate::BlockEvent;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_types::U256;
use std::collections::VecDeque;

pub fn drive_harness<F>(
    ghost: &mut GhostAdpter,
    mut events: Vec<BlockEvent>,
    _is_adversary: F,
) -> Result<usize>
where
    F: Fn(usize) -> bool,
{
    events.sort_by_key(|e| e.arrival_time);
    let mut accepted = 0;
    let mut index = 0;
    let genesis_id = ghost.genesis().id();

    let miner_count = events
        .iter()
        .map(|e| e.miner_id)
        .max()
        .map(|m| m + 1)
        .unwrap_or(0);
    if miner_count == 0 {
        return Ok(0);
    }

    let mut miner_delays = vec![0u64; miner_count];
    for ev in &events {
        miner_delays[ev.miner_id] = ev.network_delay;
    }

    let mut visibility: Vec<VecDeque<(u64, HashValue)>> = vec![VecDeque::new(); miner_count];
    let mut last_seen = vec![genesis_id; miner_count];

    while index < events.len() {
        let current_time = events[index].arrival_time;
        let mut batch_end = index;
        while batch_end < events.len() && events[batch_end].arrival_time == current_time {
            batch_end += 1;
        }

        for miner_id in 0..miner_count {
            while let Some(&(vis_time, hash)) = visibility[miner_id].front() {
                if vis_time <= current_time {
                    last_seen[miner_id] = hash;
                    visibility[miner_id].pop_front();
                } else {
                    break;
                }
            }
        }

        while index < batch_end {
            let ev = &events[index];
            let diff = U256::from(1u64);

            let visible_parent = last_seen.get(ev.miner_id).copied().unwrap_or(genesis_id);
            let plan = ghost.plan_with_parents(vec![visible_parent])?;

            if std::env::var("SIMNET_DEBUG").is_ok() {
                println!(
                    "event miner {} plan parents {:?}",
                    ev.miner_id, plan.selected_parents
                );
            }

            let choice = if _is_adversary(ev.miner_id) {
                ParentChoice::Subset(vec![visible_parent])
            } else {
                ParentChoice::Custom(vec![visible_parent])
            };

            let block_id = ghost.commit_with_parents(diff, ev, plan, choice)?;

            for (miner_id, delay) in miner_delays.iter().enumerate() {
                let vis_time = if miner_id == ev.miner_id {
                    current_time
                } else {
                    current_time.saturating_add(*delay)
                };
                visibility[miner_id].push_back((vis_time, block_id));
            }
            last_seen[ev.miner_id] = block_id;
            accepted += 1;
            index += 1;
        }
    }

    Ok(accepted)
}
