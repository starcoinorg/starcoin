use super::{GhostAdpter, ParentChoice};
use crate::BlockEvent;
use anyhow::Result;
use starcoin_types::U256;
use std::cmp::Ordering;

pub fn drive_harness<F>(
    ghost: &mut GhostAdpter,
    mut events: Vec<BlockEvent>,
    _is_adversary: F,
) -> Result<usize>
where
    F: Fn(usize) -> bool,
{
    // 两阶段：按 header_time 计划父集，按 arrival_time 提交
    // 确保在计划时先提交所有 arrival_time <= 当前 header_time 的任务
    #[derive(Clone)]
    struct PendingPlan {
        event: BlockEvent,
        selected_parents: Vec<starcoin_crypto::HashValue>,
        ghostdata_planned: starcoin_dag::types::ghostdata::GhostdagData,
        pruning_point: starcoin_crypto::HashValue,
    }

    // 生产顺序：按 header_time
    let mut by_start = events.clone();
    by_start.sort_by(|a, b| a.header_time.cmp(&b.header_time));
    // 提交顺序：按 arrival_time
    let mut pending: Vec<PendingPlan> = Vec::new();
    let mut accepted = 0usize;
    let mut i = 0usize; // index over by_start

    // 小工具：提交到达时间 <= t 的所有计划
    let mut flush_until = |t: u64,
                           ghost: &mut GhostAdpter,
                           pending: &mut Vec<PendingPlan>,
                           accepted: &mut usize|
     -> Result<()> {
        // 稳定排序，先到先提交
        pending.sort_by(
            |a, b| match a.event.arrival_time.cmp(&b.event.arrival_time) {
                Ordering::Equal => a.event.miner_id.cmp(&b.event.miner_id),
                other => other,
            },
        );
        let mut j = 0;
        while j < pending.len() {
            if pending[j].event.arrival_time > t {
                break;
            }
            let plan = starcoin_dag::blockdag::MineNewDagBlockInfo {
                selected_parents: pending[j].selected_parents.clone(),
                ghostdata: pending[j].ghostdata_planned.clone(),
                pruning_point: pending[j].pruning_point,
            };
            let ev = pending[j].event.clone();
            let diff = U256::from(1u64);
            let choice = if _is_adversary(ev.miner_id) {
                ParentChoice::Subset(plan.selected_parents.clone())
            } else {
                ParentChoice::Honest
            };
            let _ = ghost.commit_with_parents(diff, &ev, plan, choice)?;
            *accepted += 1;
            pending.remove(j);
        }
        Ok(())
    };

    while i < by_start.len() {
        let start_t = by_start[i].header_time;
        // 先提交所有已经到达的任务
        flush_until(start_t, ghost, &mut pending, &mut accepted)?;

        // 以当前视图规划本次模板
        let base = ghost.plan_next_block()?;
        let plan = starcoin_dag::blockdag::MineNewDagBlockInfo {
            selected_parents: base.selected_parents.clone(),
            ghostdata: base.ghostdata.clone(),
            pruning_point: base.pruning_point,
        };

        if std::env::var("SIMNET_DEBUG").is_ok() {
            println!(
                "plan at start_t={} parents {:?}",
                start_t, plan.selected_parents
            );
        }

        pending.push(PendingPlan {
            event: by_start[i].clone(),
            selected_parents: plan.selected_parents,
            ghostdata_planned: plan.ghostdata,
            pruning_point: plan.pruning_point,
        });

        i += 1;
    }

    // 提交剩余
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
