//! Regression Detector - Detects performance regressions between runs
//!
//! Compares current benchmark results against historical baselines to detect
//! performance regressions in TPS, latency, and pipeline stages.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::history::HistoricalRun;
use crate::results::BenchmarkStats;
use starcoin_pipeline_timing::StageTiming;

/// Regression severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RegressionSeverity {
    /// No regression detected
    None,
    /// Minor regression (<5%)
    Minor,
    /// Moderate regression (5-15%)
    Moderate,
    /// Major regression (15-30%)
    Major,
    /// Critical regression (>30%)
    Critical,
}

impl RegressionSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            RegressionSeverity::None => "none",
            RegressionSeverity::Minor => "minor",
            RegressionSeverity::Moderate => "moderate",
            RegressionSeverity::Major => "major",
            RegressionSeverity::Critical => "critical",
        }
    }

    pub fn from_percentage(degradation_pct: f64) -> Self {
        if degradation_pct < 0.0 {
            RegressionSeverity::None // Improvement
        } else if degradation_pct < 5.0 {
            RegressionSeverity::Minor
        } else if degradation_pct < 15.0 {
            RegressionSeverity::Moderate
        } else if degradation_pct < 30.0 {
            RegressionSeverity::Major
        } else {
            RegressionSeverity::Critical
        }
    }
}

/// A single metric comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricComparison {
    pub metric_name: String,
    pub baseline_value: f64,
    pub current_value: f64,
    /// Positive = regression (worse), Negative = improvement (better)
    pub change_pct: f64,
    pub severity: RegressionSeverity,
    /// True if this metric improved
    pub improved: bool,
}

/// Stage-level regression info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRegression {
    pub stage_name: String,
    /// Change in average execution time
    pub time_change_pct: f64,
    /// Change in throughput
    pub throughput_change_pct: f64,
    pub severity: RegressionSeverity,
}

/// Complete regression analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionReport {
    /// Baseline run ID
    pub baseline_run_id: String,
    /// Baseline timestamp
    pub baseline_timestamp: String,
    /// Current run ID
    pub current_run_id: String,
    /// Overall regression detected
    pub has_regression: bool,
    /// Max severity across all metrics
    pub max_severity: RegressionSeverity,
    /// TPS comparison
    pub tps_comparison: MetricComparison,
    /// Latency comparison
    pub latency_comparison: MetricComparison,
    /// Per-stage regressions
    pub stage_regressions: Vec<StageRegression>,
    /// Summary recommendation
    pub recommendation: String,
}

/// Regression detector configuration
pub struct RegressionDetector {
    /// Threshold for considering TPS change a regression (percentage)
    tps_threshold_pct: f64,
    /// Threshold for considering latency change a regression (percentage)
    latency_threshold_pct: f64,
    /// Threshold for considering stage time change significant (percentage)
    stage_threshold_pct: f64,
}

impl Default for RegressionDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl RegressionDetector {
    pub fn new() -> Self {
        Self {
            tps_threshold_pct: 5.0,      // 5% TPS drop is a regression
            latency_threshold_pct: 10.0, // 10% latency increase is a regression
            stage_threshold_pct: 15.0,   // 15% stage slowdown is significant
        }
    }

    /// Configure thresholds
    pub fn with_thresholds(mut self, tps_pct: f64, latency_pct: f64, stage_pct: f64) -> Self {
        self.tps_threshold_pct = tps_pct;
        self.latency_threshold_pct = latency_pct;
        self.stage_threshold_pct = stage_pct;
        self
    }

    /// Compare current run against baseline
    pub fn compare(
        &self,
        baseline: &HistoricalRun,
        current_stats: &BenchmarkStats,
        current_stages: &HashMap<String, StageTiming>,
        current_run_id: &str,
    ) -> RegressionReport {
        // Compare TPS (higher is better, so negative change = regression)
        let tps_change_pct = if baseline.stats.tps > 0.0 {
            ((baseline.stats.tps - current_stats.tps) / baseline.stats.tps) * 100.0
        } else {
            0.0
        };
        
        let tps_comparison = MetricComparison {
            metric_name: "TPS".to_string(),
            baseline_value: baseline.stats.tps,
            current_value: current_stats.tps,
            change_pct: tps_change_pct,
            severity: RegressionSeverity::from_percentage(tps_change_pct),
            improved: tps_change_pct < 0.0,
        };

        // Compare latency (lower is better, so positive change = regression)
        let latency_change_pct = if baseline.stats.avg_latency_ms > 0.0 {
            ((current_stats.avg_latency_ms - baseline.stats.avg_latency_ms) / baseline.stats.avg_latency_ms) * 100.0
        } else {
            0.0
        };
        
        let latency_comparison = MetricComparison {
            metric_name: "Latency".to_string(),
            baseline_value: baseline.stats.avg_latency_ms,
            current_value: current_stats.avg_latency_ms,
            change_pct: latency_change_pct,
            severity: RegressionSeverity::from_percentage(latency_change_pct),
            improved: latency_change_pct < 0.0,
        };

        // Compare pipeline stages
        let stage_regressions = self.compare_stages(&baseline.pipeline_stages, current_stages);

        // Determine overall regression status
        let has_tps_regression = tps_change_pct >= self.tps_threshold_pct;
        let has_latency_regression = latency_change_pct >= self.latency_threshold_pct;
        let has_stage_regression = stage_regressions.iter().any(|s| s.severity >= RegressionSeverity::Moderate);
        
        let has_regression = has_tps_regression || has_latency_regression || has_stage_regression;

        // Find max severity
        let max_severity = [
            tps_comparison.severity,
            latency_comparison.severity,
        ]
        .into_iter()
        .chain(stage_regressions.iter().map(|s| s.severity))
        .max()
        .unwrap_or(RegressionSeverity::None);

        // Generate recommendation
        let recommendation = self.generate_recommendation(
            has_regression,
            &tps_comparison,
            &latency_comparison,
            &stage_regressions,
        );

        RegressionReport {
            baseline_run_id: baseline.run_id.clone(),
            baseline_timestamp: baseline.timestamp.to_rfc3339(),
            current_run_id: current_run_id.to_string(),
            has_regression,
            max_severity,
            tps_comparison,
            latency_comparison,
            stage_regressions,
            recommendation,
        }
    }

    fn compare_stages(
        &self,
        baseline_stages: &HashMap<String, StageTiming>,
        current_stages: &HashMap<String, StageTiming>,
    ) -> Vec<StageRegression> {
        let mut regressions = Vec::new();

        for (stage_name, baseline_timing) in baseline_stages {
            if let Some(current_timing) = current_stages.get(stage_name) {
                // Compare average time (higher = worse)
                let time_change_pct = if baseline_timing.avg_ms > 0.0 {
                    ((current_timing.avg_ms - baseline_timing.avg_ms) / baseline_timing.avg_ms) * 100.0
                } else {
                    0.0
                };

                // Compare throughput (lower = worse)
                let throughput_change_pct = if baseline_timing.throughput > 0.0 {
                    ((baseline_timing.throughput - current_timing.throughput) / baseline_timing.throughput) * 100.0
                } else {
                    0.0
                };

                // Use the worse of the two metrics
                let max_degradation = time_change_pct.max(throughput_change_pct);
                let severity = RegressionSeverity::from_percentage(max_degradation);

                regressions.push(StageRegression {
                    stage_name: stage_name.clone(),
                    time_change_pct,
                    throughput_change_pct,
                    severity,
                });
            }
        }

        // Sort by severity (worst first)
        regressions.sort_by(|a, b| b.severity.cmp(&a.severity));

        regressions
    }

    fn generate_recommendation(
        &self,
        has_regression: bool,
        tps: &MetricComparison,
        latency: &MetricComparison,
        stages: &[StageRegression],
    ) -> String {
        if !has_regression {
            if tps.improved && latency.improved {
                return "🎉 Performance IMPROVED! TPS increased and latency decreased.".to_string();
            } else if tps.improved {
                return "✅ Performance improved: TPS increased.".to_string();
            } else if latency.improved {
                return "✅ Performance improved: Latency decreased.".to_string();
            } else {
                return "✅ No performance regression detected. Results are within acceptable range.".to_string();
            }
        }

        let mut recommendations = vec!["⚠️ REGRESSION DETECTED:".to_string()];

        if tps.severity >= RegressionSeverity::Moderate {
            recommendations.push(format!(
                "  - TPS dropped by {:.1}% ({:.2} → {:.2})",
                tps.change_pct, tps.baseline_value, tps.current_value
            ));
        }

        if latency.severity >= RegressionSeverity::Moderate {
            recommendations.push(format!(
                "  - Latency increased by {:.1}% ({:.2}ms → {:.2}ms)",
                latency.change_pct, latency.baseline_value, latency.current_value
            ));
        }

        // Find worst stage regression
        if let Some(worst_stage) = stages.iter().find(|s| s.severity >= RegressionSeverity::Moderate) {
            recommendations.push(format!(
                "  - {} stage slowed down by {:.1}%",
                worst_stage.stage_name, worst_stage.time_change_pct
            ));
        }

        recommendations.push("\nActions:".to_string());
        recommendations.push("  1. Check recent code changes using: git diff HEAD~5".to_string());
        recommendations.push("  2. Profile the regressed stage using perf or flamegraph".to_string());
        recommendations.push("  3. Consider reverting recent changes if critical".to_string());

        recommendations.join("\n")
    }
}

impl std::fmt::Display for RegressionReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "========== Regression Report ==========")?;
        writeln!(f, "Baseline: {} ({})", self.baseline_run_id, self.baseline_timestamp)?;
        writeln!(f, "Current:  {}", self.current_run_id)?;
        writeln!(f)?;

        let status = if self.has_regression {
            format!("⚠️ REGRESSION ({})", self.max_severity.as_str().to_uppercase())
        } else {
            "✅ NO REGRESSION".to_string()
        };
        writeln!(f, "Status: {}", status)?;
        writeln!(f)?;

        writeln!(f, "Metrics:")?;
        let tps_arrow = if self.tps_comparison.improved { "↑" } else { "↓" };
        writeln!(
            f,
            "  TPS: {:.2} → {:.2} ({}{:.1}%) {}",
            self.tps_comparison.baseline_value,
            self.tps_comparison.current_value,
            if self.tps_comparison.change_pct < 0.0 { "+" } else { "-" },
            self.tps_comparison.change_pct.abs(),
            tps_arrow
        )?;

        let lat_arrow = if self.latency_comparison.improved { "↓" } else { "↑" };
        writeln!(
            f,
            "  Latency: {:.2}ms → {:.2}ms ({}{:.1}%) {}",
            self.latency_comparison.baseline_value,
            self.latency_comparison.current_value,
            if self.latency_comparison.change_pct > 0.0 { "+" } else { "" },
            self.latency_comparison.change_pct,
            lat_arrow
        )?;
        writeln!(f)?;

        if !self.stage_regressions.is_empty() {
            writeln!(f, "Stage Changes:")?;
            for stage in &self.stage_regressions {
                let severity_str = if stage.severity != RegressionSeverity::None {
                    format!(" [{}]", stage.severity.as_str())
                } else {
                    String::new()
                };
                writeln!(
                    f,
                    "  {}: time {}{:.1}%, throughput {}{:.1}%{}",
                    stage.stage_name,
                    if stage.time_change_pct > 0.0 { "+" } else { "" },
                    stage.time_change_pct,
                    if stage.throughput_change_pct < 0.0 { "" } else { "+" },
                    -stage.throughput_change_pct, // Negate because negative throughput change = improvement
                    severity_str
                )?;
            }
            writeln!(f)?;
        }

        writeln!(f, "{}", self.recommendation)?;
        writeln!(f, "========================================")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::history::BenchmarkConfig;

    fn create_test_stats(tps: f64, latency: f64) -> BenchmarkStats {
        BenchmarkStats {
            tps,
            block_tps_min: tps * 0.8,
            block_tps_max: tps * 1.2,
            block_tps_avg: tps,
            block_tps_median: tps,
            mined_tps_min: tps * 0.9,
            mined_tps_max: tps * 1.1,
            mined_tps_avg: tps,
            mined_tps_median: tps,
            total_executed: 100,
            unique_txn_count: 100,
            duplicate_exec_count: 0,
            duplicate_pct: 0.0,
            min_latency_ms: latency * 0.5,
            max_latency_ms: latency * 2.0,
            avg_latency_ms: latency,
            median_latency_ms: latency,
        }
    }

    #[test]
    fn test_regression_detection() {
        let baseline = HistoricalRun {
            run_id: "baseline".to_string(),
            timestamp: Utc::now(),
            git_commit: None,
            git_branch: None,
            config: BenchmarkConfig {
                account_count: 10,
                batch_user_count: 2,
                gas_price: 1,
                max_gas: 1000000,
                network: "test".to_string(),
            },
            stats: create_test_stats(100.0, 50.0),
            pipeline_stages: HashMap::new(),
            tags: vec![],
        };

        let detector = RegressionDetector::new();

        // Test regression (lower TPS, higher latency)
        let current_stats = create_test_stats(80.0, 70.0);
        let report = detector.compare(&baseline, &current_stats, &HashMap::new(), "current");
        assert!(report.has_regression);

        // Test improvement (higher TPS, lower latency)
        let current_stats = create_test_stats(120.0, 40.0);
        let report = detector.compare(&baseline, &current_stats, &HashMap::new(), "current");
        assert!(!report.has_regression);
    }
}
