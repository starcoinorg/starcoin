// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Global pipeline timing collector for benchmarking starcoin transaction processing.
//!
//! This crate provides a thread-safe, globally accessible collector for recording
//! timing data from different stages of the blockchain pipeline:
//! - TxPool Verify: Time to verify transactions in the txpool
//! - Block Build: Time to assemble a block from pending transactions
//! - VM Execute: Time to execute transactions in the VM
//! - State Commit: Time to commit state changes to storage

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use std::collections::HashMap;
use std::time::Instant;

/// Pipeline stages for timing collection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelineStage {
    TxPoolVerify,
    BlockBuild,
    VmExecute,
    StateCommit,
}

impl std::fmt::Display for PipelineStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineStage::TxPoolVerify => write!(f, "TxPool Verify"),
            PipelineStage::BlockBuild => write!(f, "Block Build"),
            PipelineStage::VmExecute => write!(f, "VM Execute"),
            PipelineStage::StateCommit => write!(f, "State Commit"),
        }
    }
}

/// Timing statistics for a single stage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StageTiming {
    pub count: u64,
    pub total_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub avg_ms: f64,
    pub throughput: f64, // items per second
}

impl StageTiming {
    pub fn from_samples(samples: &[f64], item_count: u64) -> Self {
        if samples.is_empty() {
            return Self::default();
        }
        let total_ms: f64 = samples.iter().sum();
        let min_ms = samples.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_ms = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let avg_ms = total_ms / samples.len() as f64;
        let throughput = if total_ms > 0.0 {
            (item_count as f64 / total_ms) * 1000.0
        } else {
            0.0
        };
        Self {
            count: samples.len() as u64,
            total_ms,
            min_ms,
            max_ms,
            avg_ms,
            throughput,
        }
    }
}

/// Block-level timing data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockPipelineTiming {
    pub block_number: u64,
    pub block_id: HashValue,
    pub txn_count: usize,
    /// Start/end times stored as f64 milliseconds for sub-millisecond precision
    pub build_start_ms: Option<f64>,
    pub build_end_ms: Option<f64>,
    pub exec_start_ms: Option<f64>,
    pub exec_end_ms: Option<f64>,
    pub commit_start_ms: Option<f64>,
    pub commit_end_ms: Option<f64>,
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

    pub fn build_time_ms(&self) -> Option<f64> {
        match (self.build_start_ms, self.build_end_ms) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }

    pub fn exec_time_ms(&self) -> Option<f64> {
        match (self.exec_start_ms, self.exec_end_ms) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }

    pub fn commit_time_ms(&self) -> Option<f64> {
        match (self.commit_start_ms, self.commit_end_ms) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

/// Timing record with unique ID for deduplication
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TimingRecord {
    duration_ms: f64,
    item_count: u64,
}

/// Thread-safe global timing collector
#[derive(Debug)]
pub struct GlobalTimingCollector {
    /// Per-transaction verification times
    txn_verify_times: RwLock<HashMap<HashValue, f64>>,
    /// Block-level timing data
    block_timings: RwLock<HashMap<HashValue, BlockPipelineTiming>>,
    /// Generic stage timing samples (for stages without block association)
    stage_samples: RwLock<HashMap<PipelineStage, Vec<TimingRecord>>>,
    /// Enabled flag - timing is only recorded when enabled
    enabled: RwLock<bool>,
    /// Reference time for epoch milliseconds
    start_instant: Instant,
    start_epoch_ms: u64,
}

impl Default for GlobalTimingCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalTimingCollector {
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        Self {
            txn_verify_times: RwLock::new(HashMap::new()),
            block_timings: RwLock::new(HashMap::new()),
            stage_samples: RwLock::new(HashMap::new()),
            enabled: RwLock::new(false),
            start_instant: Instant::now(),
            start_epoch_ms: now,
        }
    }

    /// Enable timing collection
    pub fn enable(&self) {
        *self.enabled.write() = true;
    }

    /// Disable timing collection
    pub fn disable(&self) {
        *self.enabled.write() = false;
    }

    /// Check if timing collection is enabled
    pub fn is_enabled(&self) -> bool {
        *self.enabled.read()
    }

    /// Get current epoch milliseconds with sub-millisecond precision
    pub fn now_epoch_ms(&self) -> f64 {
        self.start_epoch_ms as f64 + self.start_instant.elapsed().as_secs_f64() * 1000.0
    }

    /// Clear all collected timing data
    pub fn clear(&self) {
        self.txn_verify_times.write().clear();
        self.block_timings.write().clear();
        self.stage_samples.write().clear();
    }

    /// Record transaction verification time
    pub fn record_txn_verify(&self, txn_id: HashValue, duration_ms: f64) {
        if !self.is_enabled() {
            return;
        }
        self.txn_verify_times.write().insert(txn_id, duration_ms);
    }

    /// Get or create block timing entry
    pub fn get_or_create_block(
        &self,
        block_id: HashValue,
        block_number: u64,
        txn_count: usize,
    ) -> BlockPipelineTiming {
        let mut timings = self.block_timings.write();
        timings
            .entry(block_id)
            .or_insert_with(|| BlockPipelineTiming::new(block_number, block_id, txn_count))
            .clone()
    }

    /// Update block timing
    pub fn update_block_timing(
        &self,
        block_id: HashValue,
        update_fn: impl FnOnce(&mut BlockPipelineTiming),
    ) {
        if !self.is_enabled() {
            return;
        }
        let mut timings = self.block_timings.write();
        if let Some(timing) = timings.get_mut(&block_id) {
            update_fn(timing);
        }
    }

    /// Record block build start time
    pub fn record_block_build_start(
        &self,
        block_id: HashValue,
        block_number: u64,
        txn_count: usize,
    ) {
        if !self.is_enabled() {
            return;
        }
        let start_ms = self.now_epoch_ms();
        let mut timings = self.block_timings.write();
        let entry = timings
            .entry(block_id)
            .or_insert_with(|| BlockPipelineTiming::new(block_number, block_id, txn_count));
        entry.build_start_ms = Some(start_ms);
    }

    /// Record block build end time
    pub fn record_block_build_end(&self, block_id: HashValue) {
        if !self.is_enabled() {
            return;
        }
        let end_ms = self.now_epoch_ms();
        let mut timings = self.block_timings.write();
        if let Some(timing) = timings.get_mut(&block_id) {
            timing.build_end_ms = Some(end_ms);
        }
    }

    /// Record VM execution start time
    pub fn record_vm_exec_start(&self, block_id: HashValue, block_number: u64, txn_count: usize) {
        if !self.is_enabled() {
            return;
        }
        let start_ms = self.now_epoch_ms();
        let mut timings = self.block_timings.write();
        let entry = timings
            .entry(block_id)
            .or_insert_with(|| BlockPipelineTiming::new(block_number, block_id, txn_count));
        entry.exec_start_ms = Some(start_ms);
    }

    /// Record VM execution end time
    pub fn record_vm_exec_end(&self, block_id: HashValue) {
        if !self.is_enabled() {
            return;
        }
        let end_ms = self.now_epoch_ms();
        let mut timings = self.block_timings.write();
        if let Some(timing) = timings.get_mut(&block_id) {
            timing.exec_end_ms = Some(end_ms);
        }
    }

    /// Record state commit start time
    pub fn record_state_commit_start(
        &self,
        block_id: HashValue,
        block_number: u64,
        txn_count: usize,
    ) {
        if !self.is_enabled() {
            return;
        }
        let start_ms = self.now_epoch_ms();
        let mut timings = self.block_timings.write();
        let entry = timings
            .entry(block_id)
            .or_insert_with(|| BlockPipelineTiming::new(block_number, block_id, txn_count));
        entry.commit_start_ms = Some(start_ms);
    }

    /// Record state commit end time
    pub fn record_state_commit_end(&self, block_id: HashValue) {
        if !self.is_enabled() {
            return;
        }
        let end_ms = self.now_epoch_ms();
        let mut timings = self.block_timings.write();
        if let Some(timing) = timings.get_mut(&block_id) {
            timing.commit_end_ms = Some(end_ms);
        }
    }

    /// Record a generic timing sample for a stage
    pub fn record_stage_sample(&self, stage: PipelineStage, duration_ms: f64, item_count: u64) {
        if !self.is_enabled() {
            return;
        }
        let mut samples = self.stage_samples.write();
        samples.entry(stage).or_default().push(TimingRecord {
            duration_ms,
            item_count,
        });
    }

    /// Calculate aggregate statistics for each stage
    pub fn calculate_stage_stats(&self) -> HashMap<PipelineStage, StageTiming> {
        let mut stats = HashMap::new();

        // TxPool Verify stats
        let verify_times = self.txn_verify_times.read();
        let verify_samples: Vec<f64> = verify_times.values().copied().collect();
        let verify_count = verify_samples.len() as u64;
        stats.insert(
            PipelineStage::TxPoolVerify,
            StageTiming::from_samples(&verify_samples, verify_count),
        );

        // Block timing stats
        let block_timings = self.block_timings.read();

        // Block Build stats
        let build_samples: Vec<f64> = block_timings
            .values()
            .filter_map(|b| b.build_time_ms())
            .collect();
        let build_txn_count: u64 = block_timings
            .values()
            .filter(|b| b.build_time_ms().is_some())
            .map(|b| b.txn_count as u64)
            .sum();
        stats.insert(
            PipelineStage::BlockBuild,
            StageTiming::from_samples(&build_samples, build_txn_count),
        );

        // VM Execute stats
        let exec_samples: Vec<f64> = block_timings
            .values()
            .filter_map(|b| b.exec_time_ms())
            .collect();
        let exec_txn_count: u64 = block_timings
            .values()
            .filter(|b| b.exec_time_ms().is_some())
            .map(|b| b.txn_count as u64)
            .sum();
        stats.insert(
            PipelineStage::VmExecute,
            StageTiming::from_samples(&exec_samples, exec_txn_count),
        );

        // State Commit stats
        let commit_samples: Vec<f64> = block_timings
            .values()
            .filter_map(|b| b.commit_time_ms())
            .collect();
        let commit_txn_count: u64 = block_timings
            .values()
            .filter(|b| b.commit_time_ms().is_some())
            .map(|b| b.txn_count as u64)
            .sum();
        stats.insert(
            PipelineStage::StateCommit,
            StageTiming::from_samples(&commit_samples, commit_txn_count),
        );

        stats
    }

    /// Get block timings
    pub fn get_block_timings(&self) -> HashMap<HashValue, BlockPipelineTiming> {
        self.block_timings.read().clone()
    }

    /// Get transaction verify times
    pub fn get_txn_verify_times(&self) -> HashMap<HashValue, f64> {
        self.txn_verify_times.read().clone()
    }

    /// Export timing data as JSON
    pub fn export_json(&self) -> String {
        let stats = self.calculate_stage_stats();
        let block_timings = self.get_block_timings();

        let output = serde_json::json!({
            "pipeline_stages": stats.into_iter().map(|(k, v)| {
                (k.to_string(), v)
            }).collect::<HashMap<_, _>>(),
            "block_timings": block_timings.values().collect::<Vec<_>>(),
            "total_blocks": block_timings.len(),
            "total_txns_verified": self.txn_verify_times.read().len(),
        });

        serde_json::to_string_pretty(&output).unwrap_or_default()
    }
}

/// Global timing collector instance
pub static GLOBAL_TIMING: Lazy<GlobalTimingCollector> = Lazy::new(GlobalTimingCollector::new);

/// Enable global timing collection
pub fn enable_timing() {
    GLOBAL_TIMING.enable();
}

/// Disable global timing collection
pub fn disable_timing() {
    GLOBAL_TIMING.disable();
}

/// Clear all collected timing data
pub fn clear_timing() {
    GLOBAL_TIMING.clear();
}

/// Get reference to the global timing collector
pub fn global_collector() -> &'static GlobalTimingCollector {
    &GLOBAL_TIMING
}

/// RAII guard for timing a code block
#[allow(dead_code)]
pub struct TimingGuard {
    stage: PipelineStage,
    block_id: Option<HashValue>,
    start: Instant,
    item_count: u64,
    committed: bool,
}

impl TimingGuard {
    /// Create a new timing guard for a pipeline stage
    pub fn new(stage: PipelineStage) -> Self {
        Self {
            stage,
            block_id: None,
            start: Instant::now(),
            item_count: 1,
            committed: false,
        }
    }

    /// Create timing guard with block association
    pub fn with_block(stage: PipelineStage, block_id: HashValue) -> Self {
        Self {
            stage,
            block_id: Some(block_id),
            start: Instant::now(),
            item_count: 1,
            committed: false,
        }
    }

    /// Set the item count for throughput calculation
    pub fn set_item_count(&mut self, count: u64) {
        self.item_count = count;
    }

    /// Manually commit the timing (to prevent double-recording on drop)
    pub fn commit(mut self) {
        self.record();
        self.committed = true;
    }

    fn record(&self) {
        let duration_ms = self.start.elapsed().as_secs_f64() * 1000.0;
        GLOBAL_TIMING.record_stage_sample(self.stage, duration_ms, self.item_count);
    }
}

impl Drop for TimingGuard {
    fn drop(&mut self) {
        if !self.committed && GLOBAL_TIMING.is_enabled() {
            self.record();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests use local collector instances to avoid parallel test interference
    /// from shared global state (enable/disable racing).

    #[test]
    fn test_timing_guard() {
        // TimingGuard uses GLOBAL_TIMING, so we just verify it doesn't panic
        enable_timing();
        clear_timing();

        {
            let _guard = TimingGuard::new(PipelineStage::TxPoolVerify);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // Note: stage_samples are separate from txn_verify_times, so TxPoolVerify won't have samples here
        // The TimingGuard records to stage_samples, not txn_verify_times
        disable_timing();
    }

    #[test]
    fn test_block_timing() {
        let collector = GlobalTimingCollector::new();
        collector.enable();

        let block_id = HashValue::random();
        collector.record_block_build_start(block_id, 1, 10);
        std::thread::sleep(std::time::Duration::from_millis(5));
        collector.record_block_build_end(block_id);

        collector.record_vm_exec_start(block_id, 1, 10);
        std::thread::sleep(std::time::Duration::from_millis(3));
        collector.record_vm_exec_end(block_id);

        let stats = collector.calculate_stage_stats();
        assert!(stats[&PipelineStage::BlockBuild].count > 0);
        assert!(stats[&PipelineStage::VmExecute].count > 0);
    }

    #[test]
    fn test_txn_verify() {
        let collector = GlobalTimingCollector::new();
        collector.enable();

        let txn_id = HashValue::random();
        collector.record_txn_verify(txn_id, 0.5);

        let stats = collector.calculate_stage_stats();
        assert_eq!(stats[&PipelineStage::TxPoolVerify].count, 1);
    }

    #[test]
    fn test_disabled_collector_records_nothing() {
        let collector = GlobalTimingCollector::new();
        // Don't enable - should be disabled by default

        let block_id = HashValue::random();
        collector.record_block_build_start(block_id, 1, 10);
        collector.record_block_build_end(block_id);
        collector.record_vm_exec_start(block_id, 1, 10);
        collector.record_vm_exec_end(block_id);
        collector.record_state_commit_start(block_id, 1, 10);
        collector.record_state_commit_end(block_id);
        collector.record_txn_verify(HashValue::random(), 1.0);

        let stats = collector.calculate_stage_stats();
        assert_eq!(stats[&PipelineStage::BlockBuild].count, 0);
        assert_eq!(stats[&PipelineStage::VmExecute].count, 0);
        assert_eq!(stats[&PipelineStage::StateCommit].count, 0);
        assert_eq!(stats[&PipelineStage::TxPoolVerify].count, 0);
    }
}
