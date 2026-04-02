//! Agent Loop Controller - Orchestrates the TPS optimization feedback loop
//!
//! The agent loop coordinates:
//! 1. Running benchmarks
//! 2. Analyzing results (bottleneck detection)
//! 3. Generating optimization suggestions
//! 4. Tracking history and detecting regressions
//! 5. Providing structured output for AI agents

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::analyzer::{BottleneckAnalyzer, PipelineAnalysis};
use crate::history::{BenchmarkConfig, HistoryStore, TrendAnalysis};
use crate::knowledge::{KnowledgeBase, OptimizationContext, OptimizationRecord, Recommendation};
use crate::regression::{RegressionDetector, RegressionReport};
use crate::results::BenchmarkStats;
use crate::suggester::{OptimizationReport, OptimizationSuggester};
use starcoin_pipeline_timing::StageTiming;

/// Agent loop configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Directory for storing history
    pub history_dir: String,
    /// Directory for storing knowledge base
    pub knowledge_dir: String,
    /// Number of recent runs to consider for trends
    pub trend_window: usize,
    /// Bottleneck detection threshold (percentage)
    pub bottleneck_threshold_pct: f64,
    /// Critical bottleneck threshold (percentage)
    pub critical_threshold_pct: f64,
    /// Regression detection thresholds
    pub regression_tps_threshold_pct: f64,
    pub regression_latency_threshold_pct: f64,
    /// Whether to save runs to history
    pub save_history: bool,
    /// Whether to use knowledge base for recommendations
    pub use_knowledge_base: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            history_dir: ".benchmark_history".to_string(),
            knowledge_dir: ".optimization_knowledge".to_string(),
            trend_window: 10,
            bottleneck_threshold_pct: 40.0,
            critical_threshold_pct: 70.0,
            regression_tps_threshold_pct: 5.0,
            regression_latency_threshold_pct: 10.0,
            save_history: true,
            use_knowledge_base: true,
        }
    }
}

/// Complete agent loop output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLoopOutput {
    /// Current run ID
    pub run_id: String,
    /// Timestamp
    pub timestamp: String,
    /// Benchmark statistics
    pub stats: BenchmarkStats,
    /// Pipeline analysis with bottleneck detection
    pub pipeline_analysis: PipelineAnalysis,
    /// Optimization suggestions
    pub optimization_report: OptimizationReport,
    /// Regression report (if baseline exists)
    pub regression_report: Option<RegressionReport>,
    /// Trend analysis (if enough history)
    pub trend_analysis: Option<TrendAnalysis>,
    /// Knowledge base recommendations
    pub knowledge_recommendations: Vec<Recommendation>,
    /// Action items for the agent
    pub action_items: Vec<ActionItem>,
}

/// An action item for the agent to take
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub action_type: ActionType,
    pub priority: u8, // 1-10
    pub description: String,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    /// Investigate a specific code path
    Investigate,
    /// Tune a configuration parameter
    TuneConfig,
    /// Profile a stage for deeper analysis
    Profile,
    /// Revert recent changes
    Revert,
    /// Continue optimization loop
    Continue,
    /// Performance is good, no action needed
    NoAction,
}

/// The main Agent Loop Controller
pub struct AgentLoop {
    config: AgentConfig,
    history_store: Option<HistoryStore>,
    knowledge_base: Option<KnowledgeBase>,
    bottleneck_analyzer: BottleneckAnalyzer,
    optimization_suggester: OptimizationSuggester,
    regression_detector: RegressionDetector,
}

impl AgentLoop {
    /// Create a new agent loop with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(AgentConfig::default())
    }

    /// Create a new agent loop with custom configuration
    pub fn with_config(config: AgentConfig) -> Result<Self> {
        let history_store = if config.save_history {
            Some(
                HistoryStore::new(&config.history_dir)
                    .context("Failed to initialize history store")?,
            )
        } else {
            None
        };

        let knowledge_base = if config.use_knowledge_base {
            Some(
                KnowledgeBase::new(&config.knowledge_dir)
                    .context("Failed to initialize knowledge base")?,
            )
        } else {
            None
        };

        let bottleneck_analyzer = BottleneckAnalyzer::new()
            .with_thresholds(config.bottleneck_threshold_pct, config.critical_threshold_pct);

        let optimization_suggester = OptimizationSuggester::new();

        let regression_detector = RegressionDetector::new().with_thresholds(
            config.regression_tps_threshold_pct,
            config.regression_latency_threshold_pct,
            15.0, // Stage threshold
        );

        Ok(Self {
            config,
            history_store,
            knowledge_base,
            bottleneck_analyzer,
            optimization_suggester,
            regression_detector,
        })
    }

    /// Process benchmark results and generate agent output
    pub fn process(
        &mut self,
        bench_config: BenchmarkConfig,
        stats: BenchmarkStats,
        pipeline_stages: HashMap<String, StageTiming>,
        tags: Vec<String>,
    ) -> Result<AgentLoopOutput> {
        // Generate run ID
        let run_id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // 1. Analyze pipeline for bottlenecks
        let pipeline_analysis = self.bottleneck_analyzer.analyze(&pipeline_stages, stats.tps);

        // 2. Generate optimization suggestions
        let optimization_report = self.optimization_suggester.suggest(&pipeline_analysis);

        // 3. Check for regression against baseline
        let regression_report = self.check_regression(&stats, &pipeline_stages, &run_id);

        // 4. Calculate trends
        let trend_analysis = self.calculate_trends();

        // 5. Get knowledge base recommendations
        let knowledge_recommendations = self.get_knowledge_recommendations(&stats, &pipeline_analysis);

        // 6. Generate action items
        let action_items = self.generate_action_items(
            &pipeline_analysis,
            &optimization_report,
            &regression_report,
            &trend_analysis,
            &knowledge_recommendations,
        );

        // 7. Save to history
        if let Some(ref mut store) = self.history_store {
            let run = HistoryStore::create_run(
                bench_config,
                stats.clone(),
                pipeline_stages.clone(),
                tags,
            );
            store.save_run(run)?;
        }

        Ok(AgentLoopOutput {
            run_id,
            timestamp,
            stats,
            pipeline_analysis,
            optimization_report,
            regression_report,
            trend_analysis,
            knowledge_recommendations,
            action_items,
        })
    }

    fn get_knowledge_recommendations(
        &self,
        stats: &BenchmarkStats,
        analysis: &PipelineAnalysis,
    ) -> Vec<Recommendation> {
        if let Some(ref kb) = self.knowledge_base {
            let primary_bottleneck = analysis.primary_bottleneck.clone().unwrap_or_default();
            let severity = analysis
                .stages
                .iter()
                .find(|s| Some(&s.stage) == analysis.primary_bottleneck.as_ref())
                .map(|s| s.severity.as_str().to_string())
                .unwrap_or_else(|| "none".to_string());

            let context = OptimizationContext {
                baseline_tps: stats.tps,
                primary_bottleneck,
                bottleneck_severity: severity,
                network: "custom".to_string(),
                account_count: 0,
                batch_user_count: 0,
                tags: vec![],
            };

            kb.get_recommendations(&context)
        } else {
            Vec::new()
        }
    }

    fn check_regression(
        &self,
        stats: &BenchmarkStats,
        stages: &HashMap<String, StageTiming>,
        current_run_id: &str,
    ) -> Option<RegressionReport> {
        self.history_store
            .as_ref()
            .and_then(|store| store.get_baseline())
            .map(|baseline| {
                self.regression_detector
                    .compare(baseline, stats, stages, current_run_id)
            })
    }

    fn calculate_trends(&self) -> Option<TrendAnalysis> {
        self.history_store
            .as_ref()
            .and_then(|store| store.calculate_trends(self.config.trend_window))
    }

    fn generate_action_items(
        &self,
        pipeline: &PipelineAnalysis,
        optimization: &OptimizationReport,
        regression: &Option<RegressionReport>,
        _trends: &Option<TrendAnalysis>,
        knowledge_recommendations: &[Recommendation],
    ) -> Vec<ActionItem> {
        let mut items = Vec::new();

        // Action 0: Apply knowledge-based recommendations first (highest confidence)
        for (i, rec) in knowledge_recommendations.iter().take(2).enumerate() {
            if rec.confidence >= 0.7 {
                items.push(ActionItem {
                    action_type: ActionType::TuneConfig,
                    priority: 9 - i as u8,
                    description: format!(
                        "[Learned] {} - {:.0}% expected improvement (confidence: {:.0}%)",
                        rec.strategy.technique,
                        rec.expected_improvement_pct,
                        rec.confidence * 100.0
                    ),
                    details: [
                        ("stage".to_string(), rec.strategy.target_stage.clone()),
                        ("source".to_string(), "knowledge_base".to_string()),
                        ("reason".to_string(), rec.reason.clone()),
                    ]
                    .into_iter()
                    .collect(),
                });
            }
        }

        // Action 1: Handle regressions first
        if let Some(ref reg) = regression {
            if reg.has_regression {
                items.push(ActionItem {
                    action_type: ActionType::Investigate,
                    priority: 10,
                    description: format!(
                        "Performance regression detected ({} severity)",
                        reg.max_severity.as_str()
                    ),
                    details: [
                        ("tps_change".to_string(), format!("{:.1}%", reg.tps_comparison.change_pct)),
                        ("latency_change".to_string(), format!("{:.1}%", reg.latency_comparison.change_pct)),
                    ]
                    .into_iter()
                    .collect(),
                });

                if reg.max_severity >= crate::regression::RegressionSeverity::Major {
                    items.push(ActionItem {
                        action_type: ActionType::Revert,
                        priority: 9,
                        description: "Consider reverting recent changes".to_string(),
                        details: HashMap::new(),
                    });
                }
            }
        }

        // Action 2: Address primary bottleneck
        if let Some(ref bottleneck) = pipeline.primary_bottleneck {
            items.push(ActionItem {
                action_type: ActionType::Profile,
                priority: 8,
                description: format!("Profile {} stage - identified as primary bottleneck", bottleneck),
                details: [
                    ("stage".to_string(), bottleneck.clone()),
                    ("time_percentage".to_string(), 
                     pipeline.stages.iter()
                         .find(|s| &s.stage == bottleneck)
                         .map(|s| format!("{:.1}%", s.time_percentage))
                         .unwrap_or_default()),
                ]
                .into_iter()
                .collect(),
            });
        }

        // Action 3: Apply top optimization suggestions
        for (i, suggestion) in optimization.suggestions.iter().take(3).enumerate() {
            items.push(ActionItem {
                action_type: ActionType::TuneConfig,
                priority: 7 - i as u8,
                description: suggestion.title.clone(),
                details: [
                    ("stage".to_string(), suggestion.stage.clone()),
                    ("category".to_string(), suggestion.category.as_str().to_string()),
                    ("expected_improvement".to_string(), 
                     suggestion.expected_improvement_pct
                         .map(|p| format!("{:.0}%", p))
                         .unwrap_or_else(|| "unknown".to_string())),
                ]
                .into_iter()
                .collect(),
            });
        }

        // Action 4: Continue loop if no critical issues
        if items.is_empty() || items.iter().all(|a| a.priority < 5) {
            items.push(ActionItem {
                action_type: ActionType::Continue,
                priority: 3,
                description: "No critical issues. Continue optimization loop with next iteration.".to_string(),
                details: [
                    ("improvement_potential".to_string(), 
                     format!("{:.1}%", pipeline.improvement_potential_pct)),
                ]
                .into_iter()
                .collect(),
            });
        }

        // Sort by priority (highest first)
        items.sort_by(|a, b| b.priority.cmp(&a.priority));

        items
    }

    /// Export output as JSON
    pub fn export_json(output: &AgentLoopOutput, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(output)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Export output as JSON string
    #[allow(dead_code)]
    pub fn to_json_string(output: &AgentLoopOutput) -> Result<String> {
        Ok(serde_json::to_string_pretty(output)?)
    }

    /// Record an optimization attempt to the knowledge base
    pub fn record_optimization(&mut self, record: OptimizationRecord) -> Result<()> {
        if let Some(ref mut kb) = self.knowledge_base {
            kb.record_attempt(record)?;
        }
        Ok(())
    }

    /// Get the knowledge base (for direct access)
    pub fn knowledge_base(&self) -> Option<&KnowledgeBase> {
        self.knowledge_base.as_ref()
    }

    /// Get the knowledge base mutably
    pub fn knowledge_base_mut(&mut self) -> Option<&mut KnowledgeBase> {
        self.knowledge_base.as_mut()
    }
}

impl std::fmt::Display for AgentLoopOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "╔══════════════════════════════════════════════════════════════╗")?;
        writeln!(f, "║             TPS OPTIMIZATION AGENT LOOP OUTPUT               ║")?;
        writeln!(f, "╠══════════════════════════════════════════════════════════════╣")?;
        writeln!(f, "║ Run ID: {}", self.run_id)?;
        writeln!(f, "║ Timestamp: {}", self.timestamp)?;
        writeln!(f, "╚══════════════════════════════════════════════════════════════╝")?;
        writeln!(f)?;

        // Stats summary
        writeln!(f, "{}", self.stats)?;

        // Pipeline analysis
        writeln!(f, "{}", self.pipeline_analysis)?;

        // Regression report
        if let Some(ref reg) = self.regression_report {
            writeln!(f, "{}", reg)?;
        }

        // Trend analysis
        if let Some(ref trend) = self.trend_analysis {
            writeln!(f, "{}", trend)?;
        }

        // Action items
        writeln!(f, "========== Action Items ==========")?;
        for (i, item) in self.action_items.iter().enumerate() {
            writeln!(
                f,
                "{}. [Priority {}] {:?}: {}",
                i + 1,
                item.priority,
                item.action_type,
                item.description
            )?;
            for (k, v) in &item.details {
                writeln!(f, "   {}: {}", k, v)?;
            }
        }
        writeln!(f, "==================================")?;

        // Optimization suggestions (brief)
        writeln!(f)?;
        writeln!(f, "Top Optimization Suggestions:")?;
        for (i, suggestion) in self.optimization_report.suggestions.iter().take(5).enumerate() {
            writeln!(
                f,
                "  {}. [{}] {} - {}",
                i + 1,
                suggestion.priority.as_str().to_uppercase(),
                suggestion.stage,
                suggestion.title
            )?;
        }

        // Knowledge-based recommendations
        if !self.knowledge_recommendations.is_empty() {
            writeln!(f)?;
            writeln!(f, "Learned Recommendations (from history):")?;
            for (i, rec) in self.knowledge_recommendations.iter().take(3).enumerate() {
                writeln!(
                    f,
                    "  {}. {} - {} (confidence: {:.0}%, expected: +{:.1}%)",
                    i + 1,
                    rec.strategy.target_stage,
                    rec.strategy.technique,
                    rec.confidence * 100.0,
                    rec.expected_improvement_pct
                )?;
            }
        }

        Ok(())
    }
}

impl Default for AgentLoop {
    fn default() -> Self {
        Self::new().expect("Failed to create default AgentLoop")
    }
}
