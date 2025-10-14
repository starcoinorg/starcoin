use super::{GhostAdpter, ParentChoice};
use crate::BlockEvent;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_types::U256;

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
    let mut committed: Vec<(u64, HashValue)> = vec![(0, genesis_id)];

    while index < events.len() {
        let current_time = events[index].arrival_time;
        let mut batch_end = index;
        while batch_end < events.len() && events[batch_end].arrival_time == current_time {
            batch_end += 1;
        }

        while index < batch_end {
            let ev = &events[index];
            let diff = U256::from(1u64);

            let delay = ev.network_delay;
            let visible_time = current_time.saturating_sub(delay);
            let visible_parent = committed
                .iter()
                .rev()
                .find(|(arr, _)| *arr <= visible_time)
                .map(|(_, hash)| *hash)
                .unwrap_or(genesis_id);

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
            committed.push((ev.arrival_time, block_id));
            accepted += 1;
            index += 1;
        }
    }

    Ok(accepted)
}
