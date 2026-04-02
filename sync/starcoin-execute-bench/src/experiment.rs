//! Experiment Framework - A/B Testing for TPS Optimization
//!
//! This module provides statistical comparison of configurations:
//! - Running multiple benchmark iterations
//! - Statistical significance testing
//! - Confidence interval calculation
//! - Multi-variant comparison

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config_tuner::ConfigChange;
use crate::results::BenchmarkStats;

/// An experiment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    /// Unique experiment ID
    pub id: String,
    /// Experiment name
    pub name: String,
    /// Description
    pub description: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Experiment state
    pub state: ExperimentState,
    /// Variants to test
    pub variants: Vec<ExperimentVariant>,
    /// Number of iterations per variant
    pub iterations_per_variant: u32,
    /// Metrics to compare
    pub target_metrics: Vec<String>,
    /// Baseline variant ID (for comparison)
    pub baseline_variant_id: String,
    /// Results
    pub results: Option<ExperimentResults>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExperimentState {
    /// Not started
    Pending,
    /// Currently running
    Running,
    /// Completed with results
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}

impl ExperimentState {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExperimentState::Pending => "pending",
            ExperimentState::Running => "running",
            ExperimentState::Completed => "completed",
            ExperimentState::Failed => "failed",
            ExperimentState::Cancelled => "cancelled",
        }
    }
}

/// A variant (configuration) to test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentVariant {
    /// Variant ID
    pub id: String,
    /// Variant name (e.g., "baseline", "optimized_v1")
    pub name: String,
    /// Configuration changes from baseline
    pub config_changes: Vec<ConfigChange>,
    /// Collected samples
    pub samples: Vec<BenchmarkSample>,
}

impl ExperimentVariant {
    pub fn new(name: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            config_changes: Vec::new(),
            samples: Vec::new(),
        }
    }

    pub fn with_changes(name: &str, changes: Vec<ConfigChange>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            config_changes: changes,
            samples: Vec::new(),
        }
    }
}

/// A benchmark sample for a variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSample {
    /// Iteration number
    pub iteration: u32,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Benchmark stats
    pub stats: BenchmarkStats,
    /// Additional metrics
    pub metrics: HashMap<String, f64>,
}

/// Results of an experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResults {
    /// Statistical comparison between variants
    pub comparisons: Vec<VariantComparison>,
    /// Best variant ID
    pub best_variant_id: String,
    /// Confidence in the result
    pub confidence: f64,
    /// Recommendation
    pub recommendation: String,
    /// Detailed analysis
    pub analysis: HashMap<String, MetricAnalysis>,
}

/// Comparison between two variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantComparison {
    /// Control variant ID
    pub control_id: String,
    /// Treatment variant ID
    pub treatment_id: String,
    /// Metric comparisons
    pub metrics: HashMap<String, MetricComparison>,
    /// Overall significance
    pub is_significant: bool,
    /// Treatment is better
    pub treatment_is_better: bool,
}

/// Statistical comparison of a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricComparison {
    /// Metric name
    pub metric_name: String,
    /// Control mean
    pub control_mean: f64,
    /// Control std dev
    pub control_std: f64,
    /// Treatment mean
    pub treatment_mean: f64,
    /// Treatment std dev
    pub treatment_std: f64,
    /// Absolute difference
    pub difference: f64,
    /// Percentage difference
    pub difference_pct: f64,
    /// p-value (Welch's t-test)
    pub p_value: f64,
    /// 95% confidence interval lower bound
    pub ci_lower: f64,
    /// 95% confidence interval upper bound
    pub ci_upper: f64,
    /// Is statistically significant (p < 0.05)
    pub is_significant: bool,
    /// Higher is better for this metric
    pub higher_is_better: bool,
    /// Treatment is better
    pub treatment_is_better: bool,
}

/// Detailed analysis of a metric across all variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricAnalysis {
    pub metric_name: String,
    pub variant_stats: HashMap<String, VariantMetricStats>,
    pub best_variant_id: String,
    pub best_value: f64,
    pub improvement_over_baseline_pct: f64,
}

/// Statistics for a metric in one variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantMetricStats {
    pub mean: f64,
    pub std: f64,
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub samples: usize,
}

/// Experiment Framework
pub struct ExperimentFramework {
    /// Storage path
    store_path: PathBuf,
    /// Active experiments
    experiments: HashMap<String, Experiment>,
    /// Current experiment ID
    current_experiment_id: Option<String>,
}

impl ExperimentFramework {
    /// Create a new experiment framework
    pub fn new<P: AsRef<Path>>(store_dir: P) -> Result<Self> {
        let store_dir = store_dir.as_ref();
        std::fs::create_dir_all(store_dir)?;

        let mut framework = Self {
            store_path: store_dir.to_path_buf(),
            experiments: HashMap::new(),
            current_experiment_id: None,
        };

        framework.load()?;

        Ok(framework)
    }

    /// Load experiments from disk
    fn load(&mut self) -> Result<()> {
        let experiments_dir = self.store_path.join("experiments");
        if !experiments_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&experiments_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let content = std::fs::read_to_string(&path)?;
                if let Ok(exp) = serde_json::from_str::<Experiment>(&content) {
                    self.experiments.insert(exp.id.clone(), exp);
                }
            }
        }

        Ok(())
    }

    /// Save an experiment to disk
    fn save_experiment(&self, experiment: &Experiment) -> Result<()> {
        let experiments_dir = self.store_path.join("experiments");
        std::fs::create_dir_all(&experiments_dir)?;

        let file_path = experiments_dir.join(format!("{}.json", experiment.id));
        let json = serde_json::to_string_pretty(experiment)?;
        std::fs::write(file_path, json)?;

        Ok(())
    }

    /// Create a new experiment
    pub fn create_experiment(
        &mut self,
        name: &str,
        description: &str,
        iterations_per_variant: u32,
    ) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();

        // Create baseline variant
        let baseline = ExperimentVariant::new("baseline");
        let baseline_id = baseline.id.clone();

        let experiment = Experiment {
            id: id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            created_at: Utc::now(),
            state: ExperimentState::Pending,
            variants: vec![baseline],
            iterations_per_variant,
            target_metrics: vec![
                "tps".to_string(),
                "avg_latency_ms".to_string(),
            ],
            baseline_variant_id: baseline_id,
            results: None,
        };

        self.save_experiment(&experiment)?;
        self.experiments.insert(id.clone(), experiment);

        Ok(id)
    }

    /// Add a variant to an experiment
    pub fn add_variant(
        &mut self,
        experiment_id: &str,
        name: &str,
        config_changes: Vec<ConfigChange>,
    ) -> Result<String> {
        let experiment = self
            .experiments
            .get_mut(experiment_id)
            .ok_or_else(|| anyhow::anyhow!("Experiment not found: {}", experiment_id))?;

        if experiment.state != ExperimentState::Pending {
            bail!(
                "Cannot add variant to experiment in state: {}",
                experiment.state.as_str()
            );
        }

        let variant = ExperimentVariant::with_changes(name, config_changes);
        let variant_id = variant.id.clone();

        experiment.variants.push(variant);
        let exp_clone = experiment.clone();
        self.save_experiment(&exp_clone)?;

        Ok(variant_id)
    }

    /// Start an experiment
    pub fn start_experiment(&mut self, experiment_id: &str) -> Result<()> {
        let experiment = self
            .experiments
            .get_mut(experiment_id)
            .ok_or_else(|| anyhow::anyhow!("Experiment not found: {}", experiment_id))?;

        if experiment.state != ExperimentState::Pending {
            bail!(
                "Cannot start experiment in state: {}",
                experiment.state.as_str()
            );
        }

        if experiment.variants.len() < 2 {
            bail!("Need at least 2 variants (baseline + 1 treatment)");
        }

        experiment.state = ExperimentState::Running;
        self.current_experiment_id = Some(experiment_id.to_string());
        let exp_clone = experiment.clone();
        self.save_experiment(&exp_clone)?;

        Ok(())
    }

    /// Get current experiment
    pub fn current_experiment(&self) -> Option<&Experiment> {
        self.current_experiment_id
            .as_ref()
            .and_then(|id| self.experiments.get(id))
    }

    /// Get current experiment mutably
    pub fn current_experiment_mut(&mut self) -> Option<&mut Experiment> {
        if let Some(ref id) = self.current_experiment_id {
            self.experiments.get_mut(id)
        } else {
            None
        }
    }

    /// Record a sample for a variant
    pub fn record_sample(
        &mut self,
        experiment_id: &str,
        variant_id: &str,
        stats: BenchmarkStats,
    ) -> Result<()> {
        let experiment = self
            .experiments
            .get_mut(experiment_id)
            .ok_or_else(|| anyhow::anyhow!("Experiment not found: {}", experiment_id))?;

        let variant = experiment
            .variants
            .iter_mut()
            .find(|v| v.id == variant_id)
            .ok_or_else(|| anyhow::anyhow!("Variant not found: {}", variant_id))?;

        let iteration = variant.samples.len() as u32 + 1;

        let sample = BenchmarkSample {
            iteration,
            timestamp: Utc::now(),
            stats,
            metrics: HashMap::new(),
        };

        variant.samples.push(sample);
        let exp_clone = experiment.clone();
        self.save_experiment(&exp_clone)?;

        Ok(())
    }

    /// Check if experiment has enough samples
    pub fn has_enough_samples(&self, experiment_id: &str) -> bool {
        self.experiments.get(experiment_id).map_or(false, |exp| {
            exp.variants.iter().all(|v| {
                v.samples.len() as u32 >= exp.iterations_per_variant
            })
        })
    }

    /// Get next variant to sample
    pub fn next_variant_to_sample(&self, experiment_id: &str) -> Option<&ExperimentVariant> {
        self.experiments.get(experiment_id).and_then(|exp| {
            exp.variants
                .iter()
                .filter(|v| (v.samples.len() as u32) < exp.iterations_per_variant)
                .min_by_key(|v| v.samples.len())
        })
    }

    /// Analyze experiment results
    pub fn analyze(&mut self, experiment_id: &str) -> Result<ExperimentResults> {
        // First, extract all needed data without holding mutable borrow
        let (variants, baseline_id, target_metrics) = {
            let experiment = self
                .experiments
                .get(experiment_id)
                .ok_or_else(|| anyhow::anyhow!("Experiment not found: {}", experiment_id))?;

            if experiment.variants.iter().any(|v| v.samples.is_empty()) {
                bail!("Not all variants have samples");
            }

            (
                experiment.variants.clone(),
                experiment.baseline_variant_id.clone(),
                experiment.target_metrics.clone(),
            )
        };

        let baseline = variants
            .iter()
            .find(|v| v.id == baseline_id)
            .ok_or_else(|| anyhow::anyhow!("Baseline variant not found"))?
            .clone();

        let mut comparisons = Vec::new();
        let mut analysis: HashMap<String, MetricAnalysis> = HashMap::new();

        // Compare each treatment to baseline
        for variant in &variants {
            if variant.id == baseline.id {
                continue;
            }

            let comparison = self.compare_variants(&baseline, variant, &target_metrics);
            comparisons.push(comparison);
        }

        // Analyze each metric
        for metric in &target_metrics {
            let metric_analysis = self.analyze_metric(metric, &variants);
            analysis.insert(metric.clone(), metric_analysis);
        }

        // Find best variant (primarily by TPS)
        let tps_analysis = analysis.get("tps");
        let best_variant_id = tps_analysis
            .map(|a| a.best_variant_id.clone())
            .unwrap_or_else(|| baseline_id.clone());

        let confidence = comparisons
            .iter()
            .filter(|c| c.treatment_id == best_variant_id)
            .next()
            .map(|c| if c.is_significant { 0.95 } else { 0.5 })
            .unwrap_or(0.5);

        let recommendation = self.generate_recommendation(&comparisons, &analysis, &best_variant_id);

        let results = ExperimentResults {
            comparisons,
            best_variant_id,
            confidence,
            recommendation,
            analysis,
        };

        // Now update experiment with results
        if let Some(experiment) = self.experiments.get_mut(experiment_id) {
            experiment.state = ExperimentState::Completed;
            experiment.results = Some(results.clone());
            let exp_clone = experiment.clone();
            self.save_experiment(&exp_clone)?;
        }

        Ok(results)
    }

    /// Compare two variants statistically
    fn compare_variants(
        &self,
        control: &ExperimentVariant,
        treatment: &ExperimentVariant,
        metrics: &[String],
    ) -> VariantComparison {
        let mut metric_comparisons = HashMap::new();
        let mut all_significant = true;
        let mut treatment_better_count = 0;

        for metric in metrics {
            let comparison = self.compare_metric(control, treatment, metric);
            if !comparison.is_significant {
                all_significant = false;
            }
            if comparison.treatment_is_better {
                treatment_better_count += 1;
            }
            metric_comparisons.insert(metric.clone(), comparison);
        }

        VariantComparison {
            control_id: control.id.clone(),
            treatment_id: treatment.id.clone(),
            metrics: metric_comparisons,
            is_significant: all_significant,
            treatment_is_better: treatment_better_count > metrics.len() / 2,
        }
    }

    /// Compare a single metric between two variants
    fn compare_metric(
        &self,
        control: &ExperimentVariant,
        treatment: &ExperimentVariant,
        metric: &str,
    ) -> MetricComparison {
        let control_values = self.extract_metric_values(control, metric);
        let treatment_values = self.extract_metric_values(treatment, metric);

        let control_mean = mean(&control_values);
        let control_std = std_dev(&control_values);
        let treatment_mean = mean(&treatment_values);
        let treatment_std = std_dev(&treatment_values);

        let difference = treatment_mean - control_mean;
        let difference_pct = if control_mean != 0.0 {
            difference / control_mean * 100.0
        } else {
            0.0
        };

        // Welch's t-test
        let p_value = welch_t_test(&control_values, &treatment_values);

        // 95% confidence interval for the difference
        let pooled_se = ((control_std.powi(2) / control_values.len() as f64)
            + (treatment_std.powi(2) / treatment_values.len() as f64))
        .sqrt();
        let t_critical = 1.96; // Approximate for large samples
        let ci_lower = difference - t_critical * pooled_se;
        let ci_upper = difference + t_critical * pooled_se;

        let is_significant = p_value < 0.05;
        let higher_is_better = metric == "tps" || metric.contains("throughput");
        let treatment_is_better = if higher_is_better {
            difference > 0.0 && is_significant
        } else {
            difference < 0.0 && is_significant
        };

        MetricComparison {
            metric_name: metric.to_string(),
            control_mean,
            control_std,
            treatment_mean,
            treatment_std,
            difference,
            difference_pct,
            p_value,
            ci_lower,
            ci_upper,
            is_significant,
            higher_is_better,
            treatment_is_better,
        }
    }

    /// Extract metric values from samples
    fn extract_metric_values(&self, variant: &ExperimentVariant, metric: &str) -> Vec<f64> {
        variant
            .samples
            .iter()
            .map(|s| match metric {
                "tps" => s.stats.tps,
                "avg_latency_ms" => s.stats.avg_latency_ms,
                "min_latency_ms" => s.stats.min_latency_ms,
                "max_latency_ms" => s.stats.max_latency_ms,
                "total_executed" => s.stats.total_executed as f64,
                _ => s.metrics.get(metric).copied().unwrap_or(0.0),
            })
            .collect()
    }

    /// Analyze a metric across all variants
    fn analyze_metric(&self, metric: &str, variants: &[ExperimentVariant]) -> MetricAnalysis {
        let mut variant_stats = HashMap::new();
        let mut best_variant_id = String::new();
        let mut best_value = f64::NEG_INFINITY;
        let higher_is_better = metric == "tps" || metric.contains("throughput");

        for variant in variants {
            let values = self.extract_metric_values(variant, metric);
            let stats = VariantMetricStats {
                mean: mean(&values),
                std: std_dev(&values),
                min: values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                max: values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
                median: median(&values),
                samples: values.len(),
            };

            let is_better = if higher_is_better {
                stats.mean > best_value
            } else {
                best_value == f64::NEG_INFINITY || stats.mean < best_value
            };

            if is_better {
                best_value = stats.mean;
                best_variant_id = variant.id.clone();
            }

            variant_stats.insert(variant.id.clone(), stats);
        }

        // Calculate improvement over baseline
        let baseline_id = variants.first().map(|v| v.id.clone()).unwrap_or_default();
        let baseline_mean = variant_stats
            .get(&baseline_id)
            .map(|s| s.mean)
            .unwrap_or(0.0);

        let improvement_over_baseline_pct = if baseline_mean != 0.0 {
            (best_value - baseline_mean) / baseline_mean * 100.0
        } else {
            0.0
        };

        MetricAnalysis {
            metric_name: metric.to_string(),
            variant_stats,
            best_variant_id,
            best_value,
            improvement_over_baseline_pct,
        }
    }

    /// Generate recommendation based on results
    fn generate_recommendation(
        &self,
        comparisons: &[VariantComparison],
        analysis: &HashMap<String, MetricAnalysis>,
        best_variant_id: &str,
    ) -> String {
        let significant_improvements: Vec<_> = comparisons
            .iter()
            .filter(|c| c.is_significant && c.treatment_is_better)
            .collect();

        if significant_improvements.is_empty() {
            return "No statistically significant improvement found. Consider:\n\
                    - Running more iterations\n\
                    - Trying different optimization approaches\n\
                    - Checking if the optimization targets the actual bottleneck"
                .to_string();
        }

        let best_comparison = comparisons
            .iter()
            .find(|c| c.treatment_id == *best_variant_id);

        if let Some(comp) = best_comparison {
            let tps_change = comp
                .metrics
                .get("tps")
                .map(|m| m.difference_pct)
                .unwrap_or(0.0);

            let latency_change = comp
                .metrics
                .get("avg_latency_ms")
                .map(|m| m.difference_pct)
                .unwrap_or(0.0);

            format!(
                "Recommended: Apply variant '{}'\n\
                 - TPS improvement: {:+.1}% (statistically significant)\n\
                 - Latency change: {:+.1}%\n\
                 - Confidence: {:.0}%",
                best_variant_id,
                tps_change,
                latency_change,
                if comp.is_significant { 95.0 } else { 50.0 }
            )
        } else {
            "Unable to determine best variant. Review results manually.".to_string()
        }
    }

    /// Get all experiments
    pub fn all_experiments(&self) -> Vec<&Experiment> {
        self.experiments.values().collect()
    }

    /// Get experiment by ID
    pub fn get_experiment(&self, id: &str) -> Option<&Experiment> {
        self.experiments.get(id)
    }
}

// Statistical helper functions

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let m = mean(values);
    let variance = values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if sorted.len() % 2 == 0 {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    }
}

/// Welch's t-test for two samples with potentially different variances
fn welch_t_test(sample1: &[f64], sample2: &[f64]) -> f64 {
    if sample1.len() < 2 || sample2.len() < 2 {
        return 1.0; // Not significant
    }

    let n1 = sample1.len() as f64;
    let n2 = sample2.len() as f64;
    let m1 = mean(sample1);
    let m2 = mean(sample2);
    let s1 = std_dev(sample1);
    let s2 = std_dev(sample2);

    if s1 == 0.0 && s2 == 0.0 {
        return if (m1 - m2).abs() < 1e-10 { 1.0 } else { 0.0 };
    }

    let se = ((s1.powi(2) / n1) + (s2.powi(2) / n2)).sqrt();
    if se == 0.0 {
        return 1.0;
    }

    let t = (m1 - m2).abs() / se;

    // Approximate p-value using normal distribution for large samples
    // For small samples, this is less accurate
    let p = 2.0 * (1.0 - normal_cdf(t));

    p.max(0.0).min(1.0)
}

/// Approximate normal CDF
fn normal_cdf(x: f64) -> f64 {
    // Approximation using error function
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

/// Approximate error function
fn erf(x: f64) -> f64 {
    // Horner's method approximation
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statistics() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((mean(&values) - 3.0).abs() < 0.001);
        assert!((median(&values) - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_welch_t_test() {
        let sample1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sample2 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let p = welch_t_test(&sample1, &sample2);
        assert!(p > 0.9, "Same samples should have high p-value");

        let sample3 = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let p2 = welch_t_test(&sample1, &sample3);
        assert!(p2 < 0.05, "Different samples should have low p-value");
    }
}
