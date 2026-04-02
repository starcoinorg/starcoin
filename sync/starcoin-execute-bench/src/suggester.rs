//! Optimization Suggester - Generates optimization recommendations based on bottleneck analysis
//!
//! This module provides actionable optimization suggestions for each pipeline stage.

use serde::{Deserialize, Serialize};

use crate::analyzer::{BottleneckSeverity, PipelineAnalysis, StageAnalysis};

/// Priority level for optimization suggestions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
            Priority::Critical => "critical",
        }
    }
}

/// Category of optimization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationCategory {
    /// Code-level optimization (algorithm, data structure)
    Code,
    /// Configuration tuning (parameters, thresholds)
    Config,
    /// Architecture change (parallelism, caching)
    Architecture,
    /// Resource allocation (memory, threads)
    Resource,
    /// Storage optimization (IO, caching)
    Storage,
}

impl OptimizationCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            OptimizationCategory::Code => "code",
            OptimizationCategory::Config => "config",
            OptimizationCategory::Architecture => "architecture",
            OptimizationCategory::Resource => "resource",
            OptimizationCategory::Storage => "storage",
        }
    }
}

/// A single optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Target pipeline stage
    pub stage: String,
    /// Priority level
    pub priority: Priority,
    /// Category of optimization
    pub category: OptimizationCategory,
    /// Brief title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Expected improvement percentage (estimate)
    pub expected_improvement_pct: Option<f64>,
    /// Related code paths or files
    pub related_paths: Vec<String>,
    /// Specific parameters to tune (if config category)
    pub config_params: Vec<ConfigParam>,
}

/// A configuration parameter that can be tuned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigParam {
    pub name: String,
    pub current_value: Option<String>,
    pub suggested_value: String,
    pub description: String,
}

/// Collection of optimization suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    /// All suggestions sorted by priority
    pub suggestions: Vec<Suggestion>,
    /// Summary of recommendations
    pub summary: String,
    /// Estimated total improvement potential
    pub total_improvement_potential_pct: f64,
}

/// Optimization Suggester
pub struct OptimizationSuggester {
    /// Minimum severity to generate suggestions
    min_severity: BottleneckSeverity,
}

impl Default for OptimizationSuggester {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationSuggester {
    pub fn new() -> Self {
        Self {
            min_severity: BottleneckSeverity::Minor,
        }
    }

    /// Generate optimization suggestions based on pipeline analysis
    pub fn suggest(&self, analysis: &PipelineAnalysis) -> OptimizationReport {
        let mut suggestions = Vec::new();

        for stage in &analysis.stages {
            if stage.severity >= self.min_severity {
                let stage_suggestions = self.generate_stage_suggestions(stage);
                suggestions.extend(stage_suggestions);
            }
        }

        // Sort by priority (highest first)
        suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Generate summary
        let summary = self.generate_summary(analysis, &suggestions);

        OptimizationReport {
            suggestions,
            summary,
            total_improvement_potential_pct: analysis.improvement_potential_pct,
        }
    }

    /// Generate suggestions for a specific stage
    fn generate_stage_suggestions(&self, stage: &StageAnalysis) -> Vec<Suggestion> {
        let priority = self.severity_to_priority(stage.severity);

        match stage.stage.as_str() {
            "TxPool Verify" => self.txpool_verify_suggestions(stage, priority),
            "Block Build" => self.block_build_suggestions(stage, priority),
            "VM Execute" => self.vm_execute_suggestions(stage, priority),
            "State Commit" => self.state_commit_suggestions(stage, priority),
            _ => vec![],
        }
    }

    fn txpool_verify_suggestions(&self, stage: &StageAnalysis, priority: Priority) -> Vec<Suggestion> {
        let mut suggestions = vec![];

        // Suggestion 1: Parallel verification
        if stage.avg_time_ms > 1.0 {
            suggestions.push(Suggestion {
                stage: stage.stage.clone(),
                priority,
                category: OptimizationCategory::Architecture,
                title: "Enable parallel transaction verification".to_string(),
                description: "Use rayon or tokio to verify multiple transactions concurrently. \
                             Signature verification and balance checks can be parallelized."
                    .to_string(),
                expected_improvement_pct: Some(30.0),
                related_paths: vec![
                    "txpool/src/pool_client.rs".to_string(),
                    "txpool/src/pool.rs".to_string(),
                ],
                config_params: vec![ConfigParam {
                    name: "txpool.verify_threads".to_string(),
                    current_value: None,
                    suggested_value: "4".to_string(),
                    description: "Number of parallel verification threads".to_string(),
                }],
            });
        }

        // Suggestion 2: Signature cache
        suggestions.push(Suggestion {
            stage: stage.stage.clone(),
            priority: Priority::Medium,
            category: OptimizationCategory::Architecture,
            title: "Implement signature verification cache".to_string(),
            description: "Cache verified signatures to avoid re-verification of known transactions. \
                         Useful when transactions are resubmitted or during reorgs."
                .to_string(),
            expected_improvement_pct: Some(15.0),
            related_paths: vec!["txpool/src/pool_client.rs".to_string()],
            config_params: vec![],
        });

        // Suggestion 3: Batch verification
        if stage.count > 10 {
            suggestions.push(Suggestion {
                stage: stage.stage.clone(),
                priority,
                category: OptimizationCategory::Code,
                title: "Batch signature verification".to_string(),
                description: "Use batch verification for Ed25519/BLS signatures when multiple \
                             transactions arrive together. This can be 2-3x faster than individual verification."
                    .to_string(),
                expected_improvement_pct: Some(40.0),
                related_paths: vec!["crypto/src/ed25519.rs".to_string()],
                config_params: vec![],
            });
        }

        suggestions
    }

    fn block_build_suggestions(&self, stage: &StageAnalysis, priority: Priority) -> Vec<Suggestion> {
        let mut suggestions = vec![];

        // Suggestion 1: Transaction selection optimization
        if stage.avg_time_ms > 5.0 {
            suggestions.push(Suggestion {
                stage: stage.stage.clone(),
                priority,
                category: OptimizationCategory::Code,
                title: "Optimize transaction selection algorithm".to_string(),
                description: "Use a priority queue or heap for gas-price based selection instead of sorting. \
                             Pre-compute and cache transaction ordering."
                    .to_string(),
                expected_improvement_pct: Some(25.0),
                related_paths: vec![
                    "miner/src/create_block_template/block_builder_service.rs".to_string(),
                ],
                config_params: vec![],
            });
        }

        // Suggestion 2: Incremental state root computation
        suggestions.push(Suggestion {
            stage: stage.stage.clone(),
            priority: Priority::Medium,
            category: OptimizationCategory::Architecture,
            title: "Incremental state root computation".to_string(),
            description: "Compute state root incrementally as transactions are added to block template, \
                         rather than recomputing entire tree at the end."
                .to_string(),
            expected_improvement_pct: Some(20.0),
            related_paths: vec![
                "miner/src/create_block_template/mod.rs".to_string(),
                "state/src/lib.rs".to_string(),
            ],
            config_params: vec![],
        });

        // Suggestion 3: Block size tuning
        suggestions.push(Suggestion {
            stage: stage.stage.clone(),
            priority: Priority::Low,
            category: OptimizationCategory::Config,
            title: "Tune block gas limit".to_string(),
            description: "Adjust block gas limit to balance between throughput and block propagation time."
                .to_string(),
            expected_improvement_pct: Some(10.0),
            related_paths: vec![],
            config_params: vec![ConfigParam {
                name: "chain.block_gas_limit".to_string(),
                current_value: None,
                suggested_value: "500000000".to_string(),
                description: "Maximum gas units per block".to_string(),
            }],
        });

        suggestions
    }

    fn vm_execute_suggestions(&self, stage: &StageAnalysis, priority: Priority) -> Vec<Suggestion> {
        let mut suggestions = vec![];

        // Suggestion 1: Parallel execution
        if stage.avg_time_ms > 2.0 {
            suggestions.push(Suggestion {
                stage: stage.stage.clone(),
                priority,
                category: OptimizationCategory::Architecture,
                title: "Enable parallel transaction execution (Block-STM)".to_string(),
                description: "Implement Block-STM or similar optimistic parallel execution. \
                             Transactions that don't conflict can run in parallel with speculative execution."
                    .to_string(),
                expected_improvement_pct: Some(50.0),
                related_paths: vec![
                    "vm/starcoin-vm/src/executor.rs".to_string(),
                    "executor/src/lib.rs".to_string(),
                ],
                config_params: vec![ConfigParam {
                    name: "vm.parallel_execution".to_string(),
                    current_value: Some("false".to_string()),
                    suggested_value: "true".to_string(),
                    description: "Enable parallel transaction execution".to_string(),
                }],
            });
        }

        // Suggestion 2: JIT compilation
        suggestions.push(Suggestion {
            stage: stage.stage.clone(),
            priority: Priority::Medium,
            category: OptimizationCategory::Code,
            title: "Enable Move bytecode JIT compilation".to_string(),
            description: "Use LLVM or Cranelift to JIT-compile hot Move functions. \
                         Can provide 5-10x speedup for compute-heavy contracts."
                .to_string(),
            expected_improvement_pct: Some(30.0),
            related_paths: vec!["vm/vm-runtime/src/lib.rs".to_string()],
            config_params: vec![],
        });

        // Suggestion 3: Code cache
        suggestions.push(Suggestion {
            stage: stage.stage.clone(),
            priority: Priority::Medium,
            category: OptimizationCategory::Architecture,
            title: "Warm code cache before execution".to_string(),
            description: "Pre-load frequently used modules into code cache. \
                         Analyze transaction patterns to predict needed modules."
                .to_string(),
            expected_improvement_pct: Some(15.0),
            related_paths: vec!["vm/vm-runtime/src/code_cache.rs".to_string()],
            config_params: vec![],
        });

        suggestions
    }

    fn state_commit_suggestions(&self, stage: &StageAnalysis, priority: Priority) -> Vec<Suggestion> {
        let mut suggestions = vec![];

        // Suggestion 1: Async flush
        if stage.avg_time_ms > 1.0 {
            suggestions.push(Suggestion {
                stage: stage.stage.clone(),
                priority,
                category: OptimizationCategory::Architecture,
                title: "Async state flush".to_string(),
                description: "Perform state flush asynchronously. Return from block execution immediately \
                             and let background task persist state. Use write-ahead log for durability."
                    .to_string(),
                expected_improvement_pct: Some(40.0),
                related_paths: vec![
                    "chain/src/chain.rs".to_string(),
                    "state/statedb/src/lib.rs".to_string(),
                ],
                config_params: vec![],
            });
        }

        // Suggestion 2: Batch writes
        suggestions.push(Suggestion {
            stage: stage.stage.clone(),
            priority: Priority::Medium,
            category: OptimizationCategory::Storage,
            title: "Batch state writes".to_string(),
            description: "Combine multiple small writes into larger batches. \
                         Use RocksDB WriteBatch for atomic multi-key writes."
                .to_string(),
            expected_improvement_pct: Some(25.0),
            related_paths: vec!["storage/src/lib.rs".to_string()],
            config_params: vec![ConfigParam {
                name: "storage.write_batch_size".to_string(),
                current_value: None,
                suggested_value: "1000".to_string(),
                description: "Number of keys to batch in single write".to_string(),
            }],
        });

        // Suggestion 3: SSD optimization
        suggestions.push(Suggestion {
            stage: stage.stage.clone(),
            priority: Priority::Low,
            category: OptimizationCategory::Config,
            title: "Tune RocksDB for SSD".to_string(),
            description: "Configure RocksDB options for SSD storage: increase max_background_jobs, \
                         enable direct IO, adjust compaction settings."
                .to_string(),
            expected_improvement_pct: Some(15.0),
            related_paths: vec![],
            config_params: vec![
                ConfigParam {
                    name: "storage.rocksdb.max_background_jobs".to_string(),
                    current_value: None,
                    suggested_value: "8".to_string(),
                    description: "Number of background compaction/flush threads".to_string(),
                },
                ConfigParam {
                    name: "storage.rocksdb.use_direct_io".to_string(),
                    current_value: None,
                    suggested_value: "true".to_string(),
                    description: "Use direct IO to bypass OS cache".to_string(),
                },
            ],
        });

        suggestions
    }

    fn severity_to_priority(&self, severity: BottleneckSeverity) -> Priority {
        match severity {
            BottleneckSeverity::Critical => Priority::Critical,
            BottleneckSeverity::Severe => Priority::High,
            BottleneckSeverity::Moderate => Priority::Medium,
            BottleneckSeverity::Minor => Priority::Low,
            BottleneckSeverity::None => Priority::Low,
        }
    }

    fn generate_summary(&self, analysis: &PipelineAnalysis, suggestions: &[Suggestion]) -> String {
        let mut summary = String::new();

        if let Some(ref bottleneck) = analysis.primary_bottleneck {
            summary.push_str(&format!(
                "Primary bottleneck identified: {}. ",
                bottleneck
            ));
        } else {
            summary.push_str("No significant bottleneck detected. ");
        }

        let critical_count = suggestions.iter().filter(|s| s.priority == Priority::Critical).count();
        let high_count = suggestions.iter().filter(|s| s.priority == Priority::High).count();

        if critical_count > 0 {
            summary.push_str(&format!(
                "{} critical optimization(s) recommended. ",
                critical_count
            ));
        }
        if high_count > 0 {
            summary.push_str(&format!(
                "{} high-priority optimization(s) available. ",
                high_count
            ));
        }

        if analysis.improvement_potential_pct > 50.0 {
            summary.push_str(&format!(
                "Significant improvement potential: {:.1}%.",
                analysis.improvement_potential_pct
            ));
        } else if analysis.improvement_potential_pct > 20.0 {
            summary.push_str(&format!(
                "Moderate improvement potential: {:.1}%.",
                analysis.improvement_potential_pct
            ));
        } else {
            summary.push_str("Pipeline is relatively well-optimized.");
        }

        summary
    }
}

impl std::fmt::Display for OptimizationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "========== Optimization Report ==========")?;
        writeln!(f, "Summary: {}", self.summary)?;
        writeln!(
            f,
            "Total Improvement Potential: {:.1}%",
            self.total_improvement_potential_pct
        )?;
        writeln!(f)?;

        for (i, suggestion) in self.suggestions.iter().enumerate() {
            writeln!(
                f,
                "{}. [{}] {} - {}",
                i + 1,
                suggestion.priority.as_str().to_uppercase(),
                suggestion.stage,
                suggestion.title
            )?;
            writeln!(f, "   Category: {}", suggestion.category.as_str())?;
            writeln!(f, "   {}", suggestion.description)?;
            if let Some(improvement) = suggestion.expected_improvement_pct {
                writeln!(f, "   Expected Improvement: {:.0}%", improvement)?;
            }
            if !suggestion.config_params.is_empty() {
                writeln!(f, "   Config Changes:")?;
                for param in &suggestion.config_params {
                    writeln!(f, "     - {}: {}", param.name, param.suggested_value)?;
                }
            }
            writeln!(f)?;
        }

        writeln!(f, "==========================================")?;
        Ok(())
    }
}
