//! Strategy Tracker - Tracks optimization attempt chains and their progress
//!
//! This module manages the lifecycle of optimization strategies:
//! - Starting a new optimization attempt
//! - Tracking progress through benchmarks
//! - Recording success/failure
//! - Managing rollback decisions

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::knowledge::{OptimizationContext, OptimizationOutcome, OptimizationStrategy};
use crate::results::BenchmarkStats;

/// State of an optimization attempt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttemptState {
    /// Just created, not started
    Pending,
    /// Baseline captured, ready to apply changes
    BaselineCaptured,
    /// Changes applied, running verification
    Applied,
    /// Verification complete
    Verified,
    /// Successfully validated, ready to commit
    Validated,
    /// Committed to codebase
    Committed,
    /// Failed verification
    Failed,
    /// Rolled back
    RolledBack,
    /// Abandoned (e.g., due to timeout)
    Abandoned,
}

impl AttemptState {
    pub fn as_str(&self) -> &'static str {
        match self {
            AttemptState::Pending => "pending",
            AttemptState::BaselineCaptured => "baseline_captured",
            AttemptState::Applied => "applied",
            AttemptState::Verified => "verified",
            AttemptState::Validated => "validated",
            AttemptState::Committed => "committed",
            AttemptState::Failed => "failed",
            AttemptState::RolledBack => "rolled_back",
            AttemptState::Abandoned => "abandoned",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            AttemptState::Committed
                | AttemptState::Failed
                | AttemptState::RolledBack
                | AttemptState::Abandoned
        )
    }

    pub fn is_success(&self) -> bool {
        matches!(self, AttemptState::Committed | AttemptState::Validated)
    }
}

/// A single optimization attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationAttempt {
    /// Unique attempt ID
    pub id: String,
    /// Parent attempt (if this is a follow-up)
    pub parent_id: Option<String>,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
    /// Current state
    pub state: AttemptState,
    /// Strategy being tried
    pub strategy: OptimizationStrategy,
    /// Context when started
    pub context: OptimizationContext,
    /// Baseline stats (before optimization)
    pub baseline_stats: Option<BenchmarkStats>,
    /// Result stats (after optimization)
    pub result_stats: Option<BenchmarkStats>,
    /// Number of verification runs
    pub verification_runs: u32,
    /// Git commit before changes
    pub before_commit: Option<String>,
    /// Git commit after changes
    pub after_commit: Option<String>,
    /// State transitions log
    pub state_log: Vec<StateTransition>,
    /// Error messages (if any)
    pub errors: Vec<String>,
    /// Notes
    pub notes: Vec<String>,
}

/// A state transition event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: AttemptState,
    pub to: AttemptState,
    pub timestamp: DateTime<Utc>,
    pub reason: String,
}

impl OptimizationAttempt {
    /// Create a new attempt
    pub fn new(strategy: OptimizationStrategy, context: OptimizationContext) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: None,
            created_at: now,
            updated_at: now,
            state: AttemptState::Pending,
            strategy,
            context,
            baseline_stats: None,
            result_stats: None,
            verification_runs: 0,
            before_commit: None,
            after_commit: None,
            state_log: vec![],
            errors: vec![],
            notes: vec![],
        }
    }

    /// Create a follow-up attempt
    pub fn follow_up(
        parent_id: &str,
        strategy: OptimizationStrategy,
        context: OptimizationContext,
    ) -> Self {
        let mut attempt = Self::new(strategy, context);
        attempt.parent_id = Some(parent_id.to_string());
        attempt
    }

    /// Transition to a new state
    pub fn transition(&mut self, new_state: AttemptState, reason: &str) {
        let transition = StateTransition {
            from: self.state.clone(),
            to: new_state.clone(),
            timestamp: Utc::now(),
            reason: reason.to_string(),
        };
        self.state_log.push(transition);
        self.state = new_state;
        self.updated_at = Utc::now();
    }

    /// Record baseline stats
    pub fn record_baseline(&mut self, stats: BenchmarkStats, commit: Option<String>) {
        self.baseline_stats = Some(stats);
        self.before_commit = commit;
        self.transition(AttemptState::BaselineCaptured, "Baseline captured");
    }

    /// Record that changes were applied
    pub fn mark_applied(&mut self, commit: Option<String>) {
        self.after_commit = commit;
        self.transition(AttemptState::Applied, "Changes applied");
    }

    /// Record verification result
    pub fn record_verification(&mut self, stats: BenchmarkStats) {
        self.result_stats = Some(stats);
        self.verification_runs += 1;
        self.updated_at = Utc::now();
    }

    /// Calculate outcome
    pub fn calculate_outcome(&self) -> Option<OptimizationOutcome> {
        let baseline = self.baseline_stats.as_ref()?;
        let result = self.result_stats.as_ref()?;

        let tps_improvement_pct = if baseline.tps > 0.0 {
            (result.tps - baseline.tps) / baseline.tps * 100.0
        } else if result.tps > 0.0 {
            100.0
        } else {
            0.0
        };

        let latency_change_pct = if baseline.avg_latency_ms > 0.0 {
            (result.avg_latency_ms - baseline.avg_latency_ms) / baseline.avg_latency_ms * 100.0
        } else {
            0.0
        };

        let success = tps_improvement_pct > 0.0 || latency_change_pct < 0.0;

        let reason = if success {
            format!(
                "TPS: {:.2} → {:.2} ({:+.1}%), Latency: {:.2}ms → {:.2}ms ({:+.1}%)",
                baseline.tps,
                result.tps,
                tps_improvement_pct,
                baseline.avg_latency_ms,
                result.avg_latency_ms,
                latency_change_pct
            )
        } else {
            format!(
                "No improvement: TPS {:.2} → {:.2}, Latency {:.2}ms → {:.2}ms",
                baseline.tps, result.tps, baseline.avg_latency_ms, result.avg_latency_ms
            )
        };

        Some(OptimizationOutcome {
            success,
            result_tps: result.tps,
            tps_improvement_pct,
            latency_change_pct,
            stage_changes: HashMap::new(), // Could be enriched with stage data
            reason,
            side_effects: vec![],
        })
    }

    /// Get duration since creation
    pub fn duration_secs(&self) -> i64 {
        (self.updated_at - self.created_at).num_seconds()
    }
}

/// Strategy Tracker manages optimization attempts
pub struct StrategyTracker {
    /// Storage path
    store_path: PathBuf,
    /// Active attempts (non-terminal state)
    active_attempts: HashMap<String, OptimizationAttempt>,
    /// Completed attempts (terminal state)
    completed_attempts: Vec<OptimizationAttempt>,
    /// Current attempt ID (if any)
    current_attempt_id: Option<String>,
}

impl StrategyTracker {
    /// Create a new strategy tracker
    pub fn new<P: AsRef<Path>>(store_dir: P) -> Result<Self> {
        let store_dir = store_dir.as_ref();
        std::fs::create_dir_all(store_dir)
            .with_context(|| format!("Failed to create strategy store directory: {:?}", store_dir))?;

        let store_path = store_dir.to_path_buf();

        let mut tracker = Self {
            store_path,
            active_attempts: HashMap::new(),
            completed_attempts: Vec::new(),
            current_attempt_id: None,
        };

        tracker.load()?;

        Ok(tracker)
    }

    /// Load existing attempts from disk
    fn load(&mut self) -> Result<()> {
        let attempts_file = self.store_path.join("optimization_attempts.jsonl");
        if !attempts_file.exists() {
            return Ok(());
        }

        let file = File::open(&attempts_file)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(attempt) = serde_json::from_str::<OptimizationAttempt>(&line) {
                if attempt.state.is_terminal() {
                    self.completed_attempts.push(attempt);
                } else {
                    self.active_attempts.insert(attempt.id.clone(), attempt);
                }
            }
        }

        // Load current attempt ID
        let current_file = self.store_path.join("current_attempt.txt");
        if current_file.exists() {
            let id = std::fs::read_to_string(&current_file)?;
            let id = id.trim();
            if self.active_attempts.contains_key(id) {
                self.current_attempt_id = Some(id.to_string());
            }
        }

        Ok(())
    }

    /// Save an attempt to disk
    fn save_attempt(&self, attempt: &OptimizationAttempt) -> Result<()> {
        let attempts_file = self.store_path.join("optimization_attempts.jsonl");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&attempts_file)?;

        let json = serde_json::to_string(attempt)?;
        writeln!(file, "{}", json)?;

        Ok(())
    }

    /// Save current attempt ID
    fn save_current(&self) -> Result<()> {
        let current_file = self.store_path.join("current_attempt.txt");
        if let Some(ref id) = self.current_attempt_id {
            std::fs::write(&current_file, id)?;
        } else if current_file.exists() {
            std::fs::remove_file(&current_file)?;
        }
        Ok(())
    }

    /// Start a new optimization attempt
    pub fn start_attempt(
        &mut self,
        strategy: OptimizationStrategy,
        context: OptimizationContext,
    ) -> Result<String> {
        // Check if there's an active attempt
        if let Some(ref current_id) = self.current_attempt_id {
            if let Some(attempt) = self.active_attempts.get(current_id) {
                if !attempt.state.is_terminal() {
                    bail!(
                        "Cannot start new attempt - current attempt {} is in state '{}'",
                        current_id,
                        attempt.state.as_str()
                    );
                }
            }
        }

        let attempt = OptimizationAttempt::new(strategy, context);
        let id = attempt.id.clone();

        self.save_attempt(&attempt)?;
        self.active_attempts.insert(id.clone(), attempt);
        self.current_attempt_id = Some(id.clone());
        self.save_current()?;

        Ok(id)
    }

    /// Start a follow-up attempt
    pub fn start_follow_up(
        &mut self,
        parent_id: &str,
        strategy: OptimizationStrategy,
        context: OptimizationContext,
    ) -> Result<String> {
        if !self.completed_attempts.iter().any(|a| a.id == parent_id)
            && !self.active_attempts.contains_key(parent_id)
        {
            bail!("Parent attempt {} not found", parent_id);
        }

        let attempt = OptimizationAttempt::follow_up(parent_id, strategy, context);
        let id = attempt.id.clone();

        self.save_attempt(&attempt)?;
        self.active_attempts.insert(id.clone(), attempt);
        self.current_attempt_id = Some(id.clone());
        self.save_current()?;

        Ok(id)
    }

    /// Get current attempt
    pub fn current_attempt(&self) -> Option<&OptimizationAttempt> {
        self.current_attempt_id
            .as_ref()
            .and_then(|id| self.active_attempts.get(id))
    }

    /// Get current attempt mutably
    pub fn current_attempt_mut(&mut self) -> Option<&mut OptimizationAttempt> {
        if let Some(ref id) = self.current_attempt_id {
            self.active_attempts.get_mut(id)
        } else {
            None
        }
    }

    /// Update current attempt and save
    pub fn update_current<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut OptimizationAttempt),
    {
        if let Some(attempt) = self.current_attempt_mut() {
            f(attempt);
            let attempt_clone = attempt.clone();
            self.save_attempt(&attempt_clone)?;

            // Move to completed if terminal
            if attempt_clone.state.is_terminal() {
                if let Some(id) = self.current_attempt_id.take() {
                    if let Some(attempt) = self.active_attempts.remove(&id) {
                        self.completed_attempts.push(attempt);
                    }
                }
                self.save_current()?;
            }
        }
        Ok(())
    }

    /// Record baseline for current attempt
    pub fn record_baseline(&mut self, stats: BenchmarkStats) -> Result<()> {
        let commit = get_current_git_commit();
        self.update_current(|attempt| {
            attempt.record_baseline(stats, commit);
        })
    }

    /// Mark current attempt as applied
    pub fn mark_applied(&mut self) -> Result<()> {
        let commit = get_current_git_commit();
        self.update_current(|attempt| {
            attempt.mark_applied(commit);
        })
    }

    /// Record verification run for current attempt
    pub fn record_verification(&mut self, stats: BenchmarkStats) -> Result<()> {
        self.update_current(|attempt| {
            attempt.record_verification(stats);
        })
    }

    /// Mark current attempt as validated
    pub fn mark_validated(&mut self, reason: &str) -> Result<()> {
        self.update_current(|attempt| {
            attempt.transition(AttemptState::Validated, reason);
        })
    }

    /// Mark current attempt as committed
    pub fn mark_committed(&mut self, reason: &str) -> Result<()> {
        self.update_current(|attempt| {
            attempt.transition(AttemptState::Committed, reason);
        })
    }

    /// Mark current attempt as failed
    pub fn mark_failed(&mut self, reason: &str) -> Result<()> {
        self.update_current(|attempt| {
            attempt.errors.push(reason.to_string());
            attempt.transition(AttemptState::Failed, reason);
        })
    }

    /// Mark current attempt as rolled back
    pub fn mark_rolled_back(&mut self, reason: &str) -> Result<()> {
        self.update_current(|attempt| {
            attempt.transition(AttemptState::RolledBack, reason);
        })
    }

    /// Abandon current attempt
    pub fn abandon_current(&mut self, reason: &str) -> Result<()> {
        self.update_current(|attempt| {
            attempt.transition(AttemptState::Abandoned, reason);
        })
    }

    /// Get attempt by ID
    pub fn get_attempt(&self, id: &str) -> Option<&OptimizationAttempt> {
        self.active_attempts
            .get(id)
            .or_else(|| self.completed_attempts.iter().find(|a| a.id == id))
    }

    /// Get all active attempts
    pub fn active_attempts(&self) -> Vec<&OptimizationAttempt> {
        self.active_attempts.values().collect()
    }

    /// Get completed attempts
    pub fn completed_attempts(&self) -> &[OptimizationAttempt] {
        &self.completed_attempts
    }

    /// Get attempt chain (from root to current)
    pub fn get_attempt_chain(&self, id: &str) -> Vec<&OptimizationAttempt> {
        let mut chain = Vec::new();
        let mut current_id = Some(id.to_string());

        while let Some(id) = current_id {
            if let Some(attempt) = self.get_attempt(&id) {
                chain.push(attempt);
                current_id = attempt.parent_id.clone();
            } else {
                break;
            }
        }

        chain.reverse();
        chain
    }

    /// Get strategy summary for agent
    pub fn get_summary(&self) -> TrackerSummary {
        let active: Vec<_> = self
            .active_attempts
            .values()
            .map(|a| AttemptSummary {
                id: a.id.clone(),
                strategy: a.strategy.technique.clone(),
                state: a.state.as_str().to_string(),
                duration_secs: a.duration_secs(),
            })
            .collect();

        let recent_completed: Vec<_> = self
            .completed_attempts
            .iter()
            .rev()
            .take(5)
            .map(|a| AttemptSummary {
                id: a.id.clone(),
                strategy: a.strategy.technique.clone(),
                state: a.state.as_str().to_string(),
                duration_secs: a.duration_secs(),
            })
            .collect();

        let success_count = self
            .completed_attempts
            .iter()
            .filter(|a| a.state.is_success())
            .count();

        TrackerSummary {
            current_attempt_id: self.current_attempt_id.clone(),
            active_attempts: active,
            recent_completed,
            total_completed: self.completed_attempts.len(),
            success_count,
        }
    }
}

/// Get current git commit hash
fn get_current_git_commit() -> Option<String> {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

/// Summary of an attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptSummary {
    pub id: String,
    pub strategy: String,
    pub state: String,
    pub duration_secs: i64,
}

/// Tracker summary for agent consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerSummary {
    pub current_attempt_id: Option<String>,
    pub active_attempts: Vec<AttemptSummary>,
    pub recent_completed: Vec<AttemptSummary>,
    pub total_completed: usize,
    pub success_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::OptimizationCategory;

    #[test]
    fn test_attempt_state_transitions() {
        let strategy = OptimizationStrategy {
            target_stage: "Block Build".to_string(),
            category: OptimizationCategory::Config,
            technique: "tune_block_gas_limit".to_string(),
            parameters: HashMap::new(),
        };

        let context = OptimizationContext {
            baseline_tps: 100.0,
            primary_bottleneck: "Block Build".to_string(),
            bottleneck_severity: "Critical".to_string(),
            network: "custom".to_string(),
            account_count: 100,
            batch_user_count: 10,
            tags: vec![],
        };

        let mut attempt = OptimizationAttempt::new(strategy, context);

        assert_eq!(attempt.state, AttemptState::Pending);
        assert!(!attempt.state.is_terminal());

        attempt.transition(AttemptState::BaselineCaptured, "Test");
        assert_eq!(attempt.state, AttemptState::BaselineCaptured);
        assert_eq!(attempt.state_log.len(), 1);

        attempt.transition(AttemptState::Committed, "Test");
        assert!(attempt.state.is_terminal());
        assert!(attempt.state.is_success());
    }
}
