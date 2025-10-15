use super::{GhostAdpter, ParentChoice};
use crate::BlockEvent;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_types::U256;
use std::collections::VecDeque;

#[derive(Clone)]
struct PendingVisibility {
    time: u64,
    block: HashValue,
    parents: Vec<HashValue>,
}

struct MinerView {
    tips: Vec<HashValue>,
}

impl MinerView {
    fn new(genesis: HashValue) -> Self {
        Self {
            tips: vec![genesis],
        }
    }

    fn observe_block(
        &mut self,
        ghost: &GhostAdpter,
        block: HashValue,
        parents: &[HashValue],
    ) -> Result<()> {
        if self.tips.is_empty() {
            self.tips.push(ghost.genesis_id());
        }

        self.tips.retain(|tip| !parents.contains(tip));
        self.tips.push(block);
        self.normalize(ghost)
    }

    fn normalize(&mut self, ghost: &GhostAdpter) -> Result<()> {
        if self.tips.is_empty() {
            self.tips.push(ghost.genesis_id());
            return Ok(());
        }

        self.tips.sort_unstable();
        self.tips.dedup();

        let mut filtered = Vec::with_capacity(self.tips.len());
        'outer: for i in 0..self.tips.len() {
            let tip_i = self.tips[i];
            for (j, tip_j) in self.tips.iter().enumerate() {
                if i == j {
                    continue;
                }
                if ghost.is_ancestor(tip_i, *tip_j)? {
                    continue 'outer;
                }
            }
            filtered.push(tip_i);
        }

        if filtered.is_empty() {
            filtered.push(ghost.genesis_id());
        }

        self.tips = filtered;
        Ok(())
    }

    fn parents_for_mining(&self, ghost: &GhostAdpter) -> Result<Vec<HashValue>> {
        if self.tips.is_empty() {
            return Ok(vec![ghost.genesis_id()]);
        }

        let max_parents = ghost.max_parents().max(1);
        if self.tips.len() <= max_parents {
            return Ok(self.tips.clone());
        }

        let mut ranked = Vec::with_capacity(self.tips.len());
        for tip in &self.tips {
            let (score, work) = ghost.ghost_stats(*tip)?.unwrap_or((0, U256::from(0u8)));
            ranked.push((*tip, score, work));
        }

        ranked.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| b.2.cmp(&a.2))
                .then_with(|| a.0.cmp(&b.0))
        });

        Ok(ranked
            .into_iter()
            .take(max_parents)
            .map(|(hash, _, _)| hash)
            .collect())
    }
}

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

    let mut visibility: Vec<VecDeque<PendingVisibility>> = vec![VecDeque::new(); miner_count];
    let mut miner_views: Vec<MinerView> = (0..miner_count)
        .map(|_| MinerView::new(genesis_id))
        .collect();

    while index < events.len() {
        let current_time = events[index].arrival_time;
        let mut batch_end = index;
        while batch_end < events.len() && events[batch_end].arrival_time == current_time {
            batch_end += 1;
        }

        for miner_id in 0..miner_count {
            while let Some(front) = visibility[miner_id].front() {
                if front.time > current_time {
                    break;
                }

                let pending = visibility[miner_id]
                    .pop_front()
                    .expect("front just checked to exist");
                miner_views[miner_id].observe_block(&*ghost, pending.block, &pending.parents)?;
            }
        }

        while index < batch_end {
            let ev = &events[index];
            let diff = U256::from(1u64);

            let parents = miner_views[ev.miner_id].parents_for_mining(&*ghost)?;
            let plan = ghost.plan_with_parents(parents.clone())?;

            if std::env::var("SIMNET_DEBUG").is_ok() {
                println!(
                    "event miner {} plan parents {:?}",
                    ev.miner_id, plan.selected_parents
                );
            }

            let choice = if _is_adversary(ev.miner_id) {
                ParentChoice::Subset(parents)
            } else {
                ParentChoice::Honest
            };

            let block_id = ghost.commit_with_parents(diff, ev, plan, choice)?;

            let parents_committed = ghost
                .records()
                .last()
                .map(|record| record.parents.clone())
                .unwrap_or_else(|| vec![genesis_id]);

            miner_views[ev.miner_id].observe_block(&*ghost, block_id, &parents_committed)?;

            for (miner_id, delay) in miner_delays.iter().enumerate() {
                if miner_id == ev.miner_id {
                    continue;
                }
                let vis_time = current_time.saturating_add(*delay);
                visibility[miner_id].push_back(PendingVisibility {
                    time: vis_time,
                    block: block_id,
                    parents: parents_committed.clone(),
                });
            }
            accepted += 1;
            index += 1;
        }
    }

    Ok(accepted)
}
