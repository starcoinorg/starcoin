//! Bottleneck Analyzer - Analyzes pipeline timing data to identify performance bottlenecks
//!
//! This module provides analysis of the 4 pipeline stages:
//! 1. TxPool Verify - Transaction verification in txpool
//! 2. Block Build - Block template creation
//! 3. VM Execute - Transaction execution in VM
//! 4. State Commit - State persistence to storage

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use starcoin_pipeline_timing::{PipelineStage, StageTiming};

/// Bottleneck severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    /// No bottleneck detected
    None,
    /// Minor bottleneck - stage takes more time than others but not critical
    Minor,
    /// Moderate bottleneck - stage is significantly slower
    Moderate,
    /// Severe bottleneck - stage dominates total time
    Severe,
    /// Critical bottleneck - stage is the clear limiting factor
    Critical,
}

impl BottleneckSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            BottleneckSeverity::None => "none",
            BottleneckSeverity::Minor => "minor",
            BottleneckSeverity::Moderate => "moderate",
            BottleneckSeverity::Severe => "severe",
            BottleneckSeverity::Critical => "critical",
        }
    }
}

/// Analysis result for a single pipeline stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageAnalysis {
    pub stage: String,
    pub avg_time_ms: f64,
    pub total_time_ms: f64,
    pub count: usize,
    pub throughput: f64,
    /// Percentage of total pipeline time
    pub time_percentage: f64,
    /// Is this stage a bottleneck?
    pub is_bottleneck: bool,
    /// Bottleneck severity
    pub severity: BottleneckSeverity,
}

/// Overall pipeline analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineAnalysis {
    /// Analysis per stage
    pub stages: Vec<StageAnalysis>,
    /// Primary bottleneck stage (if any)
    pub primary_bottleneck: Option<String>,
    /// Secondary bottlenecks
    pub secondary_bottlenecks: Vec<String>,
    /// Overall pipeline efficiency (0.0 - 1.0)
    pub efficiency: f64,
    /// Total wall-clock time for entire pipeline
    pub total_time_ms: f64,
    /// Theoretical max TPS based on bottleneck
    pub theoretical_max_tps: f64,
    /// Current observed TPS
    pub observed_tps: f64,
    /// Improvement potential percentage
    pub improvement_potential_pct: f64,
}

/// Bottleneck Analyzer
pub struct BottleneckAnalyzer {
    /// Threshold for considering a stage a bottleneck (percentage of total time)
    bottleneck_threshold_pct: f64,
    /// Threshold for critical bottleneck
    critical_threshold_pct: f64,
}

impl Default for BottleneckAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl BottleneckAnalyzer {
    pub fn new() -> Self {
        Self {
            bottleneck_threshold_pct: 40.0, // Stage taking >40% of time is a bottleneck
            critical_threshold_pct: 70.0,   // Stage taking >70% is critical
        }
    }

    /// Configure bottleneck thresholds
    pub fn with_thresholds(mut self, bottleneck_pct: f64, critical_pct: f64) -> Self {
        self.bottleneck_threshold_pct = bottleneck_pct;
        self.critical_threshold_pct = critical_pct;
        self
    }

    /// Analyze pipeline timing data to identify bottlenecks
    pub fn analyze(
        &self,
        stage_timings: &HashMap<String, StageTiming>,
        observed_tps: f64,
    ) -> PipelineAnalysis {
        // Calculate total time across all stages
        let total_time: f64 = stage_timings.values().map(|s| s.total_ms).sum();

        if total_time == 0.0 {
            return PipelineAnalysis {
                stages: vec![],
                primary_bottleneck: None,
                secondary_bottlenecks: vec![],
                efficiency: 0.0,
                total_time_ms: 0.0,
                theoretical_max_tps: 0.0,
                observed_tps,
                improvement_potential_pct: 0.0,
            };
        }

        // Analyze each stage
        let mut stage_analyses: Vec<StageAnalysis> = stage_timings
            .iter()
            .map(|(name, timing)| {
                let time_pct = (timing.total_ms / total_time) * 100.0;
                let severity = self.calculate_severity(time_pct);
                let is_bottleneck = time_pct >= self.bottleneck_threshold_pct;

                StageAnalysis {
                    stage: name.clone(),
                    avg_time_ms: timing.avg_ms,
                    total_time_ms: timing.total_ms,
                    count: timing.count as usize,
                    throughput: timing.throughput,
                    time_percentage: time_pct,
                    is_bottleneck,
                    severity,
                }
            })
            .collect();

        // Sort by time percentage (descending) to find bottlenecks
        stage_analyses.sort_by(|a, b| {
            b.time_percentage
                .partial_cmp(&a.time_percentage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Identify primary and secondary bottlenecks
        let primary_bottleneck = stage_analyses
            .iter()
            .find(|s| s.is_bottleneck)
            .map(|s| s.stage.clone());

        let secondary_bottlenecks: Vec<String> = stage_analyses
            .iter()
            .skip(1) // Skip primary
            .filter(|s| s.severity >= BottleneckSeverity::Minor)
            .map(|s| s.stage.clone())
            .collect();

        // Calculate efficiency (how balanced the pipeline is)
        // Perfect efficiency = all stages take equal time (25% each for 4 stages)
        let ideal_pct = 100.0 / stage_analyses.len() as f64;
        let variance: f64 = stage_analyses
            .iter()
            .map(|s| (s.time_percentage - ideal_pct).powi(2))
            .sum::<f64>()
            / stage_analyses.len() as f64;
        let efficiency = 1.0 - (variance.sqrt() / 100.0).min(1.0);

        // Calculate theoretical max TPS
        // Theoretical max = 1000 / (slowest_stage_avg_ms) if stages were perfectly parallel
        let slowest_stage_avg = stage_analyses
            .iter()
            .map(|s| s.avg_time_ms)
            .fold(0.0f64, |a, b| a.max(b));
        let theoretical_max_tps = if slowest_stage_avg > 0.0 {
            1000.0 / slowest_stage_avg
        } else {
            0.0
        };

        // Improvement potential
        let improvement_potential_pct = if observed_tps > 0.0 && theoretical_max_tps > observed_tps
        {
            ((theoretical_max_tps - observed_tps) / observed_tps) * 100.0
        } else {
            0.0
        };

        PipelineAnalysis {
            stages: stage_analyses,
            primary_bottleneck,
            secondary_bottlenecks,
            efficiency,
            total_time_ms: total_time,
            theoretical_max_tps,
            observed_tps,
            improvement_potential_pct,
        }
    }

    /// Calculate bottleneck severity based on time percentage
    fn calculate_severity(&self, time_pct: f64) -> BottleneckSeverity {
        if time_pct >= self.critical_threshold_pct {
            BottleneckSeverity::Critical
        } else if time_pct >= self.bottleneck_threshold_pct + 15.0 {
            BottleneckSeverity::Severe
        } else if time_pct >= self.bottleneck_threshold_pct {
            BottleneckSeverity::Moderate
        } else if time_pct >= 25.0 {
            // More than equal share
            BottleneckSeverity::Minor
        } else {
            BottleneckSeverity::None
        }
    }
}

impl std::fmt::Display for PipelineAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "========== Pipeline Analysis ==========")?;
        writeln!(f, "Total Pipeline Time: {:.2}ms", self.total_time_ms)?;
        writeln!(f, "Observed TPS: {:.2}", self.observed_tps)?;
        writeln!(f, "Theoretical Max TPS: {:.2}", self.theoretical_max_tps)?;
        writeln!(
            f,
            "Improvement Potential: {:.1}%",
            self.improvement_potential_pct
        )?;
        writeln!(f, "Pipeline Efficiency: {:.1}%", self.efficiency * 100.0)?;
        writeln!(f)?;

        writeln!(f, "Stage Breakdown:")?;
        for stage in &self.stages {
            let bottleneck_marker = if stage.is_bottleneck {
                format!(" [BOTTLENECK: {}]", stage.severity.as_str().to_uppercase())
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

        if let Some(ref primary) = self.primary_bottleneck {
            writeln!(f)?;
            writeln!(f, "Primary Bottleneck: {}", primary)?;
        }

        if !self.secondary_bottlenecks.is_empty() {
            writeln!(
                f,
                "Secondary Bottlenecks: {}",
                self.secondary_bottlenecks.join(", ")
            )?;
        }

        writeln!(f, "========================================")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bottleneck_detection() {
        let mut timings = HashMap::new();
        timings.insert(
            "TxPool Verify".to_string(),
            StageTiming {
                count: 100,
                total_ms: 100.0,
                min_ms: 0.5,
                max_ms: 2.0,
                avg_ms: 1.0,
                throughput: 1000.0,
            },
        );
        timings.insert(
            "Block Build".to_string(),
            StageTiming {
                count: 10,
                total_ms: 500.0, // This should be the bottleneck
                min_ms: 40.0,
                max_ms: 60.0,
                avg_ms: 50.0,
                throughput: 20.0,
            },
        );
        timings.insert(
            "VM Execute".to_string(),
            StageTiming {
                count: 10,
                total_ms: 200.0,
                min_ms: 15.0,
                max_ms: 25.0,
                avg_ms: 20.0,
                throughput: 50.0,
            },
        );
        timings.insert(
            "State Commit".to_string(),
            StageTiming {
                count: 10,
                total_ms: 100.0,
                min_ms: 8.0,
                max_ms: 12.0,
                avg_ms: 10.0,
                throughput: 100.0,
            },
        );

        let analyzer = BottleneckAnalyzer::new();
        let analysis = analyzer.analyze(&timings, 50.0);

        assert_eq!(
            analysis.primary_bottleneck,
            Some("Block Build".to_string())
        );
        assert!(analysis.improvement_potential_pct > 0.0);
    }
}
