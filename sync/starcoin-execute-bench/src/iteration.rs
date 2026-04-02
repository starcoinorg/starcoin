//! Iteration Controller - Automated optimization loop orchestration
//!
//! This module provides the high-level control loop that:
//! 1. Selects optimization strategies based on bottleneck analysis
//! 2. Applies changes (config or code suggestions)
//! 3. Runs verification benchmarks
//! 4. Validates improvements
//! 5. Commits or rolls back changes
//! 6. Records learnings to knowledge base
//! 7. Repeats until target TPS is achieved or no more improvements found

use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::analyzer::{BottleneckAnalyzer, PipelineAnalysis};
use crate::config_tuner::{ConfigChange, ConfigTuner, TuningSuggestion};
use crate::experiment::{Experiment, ExperimentFramework, ExperimentResults};
use crate::knowledge::{
    KnowledgeBase, OptimizationCategory, OptimizationContext, OptimizationOutcome,
    OptimizationRecord, OptimizationStrategy, ParameterChange, Recommendation,
};
use crate::results::BenchmarkStats;
use crate::strategy::{AttemptState, OptimizationAttempt, StrategyTracker};
use crate::suggester::{OptimizationSuggester, Suggestion};
use starcoin_pipeline_timing::StageTiming;

/// Configuration for the iteration controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationConfig {
    /// Target TPS to achieve
    pub target_tps: f64,
    /// Maximum iterations
    pub max_iterations: u32,
    /// Minimum TPS improvement to continue
    pub min_improvement_pct: f64,
    /// Number of verification runs per change
    pub verification_runs: u32,
    /// Timeout per iteration (seconds)
    pub iteration_timeout_secs: u64,
    /// Auto-commit successful changes
    pub auto_commit: bool,
    /// Auto-rollback failed changes
    pub auto_rollback: bool,
    /// Only apply config changes (no code suggestions)
    pub config_only: bool,
    /// Store path for all data
    pub store_path: String,
}

impl Default for IterationConfig {
    fn default() -> Self {
        Self {
            target_tps: 1000.0,
            max_iterations: 10,
            min_improvement_pct: 2.0,
            verification_runs: 3,
            iteration_timeout_secs: 300,
            auto_commit: false,
            auto_rollback: true,
            config_only: false,
            store_path: ".optimization_data".to_string(),
        }
    }
}

/// State of the iteration loop
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoopState {
    /// Not started
    Idle,
    /// Analyzing current performance
    Analyzing,
    /// Selecting next optimization
    SelectingStrategy,
    /// Applying changes
    ApplyingChanges,
    /// Running verification benchmarks
    Verifying,
    /// Validating results
    Validating,
    /// Committing successful changes
    Committing,
    /// Rolling back failed changes
    RollingBack,
    /// Recording learnings
    Recording,
    /// Target achieved
    TargetAchieved,
    /// No more improvements found
    NoMoreImprovements,
    /// Maximum iterations reached
    MaxIterationsReached,
    /// Failed with error
    Failed,
}

impl LoopState {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoopState::Idle => "idle",
            LoopState::Analyzing => "analyzing",
            LoopState::SelectingStrategy => "selecting_strategy",
            LoopState::ApplyingChanges => "applying_changes",
            LoopState::Verifying => "verifying",
            LoopState::Validating => "validating",
            LoopState::Committing => "committing",
            LoopState::RollingBack => "rolling_back",
            LoopState::Recording => "recording",
            LoopState::TargetAchieved => "target_achieved",
            LoopState::NoMoreImprovements => "no_more_improvements",
            LoopState::MaxIterationsReached => "max_iterations_reached",
            LoopState::Failed => "failed",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            LoopState::TargetAchieved
                | LoopState::NoMoreImprovements
                | LoopState::MaxIterationsReached
                | LoopState::Failed
        )
    }
}

/// Result of one iteration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationResult {
    /// Iteration number
    pub iteration: u32,
    /// Strategy tried
    pub strategy: OptimizationStrategy,
    /// Baseline stats
    pub baseline_stats: BenchmarkStats,
    /// Result stats
    pub result_stats: BenchmarkStats,
    /// TPS improvement percentage
    pub tps_improvement_pct: f64,
    /// Latency change percentage
    pub latency_change_pct: f64,
    /// Whether the change was successful
    pub success: bool,
    /// Whether the change was committed
    pub committed: bool,
    /// Duration of iteration
    pub duration_secs: u64,
    /// Notes
    pub notes: Vec<String>,
}

/// Summary of the optimization session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSummary {
    /// Final state
    pub final_state: String,
    /// Total iterations
    pub total_iterations: u32,
    /// Successful iterations
    pub successful_iterations: u32,
    /// Starting TPS
    pub starting_tps: f64,
    /// Final TPS
    pub final_tps: f64,
    /// Total TPS improvement percentage
    pub total_improvement_pct: f64,
    /// Strategies that worked
    pub successful_strategies: Vec<String>,
    /// Strategies that failed
    pub failed_strategies: Vec<String>,
    /// Total duration
    pub total_duration_secs: u64,
    /// Iteration results
    pub iterations: Vec<IterationResult>,
}

/// The main iteration controller
pub struct IterationController {
    config: IterationConfig,
    state: LoopState,
    iteration: u32,
    knowledge_base: KnowledgeBase,
    strategy_tracker: StrategyTracker,
    config_tuner: ConfigTuner,
    experiment_framework: ExperimentFramework,
    bottleneck_analyzer: BottleneckAnalyzer,
    optimization_suggester: OptimizationSuggester,
    /// Current baseline stats
    baseline_stats: Option<BenchmarkStats>,
    /// Starting TPS (first iteration)
    starting_tps: f64,
    /// Iteration results
    iteration_results: Vec<IterationResult>,
    /// Failed strategies (to avoid retrying)
    failed_strategies: Vec<String>,
    /// Start time
    start_time: Option<Instant>,
}

impl IterationController {
    /// Create a new iteration controller
    pub fn new(config: IterationConfig) -> Result<Self> {
        let store_path = Path::new(&config.store_path);

        let knowledge_base = KnowledgeBase::new(store_path.join("knowledge"))?;
        let strategy_tracker = StrategyTracker::new(store_path.join("strategies"))?;
        let config_tuner = ConfigTuner::new(store_path.join("config"))?;
        let experiment_framework = ExperimentFramework::new(store_path.join("experiments"))?;

        let bottleneck_analyzer = BottleneckAnalyzer::new();
        let optimization_suggester = OptimizationSuggester::new();

        Ok(Self {
            config,
            state: LoopState::Idle,
            iteration: 0,
            knowledge_base,
            strategy_tracker,
            config_tuner,
            experiment_framework,
            bottleneck_analyzer,
            optimization_suggester,
            baseline_stats: None,
            starting_tps: 0.0,
            iteration_results: Vec::new(),
            failed_strategies: Vec::new(),
            start_time: None,
        })
    }

    /// Get current state
    pub fn state(&self) -> &LoopState {
        &self.state
    }

    /// Get current iteration
    pub fn iteration(&self) -> u32 {
        self.iteration
    }

    /// Check if loop should continue
    pub fn should_continue(&self) -> bool {
        !self.state.is_terminal() && self.iteration < self.config.max_iterations
    }

    /// Start the optimization loop
    pub fn start(&mut self, initial_stats: BenchmarkStats) -> Result<()> {
        if self.state != LoopState::Idle {
            bail!("Cannot start: loop is in state {}", self.state.as_str());
        }

        self.baseline_stats = Some(initial_stats.clone());
        self.starting_tps = initial_stats.tps;
        self.start_time = Some(Instant::now());

        // Check if already at target
        if initial_stats.tps >= self.config.target_tps {
            self.state = LoopState::TargetAchieved;
            return Ok(());
        }

        // Create initial config snapshot
        self.config_tuner.create_snapshot("initial_baseline")?;

        self.state = LoopState::Analyzing;
        Ok(())
    }

    /// Analyze current performance and bottlenecks
    pub fn analyze(
        &mut self,
        stats: BenchmarkStats,
        pipeline_stages: &HashMap<String, StageTiming>,
    ) -> Result<PipelineAnalysis> {
        self.state = LoopState::Analyzing;

        let analysis = self.bottleneck_analyzer.analyze(pipeline_stages, stats.tps);

        self.baseline_stats = Some(stats);
        self.state = LoopState::SelectingStrategy;

        Ok(analysis)
    }

    /// Select next optimization strategy
    pub fn select_strategy(
        &mut self,
        analysis: &PipelineAnalysis,
    ) -> Result<Option<SelectedStrategy>> {
        self.state = LoopState::SelectingStrategy;

        // Build context for knowledge base query
        let context = self.build_context(analysis);

        // 1. Check knowledge base for learned strategies
        let kb_recommendations = self.knowledge_base.get_recommendations(&context);

        // 2. Get suggestions from analyzer
        let suggestions = self.optimization_suggester.suggest(analysis);

        // 3. Get config tuning suggestions
        let primary_bottleneck_str = analysis.primary_bottleneck.clone().unwrap_or_else(|| "None".to_string());
        let severity_str = analysis
            .stages
            .iter()
            .find(|s| Some(&s.stage) == analysis.primary_bottleneck.as_ref())
            .map(|s| s.severity.as_str().to_string())
            .unwrap_or_else(|| "none".to_string());
            
        let config_suggestions = self.config_tuner.suggest_tuning(
            &primary_bottleneck_str,
            &severity_str,
        );

        // 4. Rank and select best strategy
        let selected = self.rank_and_select(
            &kb_recommendations,
            &suggestions.suggestions,
            &config_suggestions,
        );

        if selected.is_none() {
            self.state = LoopState::NoMoreImprovements;
        }

        Ok(selected)
    }

    /// Build optimization context
    fn build_context(&self, analysis: &PipelineAnalysis) -> OptimizationContext {
        let primary_bottleneck_str = analysis.primary_bottleneck.clone().unwrap_or_else(|| "None".to_string());
        let severity_str = analysis
            .stages
            .iter()
            .find(|s| Some(&s.stage) == analysis.primary_bottleneck.as_ref())
            .map(|s| s.severity.as_str().to_string())
            .unwrap_or_else(|| "none".to_string());
            
        OptimizationContext {
            baseline_tps: self.baseline_stats.as_ref().map(|s| s.tps).unwrap_or(0.0),
            primary_bottleneck: primary_bottleneck_str,
            bottleneck_severity: severity_str,
            network: "custom".to_string(),
            account_count: 0,
            batch_user_count: 0,
            tags: vec![],
        }
    }

    /// Rank and select best strategy
    fn rank_and_select(
        &self,
        kb_recommendations: &[Recommendation],
        suggestions: &[Suggestion],
        config_suggestions: &[TuningSuggestion],
    ) -> Option<SelectedStrategy> {
        let mut candidates: Vec<StrategyCandidate> = Vec::new();

        // Add knowledge base recommendations
        for rec in kb_recommendations {
            let key = format!("{}:{}", rec.strategy.target_stage, rec.strategy.technique);
            if !self.failed_strategies.contains(&key) {
                candidates.push(StrategyCandidate {
                    strategy: rec.strategy.clone(),
                    score: rec.confidence * rec.expected_improvement_pct,
                    source: "knowledge_base".to_string(),
                    changes: Vec::new(),
                });
            }
        }

        // Add config tuning suggestions (highest priority for automated execution)
        for sug in config_suggestions {
            let key = format!("config:{}", sug.param_name);
            if !self.failed_strategies.contains(&key) {
                let strategy = OptimizationStrategy {
                    target_stage: "Config".to_string(),
                    category: OptimizationCategory::Config,
                    technique: format!("tune_{}", sug.param_name),
                    parameters: HashMap::new(),
                };

                candidates.push(StrategyCandidate {
                    strategy,
                    score: sug.confidence * sug.percentage,
                    source: "config_tuner".to_string(),
                    changes: vec![ConfigCandidate {
                        param_name: sug.param_name.clone(),
                        percentage: sug.percentage,
                    }],
                });
            }
        }

        // Add suggestor recommendations (may require code changes)
        if !self.config.config_only {
            for sug in suggestions {
                let key = format!("{}:{}", sug.stage, sug.title);
                if !self.failed_strategies.contains(&key) {
                    let strategy = OptimizationStrategy {
                        target_stage: sug.stage.clone(),
                        category: match sug.category.as_str() {
                            "config" => OptimizationCategory::Config,
                            "code" => OptimizationCategory::Code,
                            "architecture" => OptimizationCategory::Architecture,
                            _ => OptimizationCategory::Code,
                        },
                        technique: sug.title.clone(),
                        parameters: HashMap::new(),
                    };

                    candidates.push(StrategyCandidate {
                        strategy,
                        score: sug.expected_improvement_pct.unwrap_or(5.0) * 0.5, // Lower score for code changes
                        source: "suggester".to_string(),
                        changes: Vec::new(),
                    });
                }
            }
        }

        // Sort by score (descending)
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Select highest scoring
        candidates.first().map(|c| SelectedStrategy {
            strategy: c.strategy.clone(),
            source: c.source.clone(),
            expected_improvement: c.score,
            config_changes: c.changes.clone(),
        })
    }

    /// Apply selected strategy
    pub fn apply_strategy(&mut self, selected: &SelectedStrategy) -> Result<Vec<ConfigChange>> {
        self.state = LoopState::ApplyingChanges;

        // Create snapshot before changes
        self.config_tuner.create_snapshot(&format!(
            "before_iteration_{}",
            self.iteration
        ))?;

        // Start tracking this attempt
        let context = OptimizationContext {
            baseline_tps: self.baseline_stats.as_ref().map(|s| s.tps).unwrap_or(0.0),
            primary_bottleneck: selected.strategy.target_stage.clone(),
            bottleneck_severity: "Unknown".to_string(),
            network: "custom".to_string(),
            account_count: 0,
            batch_user_count: 0,
            tags: vec![format!("iteration_{}", self.iteration)],
        };

        self.strategy_tracker
            .start_attempt(selected.strategy.clone(), context)?;

        // Record baseline
        if let Some(ref stats) = self.baseline_stats {
            self.strategy_tracker.record_baseline(stats.clone())?;
        }

        // Apply config changes
        let mut applied_changes = Vec::new();

        for change in &selected.config_changes {
            self.config_tuner.queue_percentage_change(
                &change.param_name,
                change.percentage,
                &format!("Iteration {} optimization", self.iteration),
            )?;
        }

        applied_changes = self.config_tuner.apply_pending()?;

        // Mark as applied
        self.strategy_tracker.mark_applied()?;

        self.state = LoopState::Verifying;

        Ok(applied_changes)
    }

    /// Record verification result
    pub fn record_verification(&mut self, stats: BenchmarkStats) -> Result<()> {
        self.strategy_tracker.record_verification(stats)?;
        Ok(())
    }

    /// Check if verification is complete
    pub fn is_verification_complete(&self) -> bool {
        self.strategy_tracker
            .current_attempt()
            .map(|a| a.verification_runs >= self.config.verification_runs)
            .unwrap_or(false)
    }

    /// Validate results
    pub fn validate(&mut self) -> Result<ValidationResult> {
        self.state = LoopState::Validating;

        let attempt = self
            .strategy_tracker
            .current_attempt()
            .ok_or_else(|| anyhow::anyhow!("No current attempt"))?;

        let outcome = attempt
            .calculate_outcome()
            .ok_or_else(|| anyhow::anyhow!("Cannot calculate outcome"))?;

        let success = outcome.tps_improvement_pct >= self.config.min_improvement_pct;

        Ok(ValidationResult {
            success,
            tps_improvement_pct: outcome.tps_improvement_pct,
            latency_change_pct: outcome.latency_change_pct,
            reason: outcome.reason,
        })
    }

    /// Commit successful changes
    pub fn commit(&mut self, reason: &str) -> Result<()> {
        self.state = LoopState::Committing;

        self.strategy_tracker.mark_committed(reason)?;

        // Update baseline to new stats
        if let Some(stats) = self
            .strategy_tracker
            .current_attempt()
            .and_then(|a| a.result_stats.clone())
        {
            self.baseline_stats = Some(stats);
        }

        self.state = LoopState::Recording;
        Ok(())
    }

    /// Rollback failed changes
    pub fn rollback(&mut self, reason: &str) -> Result<()> {
        self.state = LoopState::RollingBack;

        // Rollback config changes
        if let Some(snapshot) = self.config_tuner.latest_snapshot() {
            let snapshot_id = snapshot.id.clone();
            self.config_tuner.rollback_to(&snapshot_id)?;
        }

        self.strategy_tracker.mark_rolled_back(reason)?;

        // Mark strategy as failed
        if let Some(attempt) = self.strategy_tracker.current_attempt() {
            let key = format!(
                "{}:{}",
                attempt.strategy.target_stage, attempt.strategy.technique
            );
            self.failed_strategies.push(key);
        }

        self.state = LoopState::Recording;
        Ok(())
    }

    /// Record learnings to knowledge base
    pub fn record_learnings(&mut self) -> Result<()> {
        self.state = LoopState::Recording;

        if let Some(attempt) = self.strategy_tracker.current_attempt() {
            let outcome = attempt.calculate_outcome().unwrap_or_else(|| OptimizationOutcome {
                success: false,
                result_tps: 0.0,
                tps_improvement_pct: 0.0,
                latency_change_pct: 0.0,
                stage_changes: HashMap::new(),
                reason: "Unknown".to_string(),
                side_effects: vec![],
            });

            let record = OptimizationRecord {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                strategy: attempt.strategy.clone(),
                context: attempt.context.clone(),
                outcome,
                before_commit: attempt.before_commit.clone(),
                after_commit: attempt.after_commit.clone(),
                notes: attempt.notes.clone(),
            };

            self.knowledge_base.record_attempt(record)?;
        }

        // Record iteration result
        if let Some(attempt) = self.strategy_tracker.current_attempt() {
            let result = IterationResult {
                iteration: self.iteration,
                strategy: attempt.strategy.clone(),
                baseline_stats: attempt.baseline_stats.clone().unwrap_or_default(),
                result_stats: attempt.result_stats.clone().unwrap_or_default(),
                tps_improvement_pct: attempt
                    .calculate_outcome()
                    .map(|o| o.tps_improvement_pct)
                    .unwrap_or(0.0),
                latency_change_pct: attempt
                    .calculate_outcome()
                    .map(|o| o.latency_change_pct)
                    .unwrap_or(0.0),
                success: attempt.state.is_success(),
                committed: attempt.state == AttemptState::Committed,
                duration_secs: attempt.duration_secs() as u64,
                notes: attempt.notes.clone(),
            };

            self.iteration_results.push(result);
        }

        // Prepare for next iteration
        self.iteration += 1;

        // Check termination conditions
        if let Some(ref stats) = self.baseline_stats {
            if stats.tps >= self.config.target_tps {
                self.state = LoopState::TargetAchieved;
                return Ok(());
            }
        }

        if self.iteration >= self.config.max_iterations {
            self.state = LoopState::MaxIterationsReached;
            return Ok(());
        }

        self.state = LoopState::Analyzing;
        Ok(())
    }

    /// Get optimization summary
    pub fn get_summary(&self) -> OptimizationSummary {
        let final_tps = self.baseline_stats.as_ref().map(|s| s.tps).unwrap_or(0.0);
        let total_improvement_pct = if self.starting_tps > 0.0 {
            (final_tps - self.starting_tps) / self.starting_tps * 100.0
        } else {
            0.0
        };

        let successful_strategies: Vec<_> = self
            .iteration_results
            .iter()
            .filter(|r| r.success)
            .map(|r| r.strategy.technique.clone())
            .collect();

        let failed_strategies: Vec<_> = self
            .iteration_results
            .iter()
            .filter(|r| !r.success)
            .map(|r| r.strategy.technique.clone())
            .collect();

        let total_duration_secs = self
            .start_time
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0);

        OptimizationSummary {
            final_state: self.state.as_str().to_string(),
            total_iterations: self.iteration,
            successful_iterations: self.iteration_results.iter().filter(|r| r.success).count()
                as u32,
            starting_tps: self.starting_tps,
            final_tps,
            total_improvement_pct,
            successful_strategies,
            failed_strategies,
            total_duration_secs,
            iterations: self.iteration_results.clone(),
        }
    }

    /// Get knowledge base
    pub fn knowledge_base(&self) -> &KnowledgeBase {
        &self.knowledge_base
    }

    /// Get strategy tracker
    pub fn strategy_tracker(&self) -> &StrategyTracker {
        &self.strategy_tracker
    }

    /// Get config tuner
    pub fn config_tuner(&self) -> &ConfigTuner {
        &self.config_tuner
    }

    /// Get experiment framework
    pub fn experiment_framework(&self) -> &ExperimentFramework {
        &self.experiment_framework
    }
}

/// A selected optimization strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectedStrategy {
    pub strategy: OptimizationStrategy,
    pub source: String,
    pub expected_improvement: f64,
    pub config_changes: Vec<ConfigCandidate>,
}

/// A config change candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigCandidate {
    pub param_name: String,
    pub percentage: f64,
}

/// Internal strategy candidate for ranking
struct StrategyCandidate {
    strategy: OptimizationStrategy,
    score: f64,
    source: String,
    changes: Vec<ConfigCandidate>,
}

/// Result of validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub success: bool,
    pub tps_improvement_pct: f64,
    pub latency_change_pct: f64,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_state_transitions() {
        assert!(!LoopState::Analyzing.is_terminal());
        assert!(LoopState::TargetAchieved.is_terminal());
        assert!(LoopState::Failed.is_terminal());
    }
}
