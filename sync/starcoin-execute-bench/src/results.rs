use std::{collections::HashMap, error::Error, fs::OpenOptions, io::Write};

use anyhow::Context;
use chrono::{DateTime, Local};
use plotters::prelude::*;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;
use starcoin_pipeline_timing::{global_collector, StageTiming};

#[derive(Clone)]
pub enum TransactionExecutionResult {
    /// Added(timestamp_ms) - epoch milliseconds when txn was added to txpool
    Added(u64),
    Rejected(String),
    Culled(String),
    /// Mined(mined_time_ms, block_number, block_id) - epoch ms when MinedBlock event received
    Mined(u64, u64, HashValue),
    /// BlueTemplateSelected(timestamp_ms) - epoch ms when tx appears in blue_txns during create_block_template
    BlueTemplateSelected(u64),
    /// Executed(connected_time_ms, block_number, block_id, block_timestamp_ms) - epoch ms from NewHeadBlock
    /// block_timestamp_ms is the block's timestamp (when it was created)
    Executed(u64, u64, HashValue, u64),
    #[allow(dead_code)]
    ExecutedNotInMain(String),
    Other(String),
}

/// Helper function to format epoch millis to readable string
fn format_epoch_ms(epoch_ms: u64) -> String {
    let secs = (epoch_ms / 1000) as i64;
    let nanos = ((epoch_ms % 1000) * 1_000_000) as u32;
    DateTime::from_timestamp(secs, nanos)
        .map(|dt| {
            let local: DateTime<Local> = dt.into();
            local.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
        })
        .unwrap_or_else(|| format!("{}ms", epoch_ms))
}

fn calculate_statistics(values: &[f64]) -> (f64, f64, f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = values.iter().fold(0.0f64, |a, &b| a.max(b));
    let avg = values.iter().sum::<f64>() / values.len() as f64;

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if sorted.len().is_multiple_of(2) {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };
    (min, max, avg, median)
}

/// Calculate trimmed mean by removing top and bottom N% of values
/// trim_pct: percentage to trim from each end (e.g., 0.1 = remove top 10% and bottom 10%)
fn calculate_trimmed_mean(values: &[f64], trim_pct: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    if values.len() <= 2 {
        // Not enough data to trim, return regular mean
        return values.iter().sum::<f64>() / values.len() as f64;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let trim_count = ((values.len() as f64) * trim_pct).floor() as usize;
    let trim_count = trim_count.max(1).min(values.len() / 2 - 1); // At least 1, but keep at least 1 value

    let trimmed = &sorted[trim_count..sorted.len() - trim_count];
    if trimmed.is_empty() {
        return sorted[sorted.len() / 2]; // Return median if over-trimmed
    }

    trimmed.iter().sum::<f64>() / trimmed.len() as f64
}

#[derive(Debug, Clone, Default)]
pub struct PhaseLatencyStats {
    pub sample_count: usize,
    pub min_ms: f64,
    pub max_ms: f64,
    pub avg_ms: f64,
    pub median_ms: f64,
}

impl PhaseLatencyStats {
    fn from_samples(samples: &[f64]) -> Self {
        let (min_ms, max_ms, avg_ms, median_ms) = calculate_statistics(samples);
        Self {
            sample_count: samples.len(),
            min_ms: if min_ms.is_finite() { min_ms } else { 0.0 },
            max_ms,
            avg_ms,
            median_ms,
        }
    }
}

impl std::fmt::Debug for TransactionExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionExecutionResult::Added(ts_ms) => {
                write!(f, "Added({})", format_epoch_ms(*ts_ms))
            }
            TransactionExecutionResult::Rejected(op_time) => {
                write!(f, "Rejected({})", op_time)
            }
            TransactionExecutionResult::Culled(op_time) => {
                write!(f, "Culled({})", op_time)
            }
            TransactionExecutionResult::Mined(ts_ms, block_number, block_id) => {
                write!(
                    f,
                    "Mined({}, block={}, id={})",
                    format_epoch_ms(*ts_ms),
                    block_number,
                    block_id
                )
            }
            TransactionExecutionResult::BlueTemplateSelected(ts_ms) => {
                write!(f, "BlueTemplateSelected({})", format_epoch_ms(*ts_ms))
            }
            TransactionExecutionResult::Executed(ts_ms, block_number, block_id, block_ts) => {
                write!(
                    f,
                    "Executed({}, block={}, id={}, block_ts={})",
                    format_epoch_ms(*ts_ms),
                    block_number,
                    block_id,
                    format_epoch_ms(*block_ts)
                )
            }
            TransactionExecutionResult::ExecutedNotInMain(op_time) => {
                write!(f, "ExecutedNotInMain({})", op_time)
            }
            TransactionExecutionResult::Other(op_time) => {
                write!(f, "Other({})", op_time)
            }
        }
    }
}

/// Benchmark statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BenchmarkStats {
    pub tps: f64,
    /// Stable TPS: trimmed mean of middle blocks (excludes first/last block, removes outliers)
    /// This is the recommended metric for CI comparison
    #[serde(default)]
    pub stable_tps: f64,
    /// Block-based TPS statistics (calculated per block: from block_timestamp to executed_time)
    pub block_tps_min: f64,
    pub block_tps_max: f64,
    pub block_tps_avg: f64,
    pub block_tps_median: f64,
    /// Mined-based TPS statistics (calculated per block: from mined_time to executed_time)
    pub mined_tps_min: f64,
    pub mined_tps_max: f64,
    pub mined_tps_avg: f64,
    pub mined_tps_median: f64,
    pub total_executed: usize,
    /// Number of benchmark blocks (sample size for TPS calculation)
    #[serde(default)]
    pub block_count: usize,
    /// Number of middle blocks used for stable TPS calculation
    #[serde(default)]
    pub middle_block_count: usize,
    pub unique_txn_count: usize,
    pub duplicate_exec_count: usize,
    pub duplicate_pct: f64,
    pub txpool_to_mined_latency: PhaseLatencyStats,
    pub txpool_to_mined_excluding_target_latency: Option<PhaseLatencyStats>,
    pub mined_to_executed_latency: PhaseLatencyStats,
    pub txpool_to_final_executed_latency: PhaseLatencyStats,
    pub txpool_to_final_executed_excluding_target_latency: Option<PhaseLatencyStats>,
    pub mining_target_ms: Option<u64>,
    pub blue_template_unique_txn_count: usize,
    pub blue_template_final_executed_count: usize,
    pub blue_template_duplicate_exec_count: usize,
    pub blue_template_final_latency: PhaseLatencyStats,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub avg_latency_ms: f64,
    pub median_latency_ms: f64,
}

impl std::fmt::Display for BenchmarkStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "========== Benchmark Results ==========")?;
        writeln!(f, "TPS (executed-time): {:.2}", self.tps)?;
        writeln!(f, "TPS (per-block, block_ts->exec) - Min: {:.2} | Max: {:.2} | Avg: {:.2} | Median: {:.2}",
            self.block_tps_min, self.block_tps_max, self.block_tps_avg, self.block_tps_median)?;
        writeln!(
            f,
            "TPS (per-block, mined->exec) - Min: {:.2} | Max: {:.2} | Avg: {:.2} | Median: {:.2}",
            self.mined_tps_min, self.mined_tps_max, self.mined_tps_avg, self.mined_tps_median
        )?;
        writeln!(
            f,
            "Total Executed: {} | Benchmark Blocks: {}",
            self.total_executed, self.block_count
        )?;
        writeln!(
            f,
            "Unique Txn (with Added): {} | Duplicates: {} ({:.1}%)",
            self.unique_txn_count, self.duplicate_exec_count, self.duplicate_pct
        )?;
        writeln!(
            f,
            "Txpool->Final Executed Latency - Min: {:.2}ms | Max: {:.2}ms | Avg: {:.2}ms | Median: {:.2}ms",
            self.min_latency_ms, self.max_latency_ms, self.avg_latency_ms, self.median_latency_ms
        )?;
        writeln!(
            f,
            "Stage Latency [TxpoolWrite->BlockMined] (n={}) - Min: {:.2}ms | Max: {:.2}ms | Avg: {:.2}ms | Median: {:.2}ms",
            self.txpool_to_mined_latency.sample_count,
            self.txpool_to_mined_latency.min_ms,
            self.txpool_to_mined_latency.max_ms,
            self.txpool_to_mined_latency.avg_ms,
            self.txpool_to_mined_latency.median_ms
        )?;
        if let (Some(target_ms), Some(adjusted)) = (
            self.mining_target_ms,
            self.txpool_to_mined_excluding_target_latency.as_ref(),
        ) {
            writeln!(
                f,
                "Stage Latency [TxpoolWrite->BlockMined, minus target mining {}ms] (n={}) - Min: {:.2}ms | Max: {:.2}ms | Avg: {:.2}ms | Median: {:.2}ms",
                target_ms,
                adjusted.sample_count,
                adjusted.min_ms,
                adjusted.max_ms,
                adjusted.avg_ms,
                adjusted.median_ms
            )?;
        }
        writeln!(
            f,
            "Stage Latency [BlockMined->FinalExecuted] (n={}) - Min: {:.2}ms | Max: {:.2}ms | Avg: {:.2}ms | Median: {:.2}ms",
            self.mined_to_executed_latency.sample_count,
            self.mined_to_executed_latency.min_ms,
            self.mined_to_executed_latency.max_ms,
            self.mined_to_executed_latency.avg_ms,
            self.mined_to_executed_latency.median_ms
        )?;
        writeln!(
            f,
            "Stage Latency [TxpoolWrite->FinalExecuted] (n={}) - Min: {:.2}ms | Max: {:.2}ms | Avg: {:.2}ms | Median: {:.2}ms",
            self.txpool_to_final_executed_latency.sample_count,
            self.txpool_to_final_executed_latency.min_ms,
            self.txpool_to_final_executed_latency.max_ms,
            self.txpool_to_final_executed_latency.avg_ms,
            self.txpool_to_final_executed_latency.median_ms
        )?;
        if let (Some(target_ms), Some(adjusted)) = (
            self.mining_target_ms,
            self.txpool_to_final_executed_excluding_target_latency
                .as_ref(),
        ) {
            writeln!(
                f,
                "Stage Latency [TxpoolWrite->FinalExecuted, minus target mining {}ms] (n={}) - Min: {:.2}ms | Max: {:.2}ms | Avg: {:.2}ms | Median: {:.2}ms",
                target_ms,
                adjusted.sample_count,
                adjusted.min_ms,
                adjusted.max_ms,
                adjusted.avg_ms,
                adjusted.median_ms
            )?;
        }
        writeln!(
            f,
            "Blue Txns (from create_block_template blue_txns) - Unique: {} | Final Executed: {} | Duplicate Executions: {}",
            self.blue_template_unique_txn_count,
            self.blue_template_final_executed_count,
            self.blue_template_duplicate_exec_count
        )?;
        writeln!(
            f,
            "Blue Txns Final Latency [Added->FinalExecuted] (n={}) - Min: {:.2}ms | Max: {:.2}ms | Avg: {:.2}ms | Median: {:.2}ms",
            self.blue_template_final_latency.sample_count,
            self.blue_template_final_latency.min_ms,
            self.blue_template_final_latency.max_ms,
            self.blue_template_final_latency.avg_ms,
            self.blue_template_final_latency.median_ms
        )?;
        writeln!(f, "========================================")?;
        Ok(())
    }
}

/// JSON output structure for AI agent loop consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkJsonOutput {
    /// ISO8601 timestamp of the benchmark run
    pub timestamp: String,
    /// Summary statistics
    pub summary: BenchmarkStats,
    /// Per-stage pipeline timing (if available)
    pub pipeline_stages: HashMap<String, StageTiming>,
    /// Number of blocks processed
    pub block_count: usize,
    /// Top blocks with highest latency
    pub top_latency_blocks: Vec<TopLatencyBlock>,
}

/// A block with high latency for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopLatencyBlock {
    pub block_id: String,
    pub block_number: u64,
    pub max_latency_ms: f64,
}

pub struct ResultsDumper<'a> {
    transaction_data: &'a HashMap<HashValue, Vec<TransactionExecutionResult>>,
    mining_target_ms: Option<u64>,
}

impl<'a> ResultsDumper<'a> {
    pub fn with_mining_target(
        transaction_data: &'a HashMap<HashValue, Vec<TransactionExecutionResult>>,
        mining_target_ms: Option<u64>,
    ) -> Self {
        Self {
            transaction_data,
            mining_target_ms,
        }
    }

    /// Calculate and return benchmark statistics
    pub fn calculate_stats(&self) -> BenchmarkStats {
        let (executions, unique_txn_count, duplicate_exec_count) = self.collect_executions();

        // Count raw statistics for debugging
        let total_txn_entries = self.transaction_data.len();
        let mut added_count = 0usize;
        let mut executed_count = 0usize;
        for events in self.transaction_data.values() {
            for ev in events {
                match ev {
                    TransactionExecutionResult::Added(_) => added_count += 1,
                    TransactionExecutionResult::Executed(_, _, _, _) => executed_count += 1,
                    _ => {}
                }
            }
        }

        // Filter finite latency data
        let all_delays: Vec<f64> = executions
            .iter()
            .filter(|(_, _, latency)| latency.is_finite())
            .map(|(_, _, latency)| *latency)
            .collect();

        let total_txns = all_delays.len();

        // Log debug info
        info!("DEBUG: total_txn_entries={}, added_events={}, executed_events={}, unique_with_added={}, matched_with_latency={}",
            total_txn_entries, added_count, executed_count, unique_txn_count, total_txns);

        let (txpool_to_mined_samples, mined_to_executed_samples) =
            self.collect_stage_latency_samples();
        let txpool_to_mined_latency = PhaseLatencyStats::from_samples(&txpool_to_mined_samples);
        let txpool_to_mined_excluding_target_latency = self.mining_target_ms.map(|v| {
            let target_ms = v as f64;
            let adjusted: Vec<f64> = txpool_to_mined_samples
                .iter()
                .map(|latency_ms| (latency_ms - target_ms).max(0.0))
                .collect();
            PhaseLatencyStats::from_samples(&adjusted)
        });
        let mined_to_executed_latency = PhaseLatencyStats::from_samples(&mined_to_executed_samples);
        let txpool_to_final_executed_latency = PhaseLatencyStats::from_samples(&all_delays);
        let txpool_to_final_executed_excluding_target_latency = self.mining_target_ms.map(|v| {
            let target_ms = v as f64;
            let adjusted: Vec<f64> = all_delays
                .iter()
                .map(|latency_ms| (latency_ms - target_ms).max(0.0))
                .collect();
            PhaseLatencyStats::from_samples(&adjusted)
        });
        let (
            blue_template_unique_txn_count,
            blue_template_final_executed_count,
            blue_template_duplicate_exec_count,
            blue_template_final_latency,
        ) = self.collect_blue_template_stats();

        // Calculate TPS based on executed times (OLD - affected by event queue delays)
        let tps = self.calculate_tps_from_executed();

        // Calculate TPS based on block timestamps (STABLE - not affected by event queue delays)
        // This uses block header timestamps set when blocks are created, providing more
        // consistent measurements for CI benchmarks.
        let (block_ts_tps, _total_txns, _first_block_ts, _last_block_ts) =
            self.calculate_tps_from_block_timestamps();

        // Use block timestamp TPS as stable_tps (more reliable for CI)
        let stable_tps = block_ts_tps;

        // Calculate per-block TPS statistics (for reference, not used as stable_tps)
        let (
            block_tps_min,
            block_tps_max,
            block_tps_avg,
            block_tps_median,
            _old_stable_tps,
            block_count,
            middle_block_count,
        ) = self.calculate_per_block_tps_stats();
        // Calculate per-block TPS statistics (mined_time -> executed_time)
        let (mined_tps_min, mined_tps_max, mined_tps_avg, mined_tps_median) =
            self.calculate_per_block_mined_tps_stats();

        let duplicate_pct = if unique_txn_count > 0 {
            duplicate_exec_count as f64 / unique_txn_count as f64 * 100.0
        } else {
            0.0
        };

        BenchmarkStats {
            tps,
            stable_tps,
            block_tps_min,
            block_tps_max,
            block_tps_avg,
            block_tps_median,
            mined_tps_min,
            mined_tps_max,
            mined_tps_avg,
            mined_tps_median,
            total_executed: total_txns,
            block_count,
            middle_block_count,
            unique_txn_count,
            duplicate_exec_count,
            duplicate_pct,
            txpool_to_mined_latency,
            txpool_to_mined_excluding_target_latency,
            mined_to_executed_latency,
            txpool_to_final_executed_latency: txpool_to_final_executed_latency.clone(),
            txpool_to_final_executed_excluding_target_latency,
            mining_target_ms: self.mining_target_ms,
            blue_template_unique_txn_count,
            blue_template_final_executed_count,
            blue_template_duplicate_exec_count,
            blue_template_final_latency,
            min_latency_ms: txpool_to_final_executed_latency.min_ms,
            max_latency_ms: txpool_to_final_executed_latency.max_ms,
            avg_latency_ms: txpool_to_final_executed_latency.avg_ms,
            median_latency_ms: txpool_to_final_executed_latency.median_ms,
        }
    }

    fn collect_blue_template_stats(&self) -> (usize, usize, usize, PhaseLatencyStats) {
        let mut unique_txn_count = 0usize;
        let mut final_executed_count = 0usize;
        let mut duplicate_exec_count = 0usize;
        let mut latencies: Vec<f64> = Vec::new();

        for events in self.transaction_data.values() {
            let mut has_blue_template = false;
            let mut added_times: Vec<u64> = Vec::new();
            let mut executed_times: Vec<u64> = Vec::new();

            for ev in events {
                match ev {
                    TransactionExecutionResult::BlueTemplateSelected(_) => {
                        has_blue_template = true;
                    }
                    TransactionExecutionResult::Added(ts_ms) => {
                        added_times.push(*ts_ms);
                    }
                    TransactionExecutionResult::Executed(ts_ms, _, _, _) => {
                        executed_times.push(*ts_ms);
                    }
                    _ => {}
                }
            }

            if !has_blue_template || added_times.is_empty() {
                continue;
            }

            unique_txn_count += 1;
            if executed_times.is_empty() {
                continue;
            }

            final_executed_count += 1;
            if executed_times.len() > 1 {
                duplicate_exec_count += executed_times.len() - 1;
            }

            let first_add = *added_times.iter().min().unwrap();
            let last_exec = *executed_times.iter().max().unwrap();
            if last_exec >= first_add {
                latencies.push((last_exec - first_add) as f64);
            } else {
                latencies.push(0.0);
            }
        }

        (
            unique_txn_count,
            final_executed_count,
            duplicate_exec_count,
            PhaseLatencyStats::from_samples(&latencies),
        )
    }

    fn collect_stage_latency_samples(&self) -> (Vec<f64>, Vec<f64>) {
        let mut txpool_to_mined_samples: Vec<f64> = Vec::new();
        let mut mined_to_executed_samples: Vec<f64> = Vec::new();

        for events in self.transaction_data.values() {
            let mut added_times: Vec<u64> = Vec::new();
            let mut mined_infos: Vec<(u64, HashValue)> = Vec::new();
            let mut executed_infos: Vec<(u64, HashValue)> = Vec::new();

            for ev in events {
                match ev {
                    TransactionExecutionResult::Added(ts_ms) => {
                        added_times.push(*ts_ms);
                    }
                    TransactionExecutionResult::Mined(ts_ms, _block_number, block_id) => {
                        mined_infos.push((*ts_ms, *block_id));
                    }
                    TransactionExecutionResult::Executed(
                        ts_ms,
                        _block_number,
                        block_id,
                        _block_ts,
                    ) => {
                        executed_infos.push((*ts_ms, *block_id));
                    }
                    _ => {}
                }
            }

            let first_add = added_times.iter().min().copied();
            let first_mined = mined_infos.iter().map(|(ts, _)| *ts).min();
            if let (Some(add_ts), Some(mined_ts)) = (first_add, first_mined) {
                if mined_ts >= add_ts {
                    txpool_to_mined_samples.push((mined_ts - add_ts) as f64);
                }
            }

            if let Some((final_exec_ts, final_exec_block_id)) =
                executed_infos.iter().max_by_key(|(ts, _)| *ts).copied()
            {
                let mined_for_final_block = mined_infos
                    .iter()
                    .filter(|(_, block_id)| *block_id == final_exec_block_id)
                    .map(|(ts, _)| *ts)
                    .min()
                    .or_else(|| {
                        mined_infos
                            .iter()
                            .map(|(ts, _)| *ts)
                            .filter(|ts| *ts <= final_exec_ts)
                            .max()
                    });
                if let Some(mined_ts) = mined_for_final_block {
                    if final_exec_ts >= mined_ts {
                        mined_to_executed_samples.push((final_exec_ts - mined_ts) as f64);
                    }
                }
            }
        }

        (txpool_to_mined_samples, mined_to_executed_samples)
    }

    /// Get top N blocks with highest latency transactions (deduplicated by block_id)
    /// Returns: Vec of (block_id, block_number, max_latency_ms)
    pub fn get_top_latency_blocks(&self, top_n: usize) -> Vec<(HashValue, u64, f64)> {
        // Collect (txn_id, latency, block_id, block_number) for transactions with valid latency
        let mut txn_latencies: Vec<(HashValue, f64, HashValue, u64)> = Vec::new();

        for (txn_id, events) in self.transaction_data.iter() {
            let mut added_times: Vec<u64> = Vec::new();
            let mut executed_info: Vec<(u64, u64, HashValue)> = Vec::new(); // (exec_time, block_num, block_id)

            for ev in events {
                match ev {
                    TransactionExecutionResult::Added(ts_ms) => {
                        added_times.push(*ts_ms);
                    }
                    TransactionExecutionResult::Executed(ts_ms, block_number, block_id, _) => {
                        executed_info.push((*ts_ms, *block_number, *block_id));
                    }
                    _ => {}
                }
            }

            if added_times.is_empty() || executed_info.is_empty() {
                continue;
            }

            let first_add = *added_times.iter().min().unwrap();
            // Get the last execution (max exec time)
            let last_exec = executed_info.iter().max_by_key(|(ts, _, _)| ts).unwrap();
            let latency_ms = if last_exec.0 >= first_add {
                (last_exec.0 - first_add) as f64
            } else {
                0.0
            };

            txn_latencies.push((*txn_id, latency_ms, last_exec.2, last_exec.1));
        }

        // Sort by latency descending
        txn_latencies.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Deduplicate by block_id, keep highest latency per block
        let mut seen_blocks: std::collections::HashSet<HashValue> =
            std::collections::HashSet::new();
        let mut result: Vec<(HashValue, u64, f64)> = Vec::new();

        for (_, latency, block_id, block_number) in txn_latencies {
            if seen_blocks.contains(&block_id) {
                continue;
            }
            seen_blocks.insert(block_id);
            result.push((block_id, block_number, latency));
            if result.len() >= top_n {
                break;
            }
        }

        result
    }

    /// Calculate TPS based on block timestamps (more stable than event processing time)
    /// This uses block header timestamps which are set when the block is created,
    /// not affected by event queue delays.
    fn calculate_tps_from_block_timestamps(&self) -> (f64, usize, u64, u64) {
        // Collect: block_number -> (block_timestamp, txn_count)
        let mut block_data: HashMap<u64, (u64, usize)> = HashMap::new();

        for events in self.transaction_data.values() {
            for ev in events {
                if let TransactionExecutionResult::Executed(_, block_number, _, block_ts) = ev {
                    let entry = block_data.entry(*block_number).or_insert((*block_ts, 0));
                    entry.1 += 1;
                }
            }
        }

        if block_data.len() < 2 {
            return (0.0, 0, 0, 0);
        }

        // Sort by block_number
        let mut sorted: Vec<(u64, u64, usize)> = block_data
            .into_iter()
            .map(|(num, (ts, cnt))| (num, ts, cnt))
            .collect();
        sorted.sort_by_key(|(num, _, _)| *num);

        let first_block_ts = sorted.first().unwrap().1;
        let last_block_ts = sorted.last().unwrap().1;
        let total_txns: usize = sorted.iter().map(|(_, _, cnt)| cnt).sum();

        let duration_secs = (last_block_ts - first_block_ts) as f64 / 1000.0;

        let tps = if duration_secs > 0.0 {
            total_txns as f64 / duration_secs
        } else {
            0.0
        };

        (tps, total_txns, first_block_ts, last_block_ts)
    }

    /// Calculate TPS based on executed transaction times (OLD - affected by event queue delays)
    fn calculate_tps_from_executed(&self) -> f64 {
        let mut all_exec_times: Vec<u64> = Vec::new();

        for events in self.transaction_data.values() {
            for ev in events {
                if let TransactionExecutionResult::Executed(ts_ms, _, _, _) = ev {
                    all_exec_times.push(*ts_ms);
                }
            }
        }

        if all_exec_times.len() < 2 {
            return 0.0;
        }

        all_exec_times.sort();
        let first = *all_exec_times.first().unwrap();
        let last = *all_exec_times.last().unwrap();
        let duration_secs = (last - first) as f64 / 1000.0;

        if duration_secs > 0.0 {
            all_exec_times.len() as f64 / duration_secs
        } else {
            all_exec_times.len() as f64
        }
    }

    /// Calculate per-block TPS statistics.
    /// For each block, TPS = txn_count / (exec_time - block_timestamp) in seconds.
    /// Returns: (min_tps, max_tps, avg_tps, median_tps, stable_tps, block_count, middle_block_count)
    /// NOTE: This uses event processing time (exec_ts) which is affected by event queue delays.
    /// The stable_tps from this method is NOT used anymore - we use block timestamp TPS instead.
    fn calculate_per_block_tps_stats(&self) -> (f64, f64, f64, f64, f64, usize, usize) {
        // Collect block data: block_number -> (block_timestamp, exec_time, txn_count)
        let mut block_data: HashMap<u64, (u64, u64, usize)> = HashMap::new();

        for events in self.transaction_data.values() {
            for ev in events {
                if let TransactionExecutionResult::Executed(
                    exec_ts,
                    block_number,
                    _block_id,
                    block_ts,
                ) = ev
                {
                    let entry = block_data
                        .entry(*block_number)
                        .or_insert((*block_ts, *exec_ts, 0));
                    // Update exec_time to the max (last execution time for this block)
                    if *exec_ts > entry.1 {
                        entry.1 = *exec_ts;
                    }
                    entry.2 += 1; // Increment txn count
                }
            }
        }

        let block_count = block_data.len();

        if block_data.is_empty() {
            return (0.0, 0.0, 0.0, 0.0, 0.0, 0, 0);
        }

        // Sort blocks by block_number to identify first/last
        let mut sorted_blocks: Vec<(u64, (u64, u64, usize))> = block_data.into_iter().collect();
        sorted_blocks.sort_by_key(|(block_num, _)| *block_num);

        // Calculate TPS for each block
        let mut all_block_tps: Vec<(u64, f64)> = Vec::new(); // (block_number, tps)
        for (block_num, (block_ts, exec_ts, txn_count)) in sorted_blocks.iter() {
            let duration_ms = exec_ts.saturating_sub(*block_ts);
            let duration_secs = duration_ms as f64 / 1000.0;
            let tps = if duration_secs > 0.0 {
                *txn_count as f64 / duration_secs
            } else {
                0.0
            };
            if *exec_ts > *block_ts && *txn_count > 0 && duration_secs > 0.0 {
                all_block_tps.push((*block_num, tps));
            }
        }

        if all_block_tps.is_empty() {
            return (0.0, 0.0, 0.0, 0.0, 0.0, block_count, 0);
        }

        // Extract just TPS values for statistics
        let all_tps_values: Vec<f64> = all_block_tps.iter().map(|(_, tps)| *tps).collect();
        let (min_tps, max_tps, avg_tps, median_tps) = calculate_statistics(&all_tps_values);

        // Calculate stable TPS using robust statistics:
        // 1. For 5+ blocks: exclude first/last by time order, then use trimmed mean
        // 2. For 3-4 blocks: remove min/max TPS values, use remaining mean
        // 3. For 1-2 blocks: just use median (best we can do)
        let (stable_tps, middle_block_count) = if all_block_tps.len() >= 5 {
            // Enough blocks: exclude first/last by time, use trimmed mean
            let middle_tps: Vec<f64> = all_block_tps[1..all_block_tps.len() - 1]
                .iter()
                .map(|(_, tps)| *tps)
                .collect();
            let count = middle_tps.len();
            (calculate_trimmed_mean(&middle_tps, 0.1), count)
        } else if all_block_tps.len() >= 3 {
            // Few blocks: remove min/max TPS (statistical outliers, not by position)
            let mut sorted_tps = all_tps_values.clone();
            sorted_tps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            // Remove 1 min and 1 max
            let trimmed: Vec<f64> = sorted_tps[1..sorted_tps.len() - 1].to_vec();
            let count = trimmed.len();
            let mean = trimmed.iter().sum::<f64>() / count as f64;
            (mean, count)
        } else {
            // 1-2 blocks: use median
            (median_tps, all_block_tps.len())
        };

        (
            min_tps,
            max_tps,
            avg_tps,
            median_tps,
            stable_tps,
            block_count,
            middle_block_count,
        )
    }

    /// Calculate per-block TPS statistics based on Mined event time.
    /// For each block, TPS = txn_count / (exec_time - mined_time) in seconds.
    /// Returns: (min_tps, max_tps, avg_tps, median_tps)
    fn calculate_per_block_mined_tps_stats(&self) -> (f64, f64, f64, f64) {
        // Collect block data: block_id -> (mined_time, exec_time, txn_count)
        let mut block_data: HashMap<HashValue, (Option<u64>, Option<u64>, usize)> = HashMap::new();

        for events in self.transaction_data.values() {
            for ev in events {
                match ev {
                    TransactionExecutionResult::Mined(mined_ts, _block_number, block_id) => {
                        let entry = block_data.entry(*block_id).or_insert((None, None, 0));
                        // Use the earliest mined time
                        if entry.0.is_none() || *mined_ts < entry.0.unwrap() {
                            entry.0 = Some(*mined_ts);
                        }
                    }
                    TransactionExecutionResult::Executed(
                        exec_ts,
                        _block_number,
                        block_id,
                        _block_ts,
                    ) => {
                        let entry = block_data.entry(*block_id).or_insert((None, None, 0));
                        // Use the latest exec time
                        if entry.1.is_none() || *exec_ts > entry.1.unwrap() {
                            entry.1 = Some(*exec_ts);
                        }
                        entry.2 += 1; // Increment txn count
                    }
                    _ => {}
                }
            }
        }

        if block_data.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        // Calculate TPS for each block
        let mut block_tps_list: Vec<f64> = Vec::new();
        for (_block_id, (mined_ts_opt, exec_ts_opt, txn_count)) in block_data.iter() {
            if let (Some(mined_ts), Some(exec_ts)) = (mined_ts_opt, exec_ts_opt) {
                if *exec_ts > *mined_ts && *txn_count > 0 {
                    let duration_secs = (*exec_ts - *mined_ts) as f64 / 1000.0;
                    if duration_secs > 0.0 {
                        let tps = *txn_count as f64 / duration_secs;
                        block_tps_list.push(tps);
                    }
                }
            }
        }

        if block_tps_list.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let (min_tps, max_tps, avg_tps, median_tps) = calculate_statistics(&block_tps_list);

        (min_tps, max_tps, avg_tps, median_tps)
    }

    pub fn dump_results(&self) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("./transaction_results.txt")
            .context("failed to open transaction_results.txt")?;

        for (transaction, results) in self.transaction_data {
            writeln!(
                file,
                "transaction id: {}, results: {:?}",
                *transaction, results
            )
            .context("failed to write transaction results")?;
        }

        self.export_combined_svg("./benchmark_results.svg")
            .map_err(|e| anyhow::format_err!("failed to export benchmark results svg: {}", e))?;
        Ok(())
    }

    /// Export benchmark results to JSON format for AI agent loop consumption
    pub fn export_json(&self, file_path: &str) -> anyhow::Result<()> {
        let stats = self.calculate_stats();

        // Build stage timing data from global collector
        let stage_timings: HashMap<String, StageTiming> = global_collector()
            .calculate_stage_stats()
            .into_iter()
            .map(|(stage, timing)| (stage.to_string(), timing))
            .collect();

        let output = BenchmarkJsonOutput {
            timestamp: chrono::Utc::now().to_rfc3339(),
            summary: stats,
            pipeline_stages: stage_timings,
            block_count: self.get_user_transfer_block_stats().len(),
            top_latency_blocks: self
                .get_top_latency_blocks(10)
                .into_iter()
                .map(|(id, num, lat)| TopLatencyBlock {
                    block_id: id.to_string(),
                    block_number: num,
                    max_latency_ms: lat,
                })
                .collect(),
        };

        let json = serde_json::to_string_pretty(&output)
            .context("failed to serialize benchmark results to JSON")?;

        std::fs::write(file_path, json).context("failed to write JSON file")?;

        info!("Exported benchmark results to {}", file_path);
        Ok(())
    }

    /// Collect txpool->final-executed latency for each transaction.
    /// Returns: (transaction latency data list, unique transaction count, duplicate execution count)
    /// Each element is (transaction ID, Added time in epoch ms, latency in milliseconds)
    fn collect_executions(&self) -> (Vec<(HashValue, u64, f64)>, usize, usize) {
        let mut results: Vec<(HashValue, u64, f64)> = Vec::new();
        let mut unique_txn_count = 0usize;
        let mut duplicate_exec_count = 0usize;

        for (txn_id, events) in self.transaction_data.iter() {
            let mut added_times: Vec<u64> = Vec::new();
            let mut executed_times: Vec<u64> = Vec::new();

            for ev in events {
                match ev {
                    TransactionExecutionResult::Added(ts_ms) => {
                        added_times.push(*ts_ms);
                    }
                    TransactionExecutionResult::Executed(ts_ms, _, _, _) => {
                        executed_times.push(*ts_ms);
                    }
                    _ => {}
                }
            }

            if added_times.is_empty() {
                continue;
            }

            unique_txn_count += 1;

            // Earliest Added event approximates txpool insertion time.
            let first_add = *added_times.iter().min().unwrap();

            if executed_times.is_empty() {
                results.push((*txn_id, first_add, f64::INFINITY));
                continue;
            }

            if executed_times.len() > 1 {
                duplicate_exec_count += executed_times.len() - 1;
            }

            // Use latest execution as final execution time when duplicates exist.
            let last_exec = *executed_times.iter().max().unwrap();
            // Calculate latency: final_executed_time - added_time
            let delay_ms = if last_exec >= first_add {
                (last_exec - first_add) as f64
            } else {
                0.0
            };
            results.push((*txn_id, first_add, delay_ms));
        }

        // Sort by Added time
        results.sort_by_key(|(_, add_time, _)| *add_time);

        (results, unique_txn_count, duplicate_exec_count)
    }

    fn get_user_transfer_block_stats(&self) -> Vec<(u64, usize)> {
        let mut block_counts: HashMap<u64, usize> = HashMap::new();

        for events in self.transaction_data.values() {
            let has_added = events
                .iter()
                .any(|e| matches!(e, TransactionExecutionResult::Added(_)));

            if has_added {
                for ev in events {
                    if let TransactionExecutionResult::Executed(_, block_number, _, _) = ev {
                        *block_counts.entry(*block_number).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut result: Vec<(u64, usize)> = block_counts.into_iter().collect();
        result.sort_by_key(|(block_num, _)| *block_num);
        result
    }

    pub fn export_combined_svg(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let (executions, unique_txn_count, duplicate_exec_count) = self.collect_executions();
        let block_stats = self.get_user_transfer_block_stats();

        let root = SVGBackend::new(file_path, (1600, 1600)).into_drawing_area();
        root.fill(&WHITE)?;

        let (upper, lower) = root.split_vertically(800);

        self.draw_latency_chart(&upper, &executions, unique_txn_count, duplicate_exec_count)?;
        self.draw_block_txn_chart(&lower, &block_stats)?;

        root.present()?;
        Ok(())
    }

    fn draw_latency_chart(
        &self,
        area: &DrawingArea<SVGBackend, plotters::coord::Shift>,
        executions: &[(HashValue, u64, f64)],
        unique_txn_count: usize,
        duplicate_exec_count: usize,
    ) -> Result<(), Box<dyn Error>> {
        if executions.is_empty() {
            return Ok(());
        }

        // Filter finite latency data
        let valid_executions: Vec<_> = executions
            .iter()
            .filter(|(_, _, latency)| latency.is_finite())
            .collect();

        let max_latency: f64 = valid_executions
            .iter()
            .map(|(_, _, latency)| *latency)
            .fold(0.0f64, |acc, d| acc.max(d))
            .max(1.0);

        let num_bars = valid_executions.len();
        if num_bars == 0 {
            return Ok(());
        }

        let mut chart = ChartBuilder::on(area)
            .caption("Txpool to Final Executed Latency", ("sans-serif", 28))
            .margin(20)
            .x_label_area_size(120)
            .y_label_area_size(70)
            .build_cartesian_2d(0f64..(num_bars as f64), 0f64..max_latency)?;

        chart
            .configure_mesh()
            .x_desc("Transaction Index (by Added Time)")
            .y_desc("Latency (ms)")
            .x_label_formatter(&|x| {
                let idx = *x as usize;
                if idx < valid_executions.len() {
                    let epoch_ms = valid_executions[idx].1 as i64;
                    let secs = epoch_ms / 1000;
                    let nanos = ((epoch_ms % 1000) * 1_000_000) as u32;
                    if let Some(dt) = DateTime::from_timestamp(secs, nanos) {
                        let local: DateTime<Local> = dt.into();
                        local.format("%H:%M:%S").to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            })
            .axis_desc_style(("sans-serif", 20))
            .label_style(("sans-serif", 10))
            .x_labels(num_bars.min(15))
            .draw()?;

        let bar_width = 0.8;
        for (idx, (_, _, latency)) in valid_executions.iter().enumerate() {
            let x_center = idx as f64 + 0.5;
            let x_left = x_center - bar_width / 2.0;
            let x_right = x_center + bar_width / 2.0;

            chart.draw_series(std::iter::once(Rectangle::new(
                [(x_left, 0.0), (x_right, latency.min(max_latency))],
                RGBColor(50, 100, 220).filled(),
            )))?;
        }

        let all_delays: Vec<f64> = valid_executions.iter().map(|(_, _, l)| *l).collect();
        let total_txns = all_delays.len();
        let (min_delay, max_delay_stat, avg_delay, median_delay) =
            calculate_statistics(&all_delays);

        // This chart uses submission TPS (from first/last Added timestamp).
        // It intentionally differs from calculate_tps_from_executed(), which reports execution TPS.
        let tps = if valid_executions.len() >= 2 {
            let first_time = valid_executions.first().map(|(_, t, _)| *t);
            let last_time = valid_executions.last().map(|(_, t, _)| *t);
            if let (Some(first), Some(last)) = (first_time, last_time) {
                let duration_ms = last.saturating_sub(first);
                let duration_secs = duration_ms as f64 / 1000.0;
                if duration_secs > 0.0 {
                    total_txns as f64 / duration_secs
                } else {
                    total_txns as f64
                }
            } else {
                0.0
            }
        } else {
            total_txns as f64
        };

        let duplicate_pct = if unique_txn_count > 0 {
            duplicate_exec_count as f64 / unique_txn_count as f64 * 100.0
        } else {
            0.0
        };

        let stats_lines = [
            format!("TPS: {:.2}", tps),
            format!(
                "Total Executed: {} | Unique Txn: {} | Duplicates: {} ({:.1}%)",
                total_txns, unique_txn_count, duplicate_exec_count, duplicate_pct
            ),
            format!(
                "Txpool->Final Executed Latency - Min: {:.2}ms | Max: {:.2}ms | Avg: {:.2}ms | Median: {:.2}ms",
                min_delay, max_delay_stat, avg_delay, median_delay
            ),
        ];

        let line_height = 22;
        let start_y = 720;
        for (i, line) in stats_lines.iter().enumerate() {
            area.draw(&Text::new(
                line.clone(),
                (50, start_y + (i as i32) * line_height),
                ("sans-serif", 14).into_font().color(&BLACK),
            ))?;
        }

        Ok(())
    }

    fn build_display_items(&self, block_stats: &[(u64, usize)]) -> Vec<(String, usize, bool)> {
        if block_stats.is_empty() {
            return Vec::new();
        }

        let mut items: Vec<(String, usize, bool)> = Vec::new();
        let mut i = 0;

        while i < block_stats.len() {
            let (current_block, current_count) = block_stats[i];
            items.push((format!("{}", current_block), current_count, false));

            if i + 1 < block_stats.len() {
                let (next_block, _) = block_stats[i + 1];
                let gap = next_block - current_block - 1;
                if gap > 0 {
                    let label = if gap == 1 {
                        format!("{}", current_block + 1)
                    } else {
                        format!("{}-{}", current_block + 1, next_block - 1)
                    };
                    items.push((label, 0, true));
                }
            }

            i += 1;
        }

        items
    }

    fn draw_block_txn_chart(
        &self,
        area: &DrawingArea<SVGBackend, plotters::coord::Shift>,
        block_stats: &[(u64, usize)],
    ) -> Result<(), Box<dyn Error>> {
        if block_stats.is_empty() {
            return Ok(());
        }

        let display_items = self.build_display_items(block_stats);
        let num_items = display_items.len();

        let txn_counts: Vec<usize> = block_stats.iter().map(|(_, count)| *count).collect();
        let block_numbers: Vec<u64> = block_stats.iter().map(|(num, _)| *num).collect();

        let min_block = *block_numbers.first().unwrap();
        let max_block = *block_numbers.last().unwrap();
        let max_txn_count = *txn_counts.iter().max().unwrap_or(&1);
        let min_txn_count = *txn_counts.iter().min().unwrap_or(&0);
        let total_txns: usize = txn_counts.iter().sum();
        let num_blocks = block_stats.len();
        let empty_blocks = (max_block - min_block + 1) as usize - num_blocks;

        let avg_txn_count = if !txn_counts.is_empty() {
            total_txns as f64 / txn_counts.len() as f64
        } else {
            0.0
        };
        let median_txn_count = if !txn_counts.is_empty() {
            let mut sorted = txn_counts.clone();
            sorted.sort();
            if sorted.len().is_multiple_of(2) {
                (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) as f64 / 2.0
            } else {
                sorted[sorted.len() / 2] as f64
            }
        } else {
            0.0
        };

        let labels: Vec<String> = display_items
            .iter()
            .map(|(label, _, _)| label.clone())
            .collect();

        let mut chart = ChartBuilder::on(area)
            .caption("User Transactions per Block", ("sans-serif", 28))
            .margin(20)
            .x_label_area_size(60)
            .y_label_area_size(70)
            .build_cartesian_2d(
                0f64..(num_items as f64),
                0f64..((max_txn_count as f64) * 1.1),
            )?;

        chart
            .configure_mesh()
            .x_desc("Block Number")
            .y_desc("Transaction Count")
            .x_label_formatter(&|x| {
                let idx = *x as usize;
                if idx < labels.len() {
                    labels[idx].clone()
                } else {
                    String::new()
                }
            })
            .axis_desc_style(("sans-serif", 20))
            .label_style(("sans-serif", 12))
            .x_labels(num_items.min(30))
            .draw()?;

        let bar_width = 0.8;
        for (idx, (_, count, is_empty)) in display_items.iter().enumerate() {
            let x_center = idx as f64 + 0.5;
            let x_left = x_center - bar_width / 2.0;
            let x_right = x_center + bar_width / 2.0;

            if *is_empty {
                let bar_height = max_txn_count as f64 * 0.02;
                chart.draw_series(std::iter::once(Rectangle::new(
                    [(x_left, 0.0), (x_right, bar_height)],
                    RGBColor(200, 50, 50).filled(),
                )))?;

                chart.draw_series(std::iter::once(Text::new(
                    "0".to_string(),
                    (x_center, bar_height + max_txn_count as f64 * 0.02),
                    ("sans-serif", 10).into_font().color(&RGBColor(200, 50, 50)),
                )))?;
            } else {
                chart.draw_series(std::iter::once(Rectangle::new(
                    [(x_left, 0.0), (x_right, *count as f64)],
                    RGBColor(50, 150, 100).filled(),
                )))?;

                chart.draw_series(std::iter::once(Text::new(
                    format!("{}", count),
                    (x_center, *count as f64 + max_txn_count as f64 * 0.02),
                    ("sans-serif", 10).into_font().color(&BLACK),
                )))?;
            }
        }

        let stats_lines = [
            format!(
                "Block Range: {} - {} ({} blocks with txns, {} empty blocks)",
                min_block, max_block, num_blocks, empty_blocks
            ),
            format!("Total Transactions: {}", total_txns),
            format!(
                "Txns per Block - Min: {} | Max: {} | Avg: {:.2} | Median: {:.2}",
                min_txn_count, max_txn_count, avg_txn_count, median_txn_count
            ),
        ];

        let line_height = 22;
        let start_y = 720;
        for (i, line) in stats_lines.iter().enumerate() {
            area.draw(&Text::new(
                line.clone(),
                (50, start_y + (i as i32) * line_height),
                ("sans-serif", 14).into_font().color(&BLACK),
            ))?;
        }

        Ok(())
    }
}
