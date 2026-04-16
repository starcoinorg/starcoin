// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! DAG parameter sweep: iterate over (K, max_parents, block_interval, network_delay)
//! combinations under a simulated multi-miner DAG, collecting throughput and safety
//! metrics **without** starting a real node or executing any VM transactions.

use super::{harness::drive_harness, BlockRecord, GhostAdpter};
use crate::{BlkStream, DAGGenCfg, Miner};
use anyhow::Result;
use std::time::Instant;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// One point in the parameter grid.
#[derive(Debug, Clone)]
pub struct SweepParams {
    /// GhostDAG K (max uncles / blue anticone bound).
    pub k: u16,
    /// Maximum parent references per block.
    pub max_parents: usize,
    /// Mean block interval in milliseconds (global network average).
    pub block_interval_ms: u64,
    /// Per-miner network delay in milliseconds (uniform for all honest miners).
    pub network_delay_ms: u64,
    /// Simulation wall-time horizon in milliseconds.
    pub total_time_ms: u64,
    /// Number of honest miners participating.
    pub miner_count: usize,
}

impl Default for SweepParams {
    fn default() -> Self {
        Self {
            k: 16,
            max_parents: 10,
            block_interval_ms: 1000,
            network_delay_ms: 200,
            total_time_ms: 60_000,
            miner_count: 3,
        }
    }
}

/// Aggregated output for a single parameter combination.
#[derive(Debug, Clone)]
pub struct SweepResult {
    pub params: SweepParams,
    /// Total blocks produced by the simulation.
    pub total_blocks: usize,
    /// Blocks classified as **red** (not in the blue set of any ancestor).
    pub red_blocks: usize,
    /// red_blocks / total_blocks.
    pub red_rate: f64,
    /// Average number of parents per block.
    pub avg_parents: f64,
    /// Maximum DAG width observed (most tips alive at any single virtual-tip snapshot).
    pub max_dag_width: usize,
    /// Average confirmation depth: blue-score gap between a block and the virtual tip
    /// at the moment the block is committed.
    pub avg_confirm_depth: f64,
    /// Theoretical block throughput: blocks / second.
    pub blocks_per_second: f64,
    /// Wall-clock time (ms) spent inside `drive_harness` (GhostDAG computation).
    pub harness_wall_ms: f64,
    /// Average GhostDAG commit time per block (ms).
    pub avg_commit_ms: f64,
}

impl std::fmt::Display for SweepResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "K={:<3} parents={:<3} interval={:<5}ms delay={:<4}ms | \
             blocks={:<5} red_rate={:.4} avg_parents={:.2} max_width={:<3} \
             avg_confirm={:.1} blk/s={:.1} harness={:.0}ms avg_commit={:.3}ms",
            self.params.k,
            self.params.max_parents,
            self.params.block_interval_ms,
            self.params.network_delay_ms,
            self.total_blocks,
            self.red_rate,
            self.avg_parents,
            self.max_dag_width,
            self.avg_confirm_depth,
            self.blocks_per_second,
            self.harness_wall_ms,
            self.avg_commit_ms,
        )
    }
}

// ---------------------------------------------------------------------------
// CSV helpers
// ---------------------------------------------------------------------------

impl SweepResult {
    pub fn csv_header() -> &'static str {
        "k,max_parents,block_interval_ms,network_delay_ms,miner_count,\
         total_blocks,red_blocks,red_rate,avg_parents,max_dag_width,\
         avg_confirm_depth,blocks_per_second,harness_wall_ms,avg_commit_ms"
    }

    pub fn to_csv_row(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{:.6},{:.2},{},{:.2},{:.2},{:.2},{:.4}",
            self.params.k,
            self.params.max_parents,
            self.params.block_interval_ms,
            self.params.network_delay_ms,
            self.params.miner_count,
            self.total_blocks,
            self.red_blocks,
            self.red_rate,
            self.avg_parents,
            self.max_dag_width,
            self.avg_confirm_depth,
            self.blocks_per_second,
            self.harness_wall_ms,
            self.avg_commit_ms,
        )
    }
}

// ---------------------------------------------------------------------------
// Core sweep logic
// ---------------------------------------------------------------------------

/// Run a single parameter combination and return metrics.
///
/// `seed` controls the PRNG for reproducibility.
pub fn run_single(params: &SweepParams, seed: u64) -> Result<SweepResult> {
    let merge_depth: u64 = 3600; // fixed; not a sweep variable

    // Build miners with equal hash power
    let miners: Vec<Miner> = (0..params.miner_count)
        .map(|i| Miner {
            name: Box::leak(format!("miner_{i}").into_boxed_str()),
            hash_power_ratio: 1.0 / params.miner_count as f64,
            network_delay: params.network_delay_ms,
            is_attacker: false,
            hide_blocks_plan: vec![],
        })
        .collect();

    let cfg = DAGGenCfg {
        total_time: params.total_time_ms,
        block_interval: params.block_interval_ms,
        miners,
    };

    // Generate block event stream
    let mut stream = BlkStream::from_seed(cfg, seed);
    let events = stream.run();
    if events.is_empty() {
        return Ok(empty_result(params.clone()));
    }

    // Build DAG
    let mut ghost = GhostAdpter::new(params.k.into(), merge_depth, params.max_parents)?;
    let wall_start = Instant::now();
    let accepted = drive_harness(&mut ghost, events.clone(), |_| false)?;
    let harness_wall = wall_start.elapsed();

    // Collect metrics
    let records = ghost.records();
    let virtual_tips = ghost.virtual_tips();

    let total_blocks = accepted;
    let (red_blocks, avg_parents, max_dag_width, avg_confirm_depth) =
        compute_metrics(&ghost, records, virtual_tips)?;

    let sim_duration_secs = params.total_time_ms as f64 / 1000.0;
    let blocks_per_second = if sim_duration_secs > 0.0 {
        total_blocks as f64 / sim_duration_secs
    } else {
        0.0
    };

    let harness_wall_ms = harness_wall.as_secs_f64() * 1000.0;
    let avg_commit_ms = if total_blocks > 0 {
        harness_wall_ms / total_blocks as f64
    } else {
        0.0
    };

    Ok(SweepResult {
        params: params.clone(),
        total_blocks,
        red_blocks,
        red_rate: if total_blocks > 0 {
            red_blocks as f64 / total_blocks as f64
        } else {
            0.0
        },
        avg_parents,
        max_dag_width,
        avg_confirm_depth,
        blocks_per_second,
        harness_wall_ms,
        avg_commit_ms,
    })
}

/// Run each parameter combination in `grid` with the given seeds, averaging results
/// across seeds for stability.
pub fn run_sweep(grid: &[SweepParams], seeds: &[u64]) -> Result<Vec<SweepResult>> {
    let mut results = Vec::with_capacity(grid.len());
    for (idx, params) in grid.iter().enumerate() {
        let mut accum: Option<SweepResult> = None;
        for &seed in seeds {
            let r = run_single(params, seed)?;
            match accum.as_mut() {
                None => accum = Some(r),
                Some(a) => merge_into(a, &r),
            }
        }
        if let Some(mut avg) = accum {
            let n = seeds.len() as f64;
            avg.red_rate /= n;
            avg.avg_parents /= n;
            avg.avg_confirm_depth /= n;
            avg.blocks_per_second /= n;
            avg.harness_wall_ms /= n;
            avg.avg_commit_ms /= n;
            // total_blocks and red_blocks: use average (rounded)
            avg.total_blocks = (avg.total_blocks as f64 / n).round() as usize;
            avg.red_blocks = (avg.red_blocks as f64 / n).round() as usize;
            // max_dag_width: keep the max across seeds
            results.push(avg);
        }
        eprintln!(
            "[sweep] {}/{} done: {}",
            idx + 1,
            grid.len(),
            results.last().map(|r| r.to_string()).unwrap_or_default()
        );
    }
    Ok(results)
}

/// Build a default parameter grid covering the most important dimensions.
pub fn default_grid() -> Vec<SweepParams> {
    let k_values: Vec<u16> = vec![8, 16, 32];
    let interval_values: Vec<u64> = vec![500, 1000, 2000];
    let delay_values: Vec<u64> = vec![100, 200, 500];

    let mut grid = Vec::new();
    for &k in &k_values {
        for &interval in &interval_values {
            for &delay in &delay_values {
                let max_parents = (k as usize).min(10).max(5);
                grid.push(SweepParams {
                    k,
                    max_parents,
                    block_interval_ms: interval,
                    network_delay_ms: delay,
                    total_time_ms: 60_000,
                    miner_count: 3,
                    ..Default::default()
                });
            }
        }
    }
    grid
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn empty_result(params: SweepParams) -> SweepResult {
    SweepResult {
        params,
        total_blocks: 0,
        red_blocks: 0,
        red_rate: 0.0,
        avg_parents: 0.0,
        max_dag_width: 0,
        avg_confirm_depth: 0.0,
        blocks_per_second: 0.0,
        harness_wall_ms: 0.0,
        avg_commit_ms: 0.0,
    }
}

fn compute_metrics(
    ghost: &GhostAdpter,
    records: &[BlockRecord],
    virtual_tips: &[super::VirtualTip],
) -> Result<(usize, f64, usize, f64)> {
    let mut total_parents = 0usize;
    let mut confirm_depth_sum = 0u64;
    let mut confirm_depth_count = 0u64;
    let mut max_width = 0usize;

    // Collect ALL red blocks: a block B is red if it appears in ANY block's mergeset_reds.
    // Each block's ghostdata.mergeset_reds contains blocks from its merge set that exceed
    // the K-cluster blue anticone bound.
    let mut red_set = std::collections::HashSet::new();
    for record in records {
        if let Ok(Some(data)) = ghost.ghostdata(record.block_id) {
            for red_hash in data.mergeset_reds.iter() {
                red_set.insert(*red_hash);
            }
        }
    }
    // Also check the final virtual tip's merge set for any remaining reds
    if let Some(last_vt) = virtual_tips.last() {
        if let Ok(Some(data)) = ghost.ghostdata(last_vt.tip) {
            for red_hash in data.mergeset_reds.iter() {
                red_set.insert(*red_hash);
            }
        }
    }
    let red_count = red_set.len();

    for (i, record) in records.iter().enumerate() {
        total_parents += record.parents.len();

        // Confirmation depth: distance from virtual tip blue_score to this block's blue_score
        if i < virtual_tips.len() {
            let virtual_score = virtual_tips[i].blue_score;
            if let Ok(Some(data)) = ghost.ghostdata(record.block_id) {
                let depth = virtual_score.saturating_sub(data.blue_score);
                confirm_depth_sum += depth;
                confirm_depth_count += 1;
            }
        }
    }

    // Max DAG width: approximate from virtual tips snapshot
    for vt in virtual_tips {
        if let Ok(Some(data)) = ghost.ghostdata(vt.tip) {
            let width = data.mergeset_blues.len() + data.mergeset_reds.len();
            if width > max_width {
                max_width = width;
            }
        }
    }

    let avg_parents = if records.is_empty() {
        0.0
    } else {
        total_parents as f64 / records.len() as f64
    };

    let avg_confirm = if confirm_depth_count > 0 {
        confirm_depth_sum as f64 / confirm_depth_count as f64
    } else {
        0.0
    };

    Ok((red_count, avg_parents, max_width, avg_confirm))
}

fn merge_into(accum: &mut SweepResult, other: &SweepResult) {
    accum.total_blocks += other.total_blocks;
    accum.red_blocks += other.red_blocks;
    accum.red_rate += other.red_rate;
    accum.avg_parents += other.avg_parents;
    accum.avg_confirm_depth += other.avg_confirm_depth;
    accum.blocks_per_second += other.blocks_per_second;
    accum.harness_wall_ms += other.harness_wall_ms;
    accum.avg_commit_ms += other.avg_commit_ms;
    if other.max_dag_width > accum.max_dag_width {
        accum.max_dag_width = other.max_dag_width;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_run_produces_results() {
        let params = SweepParams {
            k: 16,
            max_parents: 10,
            block_interval_ms: 1000,
            network_delay_ms: 200,
            total_time_ms: 10_000,
            miner_count: 3,
        };
        let result = run_single(&params, 42).expect("run_single failed");
        assert!(result.total_blocks > 0, "should produce blocks");
        assert!(result.red_rate >= 0.0 && result.red_rate <= 1.0);
        assert!(result.blocks_per_second > 0.0);
        println!("{}", result);
    }

    #[test]
    fn sweep_small_grid() {
        let grid = vec![
            SweepParams {
                k: 8,
                max_parents: 5,
                block_interval_ms: 500,
                total_time_ms: 5_000,
                miner_count: 3,
                ..Default::default()
            },
            SweepParams {
                k: 16,
                max_parents: 10,
                block_interval_ms: 1000,
                total_time_ms: 5_000,
                miner_count: 3,
                ..Default::default()
            },
        ];
        let results = run_sweep(&grid, &[1, 2]).expect("sweep failed");
        assert_eq!(results.len(), 2);
        println!("{}", SweepResult::csv_header());
        for r in &results {
            println!("{}", r.to_csv_row());
        }
    }

    #[test]
    fn lower_interval_produces_more_blocks() {
        let fast = run_single(
            &SweepParams {
                block_interval_ms: 200,
                total_time_ms: 10_000,
                ..Default::default()
            },
            99,
        )
        .unwrap();
        let slow = run_single(
            &SweepParams {
                block_interval_ms: 2000,
                total_time_ms: 10_000,
                ..Default::default()
            },
            99,
        )
        .unwrap();
        assert!(
            fast.total_blocks > slow.total_blocks,
            "fast={} should > slow={}",
            fast.total_blocks,
            slow.total_blocks
        );
    }
}
