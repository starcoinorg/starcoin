//! Simplified Agent Output - Stateless benchmark analysis
//!
//! Provides:
//! 1. Pipeline analysis with bottleneck detection
//! 2. Static optimization suggestions
//! 3. JSON output for CI comparison

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::analyzer::{BottleneckAnalyzer, PipelineAnalysis};
use crate::results::BenchmarkStats;
use starcoin_pipeline_timing::StageTiming;

/// Complete benchmark output for CI comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkOutput {
    /// Run ID (for tracking)
    pub run_id: String,
    /// Timestamp
    pub timestamp: String,
    /// Benchmark statistics
    pub stats: BenchmarkStats,
    /// Pipeline analysis with bottleneck detection
    pub pipeline_analysis: PipelineAnalysis,
    /// Optimization suggestions
    pub suggestions: Vec<OptimizationSuggestion>,
}

/// An optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub stage: String,
    pub priority: String,
    pub title: String,
    pub description: String,
    pub expected_improvement_pct: f64,
}

impl BenchmarkOutput {
    /// Process benchmark results and generate output
    pub fn from_stats(
        stats: BenchmarkStats,
        pipeline_stages: HashMap<String, StageTiming>,
    ) -> Self {
        let run_id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Analyze pipeline for bottlenecks
        let analyzer = BottleneckAnalyzer::new().with_thresholds(40.0, 70.0);
        let pipeline_analysis = analyzer.analyze(&pipeline_stages, stats.tps);

        // Generate static suggestions based on bottleneck
        let suggestions = Self::generate_suggestions(&pipeline_analysis);

        Self {
            run_id,
            timestamp,
            stats,
            pipeline_analysis,
            suggestions,
        }
    }

    fn generate_suggestions(analysis: &PipelineAnalysis) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // Find bottleneck stage
        if let Some(ref bottleneck) = analysis.primary_bottleneck {
            match bottleneck.as_str() {
                "Block Build" => {
                    suggestions.push(OptimizationSuggestion {
                        stage: "Block Build".to_string(),
                        priority: "CRITICAL".to_string(),
                        title: "Optimize transaction selection algorithm".to_string(),
                        description: "Use priority queue for gas-price based selection".to_string(),
                        expected_improvement_pct: 25.0,
                    });
                    suggestions.push(OptimizationSuggestion {
                        stage: "Block Build".to_string(),
                        priority: "MEDIUM".to_string(),
                        title: "Incremental state root computation".to_string(),
                        description: "Compute state root incrementally during block building"
                            .to_string(),
                        expected_improvement_pct: 20.0,
                    });
                }
                "TxPool Verify" => {
                    suggestions.push(OptimizationSuggestion {
                        stage: "TxPool Verify".to_string(),
                        priority: "CRITICAL".to_string(),
                        title: "Parallel transaction verification".to_string(),
                        description: "Verify transactions in parallel batches".to_string(),
                        expected_improvement_pct: 30.0,
                    });
                }
                "State Commit" => {
                    suggestions.push(OptimizationSuggestion {
                        stage: "State Commit".to_string(),
                        priority: "CRITICAL".to_string(),
                        title: "Batch state writes".to_string(),
                        description: "Group state writes to reduce I/O overhead".to_string(),
                        expected_improvement_pct: 20.0,
                    });
                }
                "VM Execute" => {
                    suggestions.push(OptimizationSuggestion {
                        stage: "VM Execute".to_string(),
                        priority: "CRITICAL".to_string(),
                        title: "Increase parallelism".to_string(),
                        description: "Tune TurboSTM concurrency level".to_string(),
                        expected_improvement_pct: 25.0,
                    });
                }
                _ => {}
            }
        }

        suggestions
    }
}

impl std::fmt::Display for BenchmarkOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "╔══════════════════════════════════════════════════════════════╗"
        )?;
        writeln!(
            f,
            "║                 BENCHMARK ANALYSIS OUTPUT                    ║"
        )?;
        writeln!(
            f,
            "╠══════════════════════════════════════════════════════════════╣"
        )?;
        writeln!(f, "║ Run ID: {}", self.run_id)?;
        writeln!(f, "║ Timestamp: {}", self.timestamp)?;
        writeln!(
            f,
            "╚══════════════════════════════════════════════════════════════╝"
        )?;
        writeln!(f)?;

        // Sample Quality and method description based on block count
        let (sample_quality, method_desc) = if self.stats.block_count >= 5 {
            ("GOOD", "exclude first/last + trimmed mean")
        } else if self.stats.block_count >= 3 {
            ("ACCEPTABLE", "remove min/max TPS + mean")
        } else {
            ("LOW", "median only (need more blocks)")
        };

        // Benchmark Results - highlight stable TPS for CI
        writeln!(f, "========== Benchmark Results ==========")?;
        writeln!(
            f,
            "Total Blocks: {} | Used for calc: {} [{}]",
            self.stats.block_count, self.stats.middle_block_count, sample_quality
        )?;
        writeln!(f)?;
        writeln!(f, "╭─────────────────────────────────────────╮")?;
        writeln!(
            f,
            "│  ★ STABLE TPS (CI Metric): {:<12.2} │",
            self.stats.stable_tps
        )?;
        writeln!(f, "│    ({})     │", method_desc)?;
        writeln!(f, "╰─────────────────────────────────────────╯")?;
        writeln!(f)?;
        writeln!(f, "--- CI Comparison Guidelines ---")?;
        writeln!(f, "Expected variance: ~25% (CV)")?;
        writeln!(f, "For reliable comparison: run 3-5 times, use median")?;
        writeln!(f, "Significant change threshold: >20% difference")?;
        writeln!(f)?;
        writeln!(f, "--- Raw TPS Data (for reference) ---")?;
        writeln!(f, "TPS (all blocks): {:.2}", self.stats.tps)?;
        writeln!(
            f,
            "Per-block TPS - Min: {:.2} | Max: {:.2} | Avg: {:.2} | Median: {:.2}",
            self.stats.block_tps_min,
            self.stats.block_tps_max,
            self.stats.block_tps_avg,
            self.stats.block_tps_median
        )?;
        writeln!(f, "Total Executed: {} txns", self.stats.total_executed)?;
        writeln!(
            f,
            "Latency - Min: {:.2}ms | Max: {:.2}ms | Avg: {:.2}ms | Median: {:.2}ms",
            self.stats.min_latency_ms,
            self.stats.max_latency_ms,
            self.stats.avg_latency_ms,
            self.stats.median_latency_ms
        )?;
        writeln!(f, "========================================")?;
        writeln!(f)?;

        // Pipeline Analysis
        writeln!(f, "========== Pipeline Analysis ==========")?;
        writeln!(
            f,
            "Total Pipeline Time: {:.2}ms",
            self.pipeline_analysis.total_time_ms
        )?;
        writeln!(
            f,
            "Observed TPS: {:.2}",
            self.pipeline_analysis.observed_tps
        )?;
        writeln!(
            f,
            "Pipeline Efficiency: {:.1}%",
            self.pipeline_analysis.efficiency * 100.0
        )?;
        writeln!(f)?;
        writeln!(f, "Stage Breakdown:")?;
        for stage in &self.pipeline_analysis.stages {
            let bottleneck_marker = if stage.is_bottleneck {
                format!(" [BOTTLENECK: {:?}]", stage.severity)
            } else {
                String::new()
            };
            writeln!(
                f,
                "  {}: {:.2}ms avg ({:.1}% of total){} | throughput: {:.2} txns/s",
                stage.stage,
                stage.avg_time_ms,
                stage.time_percentage,
                bottleneck_marker,
                stage.throughput
            )?;
        }
        if let Some(ref bottleneck) = self.pipeline_analysis.primary_bottleneck {
            writeln!(f)?;
            writeln!(f, "Primary Bottleneck: {}", bottleneck)?;
        }
        writeln!(f, "========================================")?;
        writeln!(f)?;

        // Optimization Suggestions
        if !self.suggestions.is_empty() {
            writeln!(f, "========== Optimization Suggestions ==========")?;
            for (i, suggestion) in self.suggestions.iter().enumerate() {
                writeln!(
                    f,
                    "{}. [{}] {} - {}",
                    i + 1,
                    suggestion.priority,
                    suggestion.stage,
                    suggestion.title
                )?;
                writeln!(
                    f,
                    "   Expected improvement: {:.0}%",
                    suggestion.expected_improvement_pct
                )?;
            }
            writeln!(f, "==============================================")?;
        }

        Ok(())
    }
}
