use super::{GhostAdpter, ParentChoice};
use crate::BlockEvent;
use anyhow::Result;
use starcoin_types::U256;
use std::cmp::Ordering;
use std::time::{Duration, Instant};

#[derive(Default)]
struct HarnessProfiler {
    flush_time: Duration,
    plan_time: Duration,
    plan_with_parents_time: Duration,
    commit_time: Duration,
    flush_calls: usize,
    plan_calls: usize,
    plan_with_parents_calls: usize,
    commit_calls: usize,
}

impl HarnessProfiler {
    fn from_env() -> Option<Self> {
        if std::env::var("SIMNET_PROFILE").is_ok() {
            Some(Self::default())
        } else {
            None
        }
    }

    fn record_flush(&mut self, duration: Duration) {
        self.flush_time += duration;
        self.flush_calls += 1;
    }

    fn record_plan(&mut self, duration: Duration) {
        self.plan_time += duration;
        self.plan_calls += 1;
    }

    fn record_plan_with_parents(&mut self, duration: Duration) {
        self.plan_with_parents_time += duration;
        self.plan_with_parents_calls += 1;
    }

    fn record_commit(&mut self, duration: Duration) {
        self.commit_time += duration;
        self.commit_calls += 1;
    }

    fn report(&self) {
        let to_ms = |d: Duration| d.as_secs_f64() * 1000.0;
        println!(
            "[simnet::drive_harness] flush_total={:.2}ms (calls={}), plan_total={:.2}ms (calls={}, avg={:.2}ms), plan_with_parents_total={:.2}ms (calls={}, avg={:.2}ms), commit_total={:.2}ms (calls={}, avg={:.2}ms)",
            to_ms(self.flush_time),
            self.flush_calls,
            to_ms(self.plan_time),
            self.plan_calls,
            if self.plan_calls > 0 { to_ms(self.plan_time) / self.plan_calls as f64 } else { 0.0 },
            to_ms(self.plan_with_parents_time),
            self.plan_with_parents_calls,
            if self.plan_with_parents_calls > 0 {
                to_ms(self.plan_with_parents_time) / self.plan_with_parents_calls as f64
            } else {
                0.0
            },
            to_ms(self.commit_time),
            self.commit_calls,
            if self.commit_calls > 0 { to_ms(self.commit_time) / self.commit_calls as f64 } else { 0.0 }
        );
    }
}

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

    let mut profiler = HarnessProfiler::from_env();

    // Helper: commit all pending plans whose arrival_time <= t
    let flush_until = |t: u64,
                       ghost: &mut GhostAdpter,
                       pending: &mut Vec<PendingPlan>,
                       accepted: &mut usize,
                       profiler: &mut Option<HarnessProfiler>|
     -> Result<()> {
        let flush_start = Instant::now();
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
            let plan_parents_start = Instant::now();
            let plan = ghost.plan_with_parents(pending_plan.parents.clone())?;
            if let Some(prof) = profiler.as_mut() {
                prof.record_plan_with_parents(plan_parents_start.elapsed());
            }
            let ev = pending_plan.event;
            let diff = U256::from(1u64);
            let choice = if _is_adversary(ev.miner_id) {
                ParentChoice::Subset(plan.selected_parents.clone())
            } else {
                ParentChoice::Honest
            };
            let commit_start = Instant::now();
            let _ = ghost.commit_with_parents(diff, &ev, plan, choice)?;
            if let Some(prof) = profiler.as_mut() {
                prof.record_commit(commit_start.elapsed());
            }
            *accepted += 1;
        }
        if let Some(prof) = profiler.as_mut() {
            prof.record_flush(flush_start.elapsed());
        }
        Ok(())
    };

    while i < by_start.len() {
        let start_t = by_start[i].header_time;
        // Commit tasks that have already arrived before planning a new one
        flush_until(start_t, ghost, &mut pending, &mut accepted, &mut profiler)?;

        // Plan a template against the current DAG view
        let plan_start = Instant::now();
        let base = ghost.plan_next_block()?;
        if let Some(prof) = profiler.as_mut() {
            prof.record_plan(plan_start.elapsed());
        }
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
        flush_until(
            max_arrival,
            ghost,
            &mut pending,
            &mut accepted,
            &mut profiler,
        )?;
    }

    if let Some(prof) = profiler {
        prof.report();
    }

    Ok(accepted)
}
