//! Optimization Knowledge Base - Learns from historical optimization attempts
//!
//! This module records which optimizations worked in what contexts,
//! building up a knowledge base that helps the agent make better decisions.

#![allow(dead_code)]

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An optimization attempt record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecord {
    /// Unique record ID
    pub id: String,
    /// When the optimization was attempted
    pub timestamp: DateTime<Utc>,
    /// The strategy used
    pub strategy: OptimizationStrategy,
    /// Context/conditions when optimization was applied
    pub context: OptimizationContext,
    /// Outcome of the optimization
    pub outcome: OptimizationOutcome,
    /// Git commit before optimization
    pub before_commit: Option<String>,
    /// Git commit after optimization
    pub after_commit: Option<String>,
    /// Notes/observations
    pub notes: Vec<String>,
}

/// The optimization strategy that was applied
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OptimizationStrategy {
    /// Target stage (e.g., "Block Build", "VM Execute")
    pub target_stage: String,
    /// Category (Config, Code, Architecture, Algorithm)
    pub category: OptimizationCategory,
    /// Specific technique used
    pub technique: String,
    /// Parameters adjusted (for config changes)
    pub parameters: HashMap<String, ParameterChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OptimizationCategory {
    Config,
    Code,
    Architecture,
    Algorithm,
    Parallelism,
    Caching,
    IO,
}

impl OptimizationCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            OptimizationCategory::Config => "Config",
            OptimizationCategory::Code => "Code",
            OptimizationCategory::Architecture => "Architecture",
            OptimizationCategory::Algorithm => "Algorithm",
            OptimizationCategory::Parallelism => "Parallelism",
            OptimizationCategory::Caching => "Caching",
            OptimizationCategory::IO => "IO",
        }
    }
}

/// A parameter change record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ParameterChange {
    pub before: String,
    pub after: String,
}

/// Context/conditions when the optimization was applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationContext {
    /// Baseline TPS before optimization
    pub baseline_tps: f64,
    /// Primary bottleneck at the time
    pub primary_bottleneck: String,
    /// Bottleneck severity
    pub bottleneck_severity: String,
    /// Network/environment
    pub network: String,
    /// Account count in benchmark
    pub account_count: u32,
    /// Batch user count
    pub batch_user_count: usize,
    /// Additional context tags
    pub tags: Vec<String>,
}

/// Outcome of the optimization attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOutcome {
    /// Whether the optimization was successful
    pub success: bool,
    /// TPS after optimization
    pub result_tps: f64,
    /// TPS improvement percentage
    pub tps_improvement_pct: f64,
    /// Latency change percentage (negative = improvement)
    pub latency_change_pct: f64,
    /// Per-stage time changes
    pub stage_changes: HashMap<String, f64>,
    /// Why it succeeded or failed
    pub reason: String,
    /// Side effects observed
    pub side_effects: Vec<String>,
}

impl OptimizationOutcome {
    pub fn is_significant_improvement(&self) -> bool {
        self.success && self.tps_improvement_pct >= 5.0
    }

    pub fn is_major_improvement(&self) -> bool {
        self.success && self.tps_improvement_pct >= 20.0
    }
}

/// A learned rule/insight from optimization history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedInsight {
    /// Unique insight ID
    pub id: String,
    /// Insight description
    pub description: String,
    /// Applicable context pattern
    pub context_pattern: InsightContextPattern,
    /// Recommended strategy
    pub recommended_strategy: OptimizationStrategy,
    /// Confidence score (0-1)
    pub confidence: f64,
    /// Number of times validated
    pub validation_count: u32,
    /// Success rate
    pub success_rate: f64,
    /// Average improvement when successful
    pub avg_improvement_pct: f64,
}

/// Pattern for matching contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightContextPattern {
    /// Match bottleneck stage (optional)
    pub bottleneck_stage: Option<String>,
    /// Match bottleneck severity (optional)
    pub min_severity: Option<String>,
    /// Match TPS range (optional)
    pub tps_range: Option<(f64, f64)>,
    /// Required tags (optional)
    pub required_tags: Vec<String>,
}

impl InsightContextPattern {
    pub fn matches(&self, context: &OptimizationContext) -> bool {
        // Check bottleneck stage
        if let Some(ref stage) = self.bottleneck_stage {
            if &context.primary_bottleneck != stage {
                return false;
            }
        }

        // Check TPS range
        if let Some((min, max)) = self.tps_range {
            if context.baseline_tps < min || context.baseline_tps > max {
                return false;
            }
        }

        // Check required tags
        for tag in &self.required_tags {
            if !context.tags.contains(tag) {
                return false;
            }
        }

        true
    }
}

/// The Knowledge Base
pub struct KnowledgeBase {
    /// Path to knowledge store
    store_path: PathBuf,
    /// Optimization records
    records: Vec<OptimizationRecord>,
    /// Learned insights
    insights: Vec<LearnedInsight>,
    /// Strategy effectiveness cache
    strategy_stats: HashMap<String, StrategyStats>,
}

/// Statistics about a strategy's effectiveness
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategyStats {
    pub total_attempts: u32,
    pub successful_attempts: u32,
    pub success_rate: f64,
    pub avg_improvement_pct: f64,
    pub max_improvement_pct: f64,
    pub contexts_worked: Vec<String>,
    pub contexts_failed: Vec<String>,
}

impl KnowledgeBase {
    /// Create a new knowledge base
    pub fn new<P: AsRef<Path>>(store_dir: P) -> Result<Self> {
        let store_dir = store_dir.as_ref();
        std::fs::create_dir_all(store_dir)
            .with_context(|| format!("Failed to create knowledge store directory: {:?}", store_dir))?;

        let store_path = store_dir.to_path_buf();

        let mut kb = Self {
            store_path,
            records: Vec::new(),
            insights: Vec::new(),
            strategy_stats: HashMap::new(),
        };

        kb.load()?;
        kb.rebuild_stats();

        Ok(kb)
    }

    /// Load existing knowledge from disk
    fn load(&mut self) -> Result<()> {
        // Load records
        let records_file = self.store_path.join("optimization_records.jsonl");
        if records_file.exists() {
            let file = File::open(&records_file)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(record) = serde_json::from_str::<OptimizationRecord>(&line) {
                    self.records.push(record);
                }
            }
        }

        // Load insights
        let insights_file = self.store_path.join("learned_insights.json");
        if insights_file.exists() {
            let content = std::fs::read_to_string(&insights_file)?;
            self.insights = serde_json::from_str(&content).unwrap_or_default();
        }

        Ok(())
    }

    /// Rebuild statistics from records
    fn rebuild_stats(&mut self) {
        self.strategy_stats.clear();

        for record in &self.records {
            let key = format!(
                "{}:{}:{}",
                record.strategy.target_stage,
                record.strategy.category.as_str(),
                record.strategy.technique
            );

            let stats = self.strategy_stats.entry(key).or_default();
            stats.total_attempts += 1;

            if record.outcome.success {
                stats.successful_attempts += 1;
                stats.avg_improvement_pct = (stats.avg_improvement_pct
                    * (stats.successful_attempts - 1) as f64
                    + record.outcome.tps_improvement_pct)
                    / stats.successful_attempts as f64;

                if record.outcome.tps_improvement_pct > stats.max_improvement_pct {
                    stats.max_improvement_pct = record.outcome.tps_improvement_pct;
                }

                stats.contexts_worked.push(record.context.primary_bottleneck.clone());
            } else {
                stats.contexts_failed.push(record.context.primary_bottleneck.clone());
            }

            stats.success_rate =
                stats.successful_attempts as f64 / stats.total_attempts as f64;
        }
    }

    /// Record a new optimization attempt
    pub fn record_attempt(&mut self, record: OptimizationRecord) -> Result<()> {
        // Append to file
        let records_file = self.store_path.join("optimization_records.jsonl");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&records_file)?;

        let json = serde_json::to_string(&record)?;
        writeln!(file, "{}", json)?;

        // Update in-memory
        self.records.push(record);
        self.rebuild_stats();

        // Try to derive new insights
        self.derive_insights()?;

        Ok(())
    }

    /// Derive insights from accumulated data
    fn derive_insights(&mut self) -> Result<()> {
        // Group records by strategy
        let mut by_strategy: HashMap<String, Vec<&OptimizationRecord>> = HashMap::new();
        for record in &self.records {
            let key = format!(
                "{}:{}:{}",
                record.strategy.target_stage,
                record.strategy.category.as_str(),
                record.strategy.technique
            );
            by_strategy.entry(key).or_default().push(record);
        }

        // Generate insights for strategies with enough data
        let mut new_insights = Vec::new();
        for (_key, records) in by_strategy {
            if records.len() < 3 {
                continue; // Need at least 3 data points
            }

            let successful: Vec<_> = records.iter().filter(|r| r.outcome.success).collect();
            let success_rate = successful.len() as f64 / records.len() as f64;

            if success_rate >= 0.6 && !successful.is_empty() {
                // Good strategy - derive an insight
                let first_success = successful[0];
                let avg_improvement: f64 = successful
                    .iter()
                    .map(|r| r.outcome.tps_improvement_pct)
                    .sum::<f64>()
                    / successful.len() as f64;

                // Find common context pattern
                let bottleneck_stage = if successful
                    .iter()
                    .all(|r| r.context.primary_bottleneck == first_success.context.primary_bottleneck)
                {
                    Some(first_success.context.primary_bottleneck.clone())
                } else {
                    None
                };

                let insight = LearnedInsight {
                    id: uuid::Uuid::new_v4().to_string(),
                    description: format!(
                        "{} optimization for {} stage ({:.0}% success rate, avg {:.1}% improvement)",
                        first_success.strategy.technique,
                        first_success.strategy.target_stage,
                        success_rate * 100.0,
                        avg_improvement
                    ),
                    context_pattern: InsightContextPattern {
                        bottleneck_stage,
                        min_severity: None,
                        tps_range: None,
                        required_tags: vec![],
                    },
                    recommended_strategy: first_success.strategy.clone(),
                    confidence: success_rate,
                    validation_count: successful.len() as u32,
                    success_rate,
                    avg_improvement_pct: avg_improvement,
                };

                // Check if we already have this insight
                let exists = self.insights.iter().any(|i| {
                    i.recommended_strategy.technique == insight.recommended_strategy.technique
                        && i.recommended_strategy.target_stage
                            == insight.recommended_strategy.target_stage
                });

                if !exists {
                    new_insights.push(insight);
                }
            }
        }

        // Add new insights
        if !new_insights.is_empty() {
            self.insights.extend(new_insights);
            self.save_insights()?;
        }

        Ok(())
    }

    /// Save insights to disk
    fn save_insights(&self) -> Result<()> {
        let insights_file = self.store_path.join("learned_insights.json");
        let json = serde_json::to_string_pretty(&self.insights)?;
        std::fs::write(&insights_file, json)?;
        Ok(())
    }

    /// Get recommendations for a given context
    pub fn get_recommendations(&self, context: &OptimizationContext) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // Check insights
        for insight in &self.insights {
            if insight.context_pattern.matches(context) && insight.confidence >= 0.5 {
                recommendations.push(Recommendation {
                    source: RecommendationSource::LearnedInsight,
                    strategy: insight.recommended_strategy.clone(),
                    confidence: insight.confidence,
                    expected_improvement_pct: insight.avg_improvement_pct,
                    reason: insight.description.clone(),
                    historical_success_rate: Some(insight.success_rate),
                });
            }
        }

        // Sort by confidence * expected improvement
        recommendations.sort_by(|a, b| {
            let score_a = a.confidence * a.expected_improvement_pct;
            let score_b = b.confidence * b.expected_improvement_pct;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        recommendations
    }

    /// Get strategy statistics
    pub fn get_strategy_stats(&self, stage: &str, technique: &str) -> Option<&StrategyStats> {
        for (key, stats) in &self.strategy_stats {
            if key.contains(stage) && key.contains(technique) {
                return Some(stats);
            }
        }
        None
    }

    /// Get all records
    pub fn get_records(&self) -> &[OptimizationRecord] {
        &self.records
    }

    /// Get all insights
    pub fn get_insights(&self) -> &[LearnedInsight] {
        &self.insights
    }

    /// Get recent successful optimizations
    pub fn get_recent_successes(&self, limit: usize) -> Vec<&OptimizationRecord> {
        self.records
            .iter()
            .rev()
            .filter(|r| r.outcome.success)
            .take(limit)
            .collect()
    }

    /// Export knowledge summary for agent consumption
    pub fn export_summary(&self) -> KnowledgeSummary {
        KnowledgeSummary {
            total_records: self.records.len(),
            total_insights: self.insights.len(),
            top_strategies: self
                .strategy_stats
                .iter()
                .filter(|(_, s)| s.success_rate >= 0.5 && s.total_attempts >= 2)
                .map(|(k, s)| (k.clone(), s.clone()))
                .collect(),
            recent_successes: self
                .get_recent_successes(5)
                .into_iter()
                .map(|r| r.strategy.technique.clone())
                .collect(),
        }
    }
}

/// A recommendation from the knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub source: RecommendationSource,
    pub strategy: OptimizationStrategy,
    pub confidence: f64,
    pub expected_improvement_pct: f64,
    pub reason: String,
    pub historical_success_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationSource {
    LearnedInsight,
    StrategyStats,
    SimilarContext,
}

/// Summary for agent consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSummary {
    pub total_records: usize,
    pub total_insights: usize,
    pub top_strategies: HashMap<String, StrategyStats>,
    pub recent_successes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_pattern_matching() {
        let pattern = InsightContextPattern {
            bottleneck_stage: Some("Block Build".to_string()),
            min_severity: None,
            tps_range: Some((0.0, 100.0)),
            required_tags: vec![],
        };

        let context = OptimizationContext {
            baseline_tps: 50.0,
            primary_bottleneck: "Block Build".to_string(),
            bottleneck_severity: "Critical".to_string(),
            network: "custom".to_string(),
            account_count: 100,
            batch_user_count: 10,
            tags: vec![],
        };

        assert!(pattern.matches(&context));

        let context2 = OptimizationContext {
            primary_bottleneck: "VM Execute".to_string(),
            ..context.clone()
        };
        assert!(!pattern.matches(&context2));
    }
}
