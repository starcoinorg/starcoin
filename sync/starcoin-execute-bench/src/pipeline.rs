// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline stage definitions and timing for TPS optimization agent loop.
//!
//! The Starcoin transaction processing pipeline has 4 main stages:
//! 1. TxPool Verify - Signature verification, gas checks, nonce validation
//! 2. Block Build - Transaction selection, DAG/consensus preparation  
//! 3. VM Execute - Move VM transaction execution (Block-STM parallel)
//! 4. State Commit - Merkle tree updates, DB writes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use starcoin_crypto::HashValue;

/// Pipeline stages for transaction processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelineStage {
    /// TxPool verification: sig check, gas, nonce
    TxPoolVerify,
    /// Block building: tx selection, consensus
    BlockBuild,
    /// VM execution: Move VM parallel execution
    VmExecute,
    /// State commit: merkle updates, DB writes
    StateCommit,
}

impl PipelineStage {
    pub fn name(&self) -> &'static str {
        match self {
            PipelineStage::TxPoolVerify => "TxPool Verify",
            PipelineStage::BlockBuild => "Block Build",
            PipelineStage::VmExecute => "VM Execute",
            PipelineStage::StateCommit => "State Commit",
        }
    }

    pub fn all() -> &'static [PipelineStage] {
        &[
            PipelineStage::TxPoolVerify,
            PipelineStage::BlockBuild,
            PipelineStage::VmExecute,
            PipelineStage::StateCommit,
        ]
    }
}

/// Timing data for a single stage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StageTiming {
    /// Total time spent in this stage (milliseconds)
    pub total_ms: f64,
    /// Number of samples
    pub count: u64,
    /// Minimum time (milliseconds)
    pub min_ms: f64,
    /// Maximum time (milliseconds)
    pub max_ms: f64,
    /// Average time (milliseconds)
    pub avg_ms: f64,
    /// Median time (milliseconds)
    pub median_ms: f64,
    /// P95 latency (milliseconds)
    pub p95_ms: f64,
    /// P99 latency (milliseconds)
    pub p99_ms: f64,
    /// Throughput (items per second)
    pub throughput: f64,
}

impl StageTiming {
    pub fn from_samples(samples: &[f64], item_count: u64) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let total_ms: f64 = sorted.iter().sum();
        let count = sorted.len() as u64;
        let min_ms = sorted.first().copied().unwrap_or(0.0);
        let max_ms = sorted.last().copied().unwrap_or(0.0);
        let avg_ms = total_ms / count as f64;

        let median_ms = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        let p95_idx = ((sorted.len() as f64) * 0.95) as usize;
        let p99_idx = ((sorted.len() as f64) * 0.99) as usize;
        let p95_ms = sorted.get(p95_idx.min(sorted.len() - 1)).copied().unwrap_or(0.0);
        let p99_ms = sorted.get(p99_idx.min(sorted.len() - 1)).copied().unwrap_or(0.0);

        // Calculate throughput: items per second
        let throughput = if total_ms > 0.0 {
            item_count as f64 / (total_ms / 1000.0)
        } else {
            0.0
        };

        Self {
            total_ms,
            count,
            min_ms,
            max_ms,
            avg_ms,
            median_ms,
            p95_ms,
            p99_ms,
            throughput,
        }
    }
}

/// Block-level timing data covering all pipeline stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPipelineTiming {
    pub block_number: u64,
    pub block_id: HashValue,
    pub txn_count: usize,
    /// Timestamp when block started building
    pub build_start_ms: Option<u64>,
    /// Timestamp when block finished building
    pub build_end_ms: Option<u64>,
    /// Timestamp when block started executing
    pub exec_start_ms: Option<u64>,
    /// Timestamp when block finished executing
    pub exec_end_ms: Option<u64>,
    /// Timestamp when state commit started
    pub commit_start_ms: Option<u64>,
    /// Timestamp when state commit finished
    pub commit_end_ms: Option<u64>,
}

impl BlockPipelineTiming {
    pub fn new(block_number: u64, block_id: HashValue, txn_count: usize) -> Self {
        Self {
            block_number,
            block_id,
            txn_count,
            build_start_ms: None,
            build_end_ms: None,
            exec_start_ms: None,
            exec_end_ms: None,
            commit_start_ms: None,
            commit_end_ms: None,
        }
    }

    /// Get build time in milliseconds
    pub fn build_time_ms(&self) -> Option<f64> {
        match (self.build_start_ms, self.build_end_ms) {
            (Some(start), Some(end)) if end >= start => Some((end - start) as f64),
            _ => None,
        }
    }

    /// Get execution time in milliseconds
    pub fn exec_time_ms(&self) -> Option<f64> {
        match (self.exec_start_ms, self.exec_end_ms) {
            (Some(start), Some(end)) if end >= start => Some((end - start) as f64),
            _ => None,
        }
    }

    /// Get commit time in milliseconds
    pub fn commit_time_ms(&self) -> Option<f64> {
        match (self.commit_start_ms, self.commit_end_ms) {
            (Some(start), Some(end)) if end >= start => Some((end - start) as f64),
            _ => None,
        }
    }

    /// Get total pipeline time (build + exec + commit)
    pub fn total_pipeline_ms(&self) -> Option<f64> {
        let build = self.build_time_ms().unwrap_or(0.0);
        let exec = self.exec_time_ms().unwrap_or(0.0);
        let commit = self.commit_time_ms().unwrap_or(0.0);
        if build > 0.0 || exec > 0.0 || commit > 0.0 {
            Some(build + exec + commit)
        } else {
            None
        }
    }
}

/// Collector for pipeline timing data
#[derive(Debug, Default)]
pub struct PipelineTimingCollector {
    /// Block-level timing data
    pub block_timings: HashMap<HashValue, BlockPipelineTiming>,
    /// Per-transaction verification times (txn_id -> verify_time_ms)
    pub txn_verify_times: HashMap<HashValue, f64>,
}

impl PipelineTimingCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record txpool verification time for a transaction
    pub fn record_verify_time(&mut self, txn_id: HashValue, time_ms: f64) {
        self.txn_verify_times.insert(txn_id, time_ms);
    }

    /// Get or create block timing entry
    pub fn get_or_create_block(&mut self, block_id: HashValue, block_number: u64, txn_count: usize) -> &mut BlockPipelineTiming {
        self.block_timings.entry(block_id).or_insert_with(|| {
            BlockPipelineTiming::new(block_number, block_id, txn_count)
        })
    }

    /// Calculate aggregate statistics for each stage
    pub fn calculate_stage_stats(&self) -> HashMap<PipelineStage, StageTiming> {
        let mut stats = HashMap::new();

        // TxPool Verify stats
        let verify_samples: Vec<f64> = self.txn_verify_times.values().copied().collect();
        let verify_count = verify_samples.len() as u64;
        stats.insert(
            PipelineStage::TxPoolVerify,
            StageTiming::from_samples(&verify_samples, verify_count),
        );

        // Block Build stats
        let build_samples: Vec<f64> = self.block_timings
            .values()
            .filter_map(|b| b.build_time_ms())
            .collect();
        let build_txn_count: u64 = self.block_timings.values()
            .filter(|b| b.build_time_ms().is_some())
            .map(|b| b.txn_count as u64)
            .sum();
        stats.insert(
            PipelineStage::BlockBuild,
            StageTiming::from_samples(&build_samples, build_txn_count),
        );

        // VM Execute stats
        let exec_samples: Vec<f64> = self.block_timings
            .values()
            .filter_map(|b| b.exec_time_ms())
            .collect();
        let exec_txn_count: u64 = self.block_timings.values()
            .filter(|b| b.exec_time_ms().is_some())
            .map(|b| b.txn_count as u64)
            .sum();
        stats.insert(
            PipelineStage::VmExecute,
            StageTiming::from_samples(&exec_samples, exec_txn_count),
        );

        // State Commit stats
        let commit_samples: Vec<f64> = self.block_timings
            .values()
            .filter_map(|b| b.commit_time_ms())
            .collect();
        let commit_txn_count: u64 = self.block_timings.values()
            .filter(|b| b.commit_time_ms().is_some())
            .map(|b| b.txn_count as u64)
            .sum();
        stats.insert(
            PipelineStage::StateCommit,
            StageTiming::from_samples(&commit_samples, commit_txn_count),
        );

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_timing_from_samples() {
        let samples = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let timing = StageTiming::from_samples(&samples, 100);
        
        assert_eq!(timing.count, 5);
        assert!((timing.min_ms - 10.0).abs() < 0.001);
        assert!((timing.max_ms - 50.0).abs() < 0.001);
        assert!((timing.avg_ms - 30.0).abs() < 0.001);
        assert!((timing.median_ms - 30.0).abs() < 0.001);
    }
}
