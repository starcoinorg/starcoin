use super::{GhostAdpter, ParentChoice};
use crate::BlockEvent;
use anyhow::Result;
use starcoin_types::U256;
use std::cmp::Ordering;

pub fn drive_harness<F>(
    ghost: &mut GhostAdpter,
    events: Vec<BlockEvent>,
    _is_adversary: F,
) -> Result<usize>
where
    F: Fn(usize) -> bool,
{
    // Two-phase scheduling: select parents ordered by header_time, commit ordered by arrival_time
    // Before each planning step flush every pending block whose arrival_time is already due
    #[derive(Clone)]
    struct PendingPlan {
        event: BlockEvent,
        parents: Vec<starcoin_crypto::HashValue>,
    }

    // Planning order: header_time ascending
    let mut by_start = events.clone();
    by_start.sort_by(|a, b| a.header_time.cmp(&b.header_time));
    // Commit order: arrival_time ascending
    let mut pending: Vec<PendingPlan> = Vec::new();
    let mut accepted = 0usize;
    let mut i = 0usize; // index over by_start

    // Helper: commit all pending plans whose arrival_time <= t
    let flush_until = |t: u64,
                       ghost: &mut GhostAdpter,
                       pending: &mut Vec<PendingPlan>,
                       accepted: &mut usize|
     -> Result<()> {
        // Stable sort so earlier arrival and lower miner id are committed first
        pending.sort_by(
            |a, b| match a.event.arrival_time.cmp(&b.event.arrival_time) {
                Ordering::Equal => a.event.miner_id.cmp(&b.event.miner_id),
                other => other,
            },
        );
        while !pending.is_empty() {
            if pending[0].event.arrival_time > t {
                break;
            }
            let pending_plan = pending.remove(0);
            let plan = ghost.plan_with_parents(pending_plan.parents.clone())?;
            let ev = pending_plan.event;
            let diff = U256::from(1u64);
            let choice = if _is_adversary(ev.miner_id) {
                ParentChoice::Subset(plan.selected_parents.clone())
            } else {
                ParentChoice::Honest
            };
            let _ = ghost.commit_with_parents(diff, &ev, plan, choice)?;
            *accepted += 1;
        }
        Ok(())
    };

    while i < by_start.len() {
        let start_t = by_start[i].header_time;
        // Commit tasks that have already arrived before planning a new one
        flush_until(start_t, ghost, &mut pending, &mut accepted)?;

        // Plan a template against the current DAG view
        let base = ghost.plan_next_block()?;
        pending.push(PendingPlan {
            event: by_start[i].clone(),
            parents: base.selected_parents.clone(),
        });

        i += 1;
    }

    // Flush any remaining plans
    if !pending.is_empty() {
        let max_arrival = pending
            .iter()
            .map(|p| p.event.arrival_time)
            .max()
            .unwrap_or(0);
        flush_until(max_arrival, ghost, &mut pending, &mut accepted)?;
    }

    Ok(accepted)
}
