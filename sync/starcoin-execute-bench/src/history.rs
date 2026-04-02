//! History Store - Persists benchmark results for historical comparison and regression detection
//!
//! Stores benchmark runs in a JSON-lines format for easy querying and comparison.

use std::{
    collections::VecDeque,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::results::BenchmarkStats;
use starcoin_pipeline_timing::StageTiming;
use std::collections::HashMap;

/// A single historical benchmark run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalRun {
    /// Unique run identifier
    pub run_id: String,
    /// Timestamp of the run
    pub timestamp: DateTime<Utc>,
    /// Git commit hash (if available)
    pub git_commit: Option<String>,
    /// Git branch (if available)  
    pub git_branch: Option<String>,
    /// Benchmark configuration used
    pub config: BenchmarkConfig,
    /// Benchmark statistics
    pub stats: BenchmarkStats,
    /// Pipeline stage timings
    pub pipeline_stages: HashMap<String, StageTiming>,
    /// Optional notes/tags
    pub tags: Vec<String>,
}

/// Benchmark configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub account_count: u32,
    pub batch_user_count: usize,
    pub gas_price: u64,
    pub max_gas: u64,
    pub network: String,
}

/// History store for benchmark results
pub struct HistoryStore {
    /// Path to the history file
    history_file: PathBuf,
    /// In-memory cache of recent runs
    recent_runs: VecDeque<HistoricalRun>,
    /// Max runs to keep in memory
    max_cached_runs: usize,
}

impl HistoryStore {
    /// Create a new history store
    pub fn new<P: AsRef<Path>>(history_dir: P) -> Result<Self> {
        let history_dir = history_dir.as_ref();
        std::fs::create_dir_all(history_dir)
            .with_context(|| format!("Failed to create history directory: {:?}", history_dir))?;

        let history_file = history_dir.join("benchmark_history.jsonl");

        let mut store = Self {
            history_file,
            recent_runs: VecDeque::new(),
            max_cached_runs: 100,
        };

        // Load existing history
        store.load_recent()?;

        Ok(store)
    }

    /// Load recent runs from disk
    fn load_recent(&mut self) -> Result<()> {
        if !self.history_file.exists() {
            return Ok(());
        }

        let file = File::open(&self.history_file)
            .with_context(|| format!("Failed to open history file: {:?}", self.history_file))?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<HistoricalRun>(&line) {
                Ok(run) => {
                    self.recent_runs.push_back(run);
                    if self.recent_runs.len() > self.max_cached_runs {
                        self.recent_runs.pop_front();
                    }
                }
                Err(e) => {
                    // Log but don't fail on corrupted entries
                    eprintln!("Warning: Failed to parse history entry: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Save a new benchmark run
    pub fn save_run(&mut self, run: HistoricalRun) -> Result<()> {
        // Append to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.history_file)
            .with_context(|| format!("Failed to open history file for writing: {:?}", self.history_file))?;

        let json = serde_json::to_string(&run)?;
        writeln!(file, "{}", json)?;

        // Update cache
        self.recent_runs.push_back(run);
        if self.recent_runs.len() > self.max_cached_runs {
            self.recent_runs.pop_front();
        }

        Ok(())
    }

    /// Get the most recent run
    pub fn get_latest(&self) -> Option<&HistoricalRun> {
        self.recent_runs.back()
    }

    /// Get the N most recent runs
    pub fn get_recent(&self, n: usize) -> Vec<&HistoricalRun> {
        self.recent_runs.iter().rev().take(n).collect()
    }

    /// Get all runs within a time range
    pub fn get_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<&HistoricalRun> {
        self.recent_runs
            .iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .collect()
    }

    /// Get runs for a specific git branch
    pub fn get_by_branch(&self, branch: &str) -> Vec<&HistoricalRun> {
        self.recent_runs
            .iter()
            .filter(|r| r.git_branch.as_deref() == Some(branch))
            .collect()
    }

    /// Get baseline run (first run or tagged as baseline)
    pub fn get_baseline(&self) -> Option<&HistoricalRun> {
        // First try to find a run tagged as baseline
        self.recent_runs
            .iter()
            .find(|r| r.tags.contains(&"baseline".to_string()))
            .or_else(|| self.recent_runs.front())
    }

    /// Calculate statistics over recent runs
    pub fn calculate_trends(&self, window_size: usize) -> Option<TrendAnalysis> {
        let runs: Vec<_> = self.get_recent(window_size);
        if runs.len() < 2 {
            return None;
        }

        let tps_values: Vec<f64> = runs.iter().map(|r| r.stats.tps).collect();
        let latency_values: Vec<f64> = runs.iter().map(|r| r.stats.avg_latency_ms).collect();

        // Calculate trends (simple linear regression slope)
        let tps_trend = calculate_trend(&tps_values);
        let latency_trend = calculate_trend(&latency_values);

        // Calculate averages
        let tps_avg = tps_values.iter().sum::<f64>() / tps_values.len() as f64;
        let latency_avg = latency_values.iter().sum::<f64>() / latency_values.len() as f64;

        // Calculate standard deviation
        let tps_std = std_deviation(&tps_values, tps_avg);
        let latency_std = std_deviation(&latency_values, latency_avg);

        Some(TrendAnalysis {
            window_size: runs.len(),
            tps_trend,
            tps_avg,
            tps_std,
            latency_trend,
            latency_avg,
            latency_std,
            is_improving: tps_trend > 0.0 && latency_trend < 0.0,
            is_degrading: tps_trend < 0.0 || latency_trend > 0.0,
        })
    }

    /// Get current git info
    pub fn get_git_info() -> (Option<String>, Option<String>) {
        let commit = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        let branch = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        (commit, branch)
    }

    /// Create a new run from current benchmark results
    pub fn create_run(
        config: BenchmarkConfig,
        stats: BenchmarkStats,
        pipeline_stages: HashMap<String, StageTiming>,
        tags: Vec<String>,
    ) -> HistoricalRun {
        let (git_commit, git_branch) = Self::get_git_info();

        HistoricalRun {
            run_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            git_commit,
            git_branch,
            config,
            stats,
            pipeline_stages,
            tags,
        }
    }
}

/// Trend analysis over multiple runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub window_size: usize,
    /// TPS trend (positive = improving)
    pub tps_trend: f64,
    pub tps_avg: f64,
    pub tps_std: f64,
    /// Latency trend (negative = improving)
    pub latency_trend: f64,
    pub latency_avg: f64,
    pub latency_std: f64,
    /// Overall performance is improving
    pub is_improving: bool,
    /// Overall performance is degrading
    pub is_degrading: bool,
}

impl std::fmt::Display for TrendAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "========== Trend Analysis ({} runs) ==========", self.window_size)?;
        
        let tps_direction = if self.tps_trend > 0.0 { "↑" } else if self.tps_trend < 0.0 { "↓" } else { "→" };
        writeln!(f, "TPS: {:.2} avg (±{:.2}) {}", self.tps_avg, self.tps_std, tps_direction)?;
        
        let lat_direction = if self.latency_trend < 0.0 { "↑(better)" } else if self.latency_trend > 0.0 { "↓(worse)" } else { "→" };
        writeln!(f, "Latency: {:.2}ms avg (±{:.2}ms) {}", self.latency_avg, self.latency_std, lat_direction)?;
        
        if self.is_improving {
            writeln!(f, "Status: ✅ Performance is IMPROVING")?;
        } else if self.is_degrading {
            writeln!(f, "Status: ⚠️ Performance is DEGRADING")?;
        } else {
            writeln!(f, "Status: ➡️ Performance is STABLE")?;
        }
        
        writeln!(f, "============================================")?;
        Ok(())
    }
}

/// Calculate linear trend (slope) of values
fn calculate_trend(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let n = values.len() as f64;
    let x_mean = (n - 1.0) / 2.0;
    let y_mean = values.iter().sum::<f64>() / n;

    let mut num = 0.0;
    let mut den = 0.0;

    for (i, &y) in values.iter().enumerate() {
        let x = i as f64;
        num += (x - x_mean) * (y - y_mean);
        den += (x - x_mean).powi(2);
    }

    if den > 0.0 {
        num / den
    } else {
        0.0
    }
}

/// Calculate standard deviation
fn std_deviation(values: &[f64], mean: f64) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trend_calculation() {
        // Upward trend
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let trend = calculate_trend(&values);
        assert!(trend > 0.0);

        // Downward trend
        let values = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let trend = calculate_trend(&values);
        assert!(trend < 0.0);

        // Flat trend
        let values = vec![3.0, 3.0, 3.0, 3.0, 3.0];
        let trend = calculate_trend(&values);
        assert!((trend - 0.0).abs() < 0.001);
    }
}
