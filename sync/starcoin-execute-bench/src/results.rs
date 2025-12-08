use std::{collections::HashMap, error::Error, fs::OpenOptions, io::Write};

use chrono::NaiveDateTime;
use plotters::prelude::*;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::info;

#[derive(Clone)]
pub enum TransactionExecutionResult {
    Added(String),
    Rejected(String),
    Culled(String),
    Executed(String, u64),
    #[allow(dead_code)]
    ExecutedNotInMain(String),
    Other(String),
}

impl std::fmt::Debug for TransactionExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionExecutionResult::Added(op_time) => {
                write!(f, "Added({})", op_time)
            }
            TransactionExecutionResult::Rejected(op_time) => {
                write!(f, "Rejected({})", op_time)
            }
            TransactionExecutionResult::Culled(op_time) => {
                write!(f, "Culled({})", op_time)
            }
            TransactionExecutionResult::Executed(op_time, block_number) => {
                write!(f, "Executed({}, block={})", op_time, block_number)
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

/// Per-block TPS statistics
#[derive(Debug, Clone)]
pub struct BlockTpsStats {
    pub block_number: u64,
    pub user_txn_count: usize,
    pub time_since_last_block_ms: f64,
    pub instant_tps: f64,
}

/// Benchmark statistics
#[derive(Debug, Clone)]
pub struct BenchmarkStats {
    pub tps: f64,
    pub user_txn_tps: f64, // TPS only for user transactions
    pub total_executed: usize,
    pub unique_txn_count: usize,
    pub duplicate_exec_count: usize,
    pub duplicate_pct: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
    pub avg_latency_ms: f64,
    pub median_latency_ms: f64,
    pub block_tps_stats: Vec<BlockTpsStats>, // Per-block TPS
    pub min_block_tps: f64,
    pub max_block_tps: f64,
    pub avg_block_tps: f64,
}

impl std::fmt::Display for BenchmarkStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "========== Benchmark Results ==========")?;
        writeln!(f, "Overall TPS: {:.2} (all txns) | User TPS: {:.2} (user txns only)", self.tps, self.user_txn_tps)?;
        writeln!(f, "Total Executed: {}", self.total_executed)?;
        writeln!(
            f,
            "Unique Txn (with Added): {} | Duplicates: {} ({:.1}%)",
            self.unique_txn_count, self.duplicate_exec_count, self.duplicate_pct
        )?;
        writeln!(
            f,
            "Latency - Min: {:.2}ms | Max: {:.2}ms | Avg: {:.2}ms | Median: {:.2}ms",
            self.min_latency_ms, self.max_latency_ms, self.avg_latency_ms, self.median_latency_ms
        )?;
        writeln!(
            f,
            "Block TPS - Min: {:.2} | Max: {:.2} | Avg: {:.2} ({} blocks with user txns)",
            self.min_block_tps, self.max_block_tps, self.avg_block_tps, self.block_tps_stats.len()
        )?;
        writeln!(f, "========================================")?;
        Ok(())
    }
}

pub struct ResultsDumper<'a> {
    transaction_data: &'a HashMap<HashValue, Vec<TransactionExecutionResult>>,
}

impl<'a> ResultsDumper<'a> {
    pub fn new(transaction_data: &'a HashMap<HashValue, Vec<TransactionExecutionResult>>) -> Self {
        Self { transaction_data }
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
                    TransactionExecutionResult::Executed(_, _) => executed_count += 1,
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

        let min_delay = all_delays.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_delay = all_delays.iter().fold(0.0f64, |a, &b| a.max(b));
        let avg_delay = if total_txns > 0 {
            all_delays.iter().sum::<f64>() / total_txns as f64
        } else {
            0.0
        };
        let median_delay = if total_txns > 0 {
            let mut sorted = all_delays.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            if sorted.len() % 2 == 0 {
                (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
            } else {
                sorted[sorted.len() / 2]
            }
        } else {
            0.0
        };

        // Calculate TPS based on all executed times
        let tps = self.calculate_tps_from_executed();
        
        // Calculate TPS for user transactions only (those with Added events)
        let user_txn_tps = self.calculate_user_txn_tps();
        
        // Calculate per-block TPS statistics
        let block_tps_stats = self.calculate_block_tps_stats();
        
        // Calculate block TPS summary
        let block_tps_values: Vec<f64> = block_tps_stats
            .iter()
            .filter(|s| s.instant_tps.is_finite() && s.instant_tps > 0.0)
            .map(|s| s.instant_tps)
            .collect();
        
        let min_block_tps = block_tps_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_block_tps = block_tps_values.iter().fold(0.0f64, |a, &b| a.max(b));
        let avg_block_tps = if !block_tps_values.is_empty() {
            block_tps_values.iter().sum::<f64>() / block_tps_values.len() as f64
        } else {
            0.0
        };

        let duplicate_pct = if unique_txn_count > 0 {
            duplicate_exec_count as f64 / unique_txn_count as f64 * 100.0
        } else {
            0.0
        };

        BenchmarkStats {
            tps,
            user_txn_tps,
            total_executed: total_txns,
            unique_txn_count,
            duplicate_exec_count,
            duplicate_pct,
            min_latency_ms: if min_delay.is_finite() {
                min_delay
            } else {
                0.0
            },
            max_latency_ms: max_delay,
            avg_latency_ms: avg_delay,
            median_latency_ms: median_delay,
            block_tps_stats,
            min_block_tps: if min_block_tps.is_finite() { min_block_tps } else { 0.0 },
            max_block_tps,
            avg_block_tps,
        }
    }

    /// Calculate TPS based on executed transaction times
    fn calculate_tps_from_executed(&self) -> f64 {
        let fmt = "%Y-%m-%d %H:%M:%S%.3f";
        let mut all_exec_times: Vec<NaiveDateTime> = Vec::new();

        for events in self.transaction_data.values() {
            for ev in events {
                if let TransactionExecutionResult::Executed(ts, _) = ev {
                    if let Ok(t) = NaiveDateTime::parse_from_str(ts, fmt) {
                        all_exec_times.push(t);
                    }
                }
            }
        }

        if all_exec_times.len() < 2 {
            return all_exec_times.len() as f64;
        }

        all_exec_times.sort();
        let first = all_exec_times.first().unwrap();
        let last = all_exec_times.last().unwrap();
        let duration_secs = (*last - *first).num_milliseconds() as f64 / 1000.0;

        if duration_secs > 0.0 {
            all_exec_times.len() as f64 / duration_secs
        } else {
            all_exec_times.len() as f64
        }
    }

    /// Calculate TPS for user transactions only (those with Added events)
    fn calculate_user_txn_tps(&self) -> f64 {
        let fmt = "%Y-%m-%d %H:%M:%S%.3f";
        let mut user_exec_times: Vec<NaiveDateTime> = Vec::new();

        for events in self.transaction_data.values() {
            // Only count transactions that have Added event (user transactions)
            let has_added = events
                .iter()
                .any(|e| matches!(e, TransactionExecutionResult::Added(_)));

            if !has_added {
                continue;
            }

            // Collect execution times for this user transaction
            for ev in events {
                if let TransactionExecutionResult::Executed(ts, _) = ev {
                    if let Ok(t) = NaiveDateTime::parse_from_str(ts, fmt) {
                        user_exec_times.push(t);
                        break; // Only count first execution to avoid duplicates
                    }
                }
            }
        }

        if user_exec_times.len() < 2 {
            return user_exec_times.len() as f64;
        }

        user_exec_times.sort();
        let first = user_exec_times.first().unwrap();
        let last = user_exec_times.last().unwrap();
        let duration_secs = (*last - *first).num_milliseconds() as f64 / 1000.0;

        info!(
            "DEBUG: user_txn_count={}, duration={:.3}s, user_tps={:.2}",
            user_exec_times.len(),
            duration_secs,
            if duration_secs > 0.0 {
                user_exec_times.len() as f64 / duration_secs
            } else {
                0.0
            }
        );

        if duration_secs > 0.0 {
            user_exec_times.len() as f64 / duration_secs
        } else {
            user_exec_times.len() as f64
        }
    }

    /// Calculate per-block TPS statistics for user transactions
    fn calculate_block_tps_stats(&self) -> Vec<BlockTpsStats> {
        let fmt = "%Y-%m-%d %H:%M:%S%.3f";

        // Collect user transactions per block with their execution times
        // Key: block_number, Value: (user_txn_count, exec_time)
        let mut block_data: HashMap<u64, (usize, NaiveDateTime)> = HashMap::new();

        for events in self.transaction_data.values() {
            // Only count transactions that have Added event (user transactions)
            let has_added = events
                .iter()
                .any(|e| matches!(e, TransactionExecutionResult::Added(_)));

            if !has_added {
                continue;
            }

            // Find the first Executed event for this user transaction
            for ev in events {
                if let TransactionExecutionResult::Executed(ts, block_number) = ev {
                    if let Ok(t) = NaiveDateTime::parse_from_str(ts, fmt) {
                        let entry = block_data.entry(*block_number).or_insert((0, t));
                        entry.0 += 1;
                        // Keep the earliest execution time for this block
                        if t < entry.1 {
                            entry.1 = t;
                        }
                        break; // Only count first execution
                    }
                }
            }
        }

        if block_data.is_empty() {
            return Vec::new();
        }

        // Sort by block number
        let mut sorted_blocks: Vec<(u64, usize, NaiveDateTime)> = block_data
            .into_iter()
            .map(|(block_num, (count, time))| (block_num, count, time))
            .collect();
        sorted_blocks.sort_by_key(|(block_num, _, _)| *block_num);

        // Calculate per-block TPS
        let mut stats: Vec<BlockTpsStats> = Vec::new();
        let mut prev_time: Option<NaiveDateTime> = None;

        for (block_number, user_txn_count, exec_time) in sorted_blocks {
            let (time_since_last_block_ms, instant_tps) = if let Some(prev) = prev_time {
                let duration_ms = (exec_time - prev).num_milliseconds() as f64;
                let tps = if duration_ms > 0.0 {
                    user_txn_count as f64 / (duration_ms / 1000.0)
                } else {
                    0.0
                };
                (duration_ms, tps)
            } else {
                (0.0, 0.0) // First block, no previous time
            };

            stats.push(BlockTpsStats {
                block_number,
                user_txn_count,
                time_since_last_block_ms,
                instant_tps,
            });

            prev_time = Some(exec_time);
        }

        // Log per-block stats for debugging
        for stat in &stats {
            if stat.user_txn_count > 0 {
                info!(
                    "Block {}: {} user txns, {:.0}ms since last, TPS: {:.2}",
                    stat.block_number,
                    stat.user_txn_count,
                    stat.time_since_last_block_ms,
                    stat.instant_tps
                );
            }
        }

        stats
    }

    pub fn dump_results(&self) -> anyhow::Result<()> {
        // let mut file = OpenOptions::new()
        //     .write(true)
        //     .create(true)
        //     .truncate(true)
        //     .open("./transaction_results.txt")?;

        // for (transaction, results) in self.transaction_data {
        //     writeln!(
        //         file,
        //         "transaction id: {}, results: {:?}",
        //         *transaction, results
        //     )?;
        // }

        // match self.export_combined_svg("./benchmark_results.svg") {
        //     Ok(_) => (),
        //     Err(e) => {
        //         return Err(anyhow::format_err!(
        //             "failed to export benchmark results svg: {}",
        //             e
        //         ));
        //     }
        // }

        Ok(())
    }

    /// Collect execution latency for each transaction
    /// Returns: (transaction latency data list, unique transaction count, duplicate execution count)
    /// Each element is (transaction ID, Added time, latency in milliseconds)
    fn collect_executions(&self) -> (Vec<(HashValue, NaiveDateTime, f64)>, usize, usize) {
        let fmt = "%Y-%m-%d %H:%M:%S%.3f";
        let mut results: Vec<(HashValue, NaiveDateTime, f64)> = Vec::new();
        let mut unique_txn_count = 0usize;
        let mut duplicate_exec_count = 0usize;

        for (txn_id, events) in self.transaction_data.iter() {
            let mut added_times = Vec::new();
            let mut executed_times = Vec::new();

            for ev in events {
                match ev {
                    TransactionExecutionResult::Added(ts) => {
                        if let Ok(t) = NaiveDateTime::parse_from_str(ts, fmt) {
                            added_times.push(t);
                        }
                    }
                    TransactionExecutionResult::Executed(ts, _) => {
                        if let Ok(t) = NaiveDateTime::parse_from_str(ts, fmt) {
                            executed_times.push(t);
                        }
                    }
                    TransactionExecutionResult::ExecutedNotInMain(ts) => {
                        if let Ok(t) = NaiveDateTime::parse_from_str(ts, fmt) {
                            executed_times.push(t);
                        }
                    }
                    _ => {}
                }
            }

            if added_times.is_empty() {
                continue;
            }

            unique_txn_count += 1;

            let first_add = added_times.iter().min().unwrap();

            if executed_times.is_empty() {
                results.push((*txn_id, *first_add, f64::INFINITY));
                continue;
            }

            if executed_times.len() > 1 {
                duplicate_exec_count += executed_times.len() - 1;
            }

            let last_exec = executed_times.iter().max().unwrap();
            let delay = *last_exec - *first_add;
            results.push((
                *txn_id,
                *first_add,
                delay.num_milliseconds().checked_abs().unwrap_or(0) as f64,
            ));
        }

        // Sort by Added time
        results.sort_by_key(|(_, add_time, _)| *add_time);

        (results, unique_txn_count, duplicate_exec_count)
    }

    /// Collect execution latency with block number for each transaction
    /// Returns: Vec of (txn_id, block_number, latency_ms)
    fn collect_executions_with_block(&self) -> Vec<(HashValue, u64, f64)> {
        let fmt = "%Y-%m-%d %H:%M:%S%.3f";
        let mut results: Vec<(HashValue, u64, f64)> = Vec::new();

        for (txn_id, events) in self.transaction_data.iter() {
            let mut added_times = Vec::new();
            let mut executed_data: Vec<(NaiveDateTime, u64)> = Vec::new();

            for ev in events {
                match ev {
                    TransactionExecutionResult::Added(ts) => {
                        if let Ok(t) = NaiveDateTime::parse_from_str(ts, fmt) {
                            added_times.push(t);
                        }
                    }
                    TransactionExecutionResult::Executed(ts, block_num) => {
                        if let Ok(t) = NaiveDateTime::parse_from_str(ts, fmt) {
                            executed_data.push((t, *block_num));
                        }
                    }
                    _ => {}
                }
            }

            if added_times.is_empty() || executed_data.is_empty() {
                continue;
            }

            let first_add = added_times.iter().min().unwrap();
            // Find the execution with max time (last execution)
            let (last_exec_time, block_num) = executed_data
                .iter()
                .max_by_key(|(t, _)| t)
                .unwrap();

            let delay = *last_exec_time - *first_add;
            let latency_ms = delay.num_milliseconds().checked_abs().unwrap_or(0) as f64;

            results.push((*txn_id, *block_num, latency_ms));
        }

        results
    }

    /// Log the top N transactions with highest latency
    pub fn log_top_latency_transactions(&self, top_n: usize) {
        let mut executions = self.collect_executions_with_block();

        // Sort by latency descending
        executions.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N
        let top_txns: Vec<_> = executions.into_iter().take(top_n).collect();

        if top_txns.is_empty() {
            info!("No transactions with latency data found");
            return;
        }

        info!("===== Top {} Highest Latency Transactions =====", top_txns.len());
        info!("{:>4} | {:>12} | {:>8} | {}", "Rank", "Latency(ms)", "Block", "TxnID");
        info!("{}", "-".repeat(80));

        for (idx, (txn_id, block_num, latency_ms)) in top_txns.iter().enumerate() {
            info!(
                "{:>4} | {:>12.2} | {:>8} | {}",
                idx + 1,
                latency_ms,
                block_num,
                txn_id
            );
        }

        // Group by block and show block summary
        let mut block_latencies: HashMap<u64, Vec<f64>> = HashMap::new();
        for (_, block_num, latency_ms) in &top_txns {
            block_latencies
                .entry(*block_num)
                .or_default()
                .push(*latency_ms);
        }

        let mut block_summary: Vec<(u64, usize, f64, f64)> = block_latencies
            .into_iter()
            .map(|(block, latencies)| {
                let count = latencies.len();
                let max_lat = latencies.iter().fold(0.0f64, |a, &b| a.max(b));
                let avg_lat = latencies.iter().sum::<f64>() / count as f64;
                (block, count, max_lat, avg_lat)
            })
            .collect();

        // Sort by max latency descending
        block_summary.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        info!("");
        info!("===== Blocks with High Latency Transactions =====");
        info!("{:>8} | {:>5} | {:>12} | {:>12}", "Block", "Count", "MaxLat(ms)", "AvgLat(ms)");
        info!("{}", "-".repeat(50));

        for (block_num, count, max_lat, avg_lat) in block_summary {
            info!(
                "{:>8} | {:>5} | {:>12.2} | {:>12.2}",
                block_num, count, max_lat, avg_lat
            );
        }
        info!("=================================================");
    }

    fn get_user_transfer_block_stats(&self) -> Vec<(u64, usize)> {
        let mut block_counts: HashMap<u64, usize> = HashMap::new();

        for events in self.transaction_data.values() {
            let has_added = events
                .iter()
                .any(|e| matches!(e, TransactionExecutionResult::Added(_)));

            if has_added {
                for ev in events {
                    if let TransactionExecutionResult::Executed(_, block_number) = ev {
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
        executions: &[(HashValue, NaiveDateTime, f64)],
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
            .caption(
                "Transaction Latency (Added to Executed)",
                ("sans-serif", 28),
            )
            .margin(20)
            .x_label_area_size(120)
            .y_label_area_size(70)
            .build_cartesian_2d(0f64..(num_bars as f64), 0f64..max_latency)?;

        let fmt = "%H:%M:%S";
        chart
            .configure_mesh()
            .x_desc("Transaction Index (by Added Time)")
            .y_desc("Latency (ms)")
            .x_label_formatter(&|x| {
                let idx = *x as usize;
                if idx < valid_executions.len() {
                    valid_executions[idx].1.format(fmt).to_string()
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

        // Calculate statistics
        let all_delays: Vec<f64> = valid_executions.iter().map(|(_, _, l)| *l).collect();
        let total_txns = all_delays.len();
        let min_delay = all_delays.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_delay_stat = all_delays.iter().fold(0.0f64, |a, &b| a.max(b));
        let avg_delay = if total_txns > 0 {
            all_delays.iter().sum::<f64>() / total_txns as f64
        } else {
            0.0
        };
        let median_delay = if total_txns > 0 {
            let mut sorted = all_delays.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            if sorted.len() % 2 == 0 {
                (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
            } else {
                sorted[sorted.len() / 2]
            }
        } else {
            0.0
        };

        // Calculate TPS based on first to last transaction Added time
        let tps = if valid_executions.len() >= 2 {
            let first_time = valid_executions.first().map(|(_, t, _)| t);
            let last_time = valid_executions.last().map(|(_, t, _)| t);
            if let (Some(first), Some(last)) = (first_time, last_time) {
                let duration_secs = (*last - *first).num_milliseconds() as f64 / 1000.0;
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

        let stats_lines = vec![
            format!("TPS: {:.2}", tps),
            format!(
                "Total Executed: {} | Unique Txn: {} | Duplicates: {} ({:.1}%)",
                total_txns, unique_txn_count, duplicate_exec_count, duplicate_pct
            ),
            format!(
                "Latency - Min: {:.2}ms | Max: {:.2}ms | Avg: {:.2}ms | Median: {:.2}ms",
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
            if sorted.len() % 2 == 0 {
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

        let stats_lines = vec![
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
