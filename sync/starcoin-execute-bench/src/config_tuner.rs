//! Config Tuner - Automatically adjusts configuration parameters for optimization
//!
//! This module manages runtime configuration changes:
//! - Reading current configuration values
//! - Applying configuration changes
//! - Validating changes
//! - Rolling back on failure

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// A tunable configuration parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunableParam {
    /// Parameter name/path (e.g., "miner.block_gas_limit")
    pub name: String,
    /// Parameter category
    pub category: ParamCategory,
    /// Current value
    pub current_value: ParamValue,
    /// Minimum value (if numeric)
    pub min_value: Option<ParamValue>,
    /// Maximum value (if numeric)
    pub max_value: Option<ParamValue>,
    /// Default value
    pub default_value: ParamValue,
    /// Description
    pub description: String,
    /// File path where parameter is stored
    pub file_path: Option<PathBuf>,
    /// JSON path within file
    pub json_path: Option<String>,
    /// Environment variable name (if applicable)
    pub env_var: Option<String>,
    /// Requires restart
    pub requires_restart: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParamCategory {
    /// Block production parameters
    BlockProduction,
    /// Transaction pool parameters
    TxPool,
    /// VM execution parameters
    VmExecution,
    /// State storage parameters
    Storage,
    /// Network parameters
    Network,
    /// Consensus parameters
    Consensus,
}

impl ParamCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ParamCategory::BlockProduction => "block_production",
            ParamCategory::TxPool => "txpool",
            ParamCategory::VmExecution => "vm_execution",
            ParamCategory::Storage => "storage",
            ParamCategory::Network => "network",
            ParamCategory::Consensus => "consensus",
        }
    }
}

/// A parameter value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParamValue {
    Integer(i64),
    UnsignedInteger(u64),
    Float(f64),
    String(String),
    Boolean(bool),
    Duration(u64), // milliseconds
}

impl ParamValue {
    pub fn as_string(&self) -> String {
        match self {
            ParamValue::Integer(v) => v.to_string(),
            ParamValue::UnsignedInteger(v) => v.to_string(),
            ParamValue::Float(v) => v.to_string(),
            ParamValue::String(v) => v.clone(),
            ParamValue::Boolean(v) => v.to_string(),
            ParamValue::Duration(v) => format!("{}ms", v),
        }
    }

    pub fn parse_string(s: &str, template: &ParamValue) -> Result<ParamValue> {
        Ok(match template {
            ParamValue::Integer(_) => ParamValue::Integer(s.parse()?),
            ParamValue::UnsignedInteger(_) => ParamValue::UnsignedInteger(s.parse()?),
            ParamValue::Float(_) => ParamValue::Float(s.parse()?),
            ParamValue::String(_) => ParamValue::String(s.to_string()),
            ParamValue::Boolean(_) => ParamValue::Boolean(s.parse()?),
            ParamValue::Duration(_) => {
                let s = s.trim_end_matches("ms");
                ParamValue::Duration(s.parse()?)
            }
        })
    }

    /// Apply a multiplier (for numeric types)
    pub fn multiply(&self, factor: f64) -> ParamValue {
        match self {
            ParamValue::Integer(v) => ParamValue::Integer((*v as f64 * factor) as i64),
            ParamValue::UnsignedInteger(v) => {
                ParamValue::UnsignedInteger((*v as f64 * factor) as u64)
            }
            ParamValue::Float(v) => ParamValue::Float(v * factor),
            ParamValue::Duration(v) => ParamValue::Duration((*v as f64 * factor) as u64),
            _ => self.clone(),
        }
    }

    /// Add an offset (for numeric types)
    pub fn add(&self, offset: f64) -> ParamValue {
        match self {
            ParamValue::Integer(v) => ParamValue::Integer(*v + offset as i64),
            ParamValue::UnsignedInteger(v) => {
                ParamValue::UnsignedInteger((*v as i64 + offset as i64).max(0) as u64)
            }
            ParamValue::Float(v) => ParamValue::Float(v + offset),
            ParamValue::Duration(v) => {
                ParamValue::Duration((*v as i64 + offset as i64).max(0) as u64)
            }
            _ => self.clone(),
        }
    }
}

/// A configuration change to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    /// Parameter name
    pub param_name: String,
    /// Old value
    pub old_value: ParamValue,
    /// New value
    pub new_value: ParamValue,
    /// Reason for change
    pub reason: String,
}

/// A saved configuration state for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    /// Snapshot ID
    pub id: String,
    /// Timestamp
    pub timestamp: String,
    /// Parameter values
    pub values: HashMap<String, ParamValue>,
    /// Description
    pub description: String,
}

/// The Config Tuner
pub struct ConfigTuner {
    /// Known tunable parameters
    params: HashMap<String, TunableParam>,
    /// Configuration snapshots for rollback
    snapshots: Vec<ConfigSnapshot>,
    /// Pending changes (not yet applied)
    pending_changes: Vec<ConfigChange>,
    /// Storage path for snapshots
    store_path: PathBuf,
}

impl ConfigTuner {
    /// Create a new config tuner
    pub fn new<P: AsRef<Path>>(store_dir: P) -> Result<Self> {
        let store_dir = store_dir.as_ref();
        std::fs::create_dir_all(store_dir)?;

        let mut tuner = Self {
            params: HashMap::new(),
            snapshots: Vec::new(),
            pending_changes: Vec::new(),
            store_path: store_dir.to_path_buf(),
        };

        tuner.register_default_params();
        tuner.load_snapshots()?;

        Ok(tuner)
    }

    /// Register default tunable parameters
    fn register_default_params(&mut self) {
        // Block production parameters
        self.register_param(TunableParam {
            name: "miner.block_gas_limit".to_string(),
            category: ParamCategory::BlockProduction,
            current_value: ParamValue::UnsignedInteger(500_000_000),
            min_value: Some(ParamValue::UnsignedInteger(100_000_000)),
            max_value: Some(ParamValue::UnsignedInteger(2_000_000_000)),
            default_value: ParamValue::UnsignedInteger(500_000_000),
            description: "Maximum gas units per block".to_string(),
            file_path: None,
            json_path: Some("miner.block_gas_limit".to_string()),
            env_var: Some("STARCOIN_BLOCK_GAS_LIMIT".to_string()),
            requires_restart: false,
        });

        self.register_param(TunableParam {
            name: "miner.max_txns_per_block".to_string(),
            category: ParamCategory::BlockProduction,
            current_value: ParamValue::UnsignedInteger(1000),
            min_value: Some(ParamValue::UnsignedInteger(100)),
            max_value: Some(ParamValue::UnsignedInteger(10000)),
            default_value: ParamValue::UnsignedInteger(1000),
            description: "Maximum transactions per block".to_string(),
            file_path: None,
            json_path: Some("miner.max_txns_per_block".to_string()),
            env_var: None,
            requires_restart: false,
        });

        // TxPool parameters
        self.register_param(TunableParam {
            name: "txpool.max_count".to_string(),
            category: ParamCategory::TxPool,
            current_value: ParamValue::UnsignedInteger(4096),
            min_value: Some(ParamValue::UnsignedInteger(1024)),
            max_value: Some(ParamValue::UnsignedInteger(100000)),
            default_value: ParamValue::UnsignedInteger(4096),
            description: "Maximum transactions in pool".to_string(),
            file_path: None,
            json_path: Some("tx_pool.max_count".to_string()),
            env_var: Some("STARCOIN_TXPOOL_MAX_COUNT".to_string()),
            requires_restart: false,
        });

        self.register_param(TunableParam {
            name: "txpool.max_per_sender".to_string(),
            category: ParamCategory::TxPool,
            current_value: ParamValue::UnsignedInteger(128),
            min_value: Some(ParamValue::UnsignedInteger(16)),
            max_value: Some(ParamValue::UnsignedInteger(1024)),
            default_value: ParamValue::UnsignedInteger(128),
            description: "Maximum transactions per sender".to_string(),
            file_path: None,
            json_path: Some("tx_pool.max_per_sender".to_string()),
            env_var: None,
            requires_restart: false,
        });

        // VM execution parameters
        self.register_param(TunableParam {
            name: "vm.concurrency_level".to_string(),
            category: ParamCategory::VmExecution,
            current_value: ParamValue::UnsignedInteger(num_cpus::get() as u64),
            min_value: Some(ParamValue::UnsignedInteger(1)),
            max_value: Some(ParamValue::UnsignedInteger(256)),
            default_value: ParamValue::UnsignedInteger(num_cpus::get() as u64),
            description: "Block-STM parallel execution threads".to_string(),
            file_path: None,
            json_path: None,
            env_var: Some("STARCOIN_VM_CONCURRENCY".to_string()),
            requires_restart: true,
        });

        // Storage parameters
        self.register_param(TunableParam {
            name: "storage.cache_size_mb".to_string(),
            category: ParamCategory::Storage,
            current_value: ParamValue::UnsignedInteger(512),
            min_value: Some(ParamValue::UnsignedInteger(64)),
            max_value: Some(ParamValue::UnsignedInteger(8192)),
            default_value: ParamValue::UnsignedInteger(512),
            description: "State cache size in MB".to_string(),
            file_path: None,
            json_path: Some("storage.cache_size".to_string()),
            env_var: Some("STARCOIN_CACHE_SIZE_MB".to_string()),
            requires_restart: true,
        });

        // Consensus parameters
        self.register_param(TunableParam {
            name: "consensus.uncle_rate_target".to_string(),
            category: ParamCategory::Consensus,
            current_value: ParamValue::UnsignedInteger(80),
            min_value: Some(ParamValue::UnsignedInteger(10)),
            max_value: Some(ParamValue::UnsignedInteger(200)),
            default_value: ParamValue::UnsignedInteger(80),
            description: "Target uncle rate percentage".to_string(),
            file_path: None,
            json_path: Some("consensus.uncle_rate_target".to_string()),
            env_var: None,
            requires_restart: false,
        });
    }

    /// Register a tunable parameter
    pub fn register_param(&mut self, param: TunableParam) {
        self.params.insert(param.name.clone(), param);
    }

    /// Get a parameter
    pub fn get_param(&self, name: &str) -> Option<&TunableParam> {
        self.params.get(name)
    }

    /// Get all parameters
    pub fn all_params(&self) -> Vec<&TunableParam> {
        self.params.values().collect()
    }

    /// Get parameters by category
    pub fn params_by_category(&self, category: ParamCategory) -> Vec<&TunableParam> {
        self.params
            .values()
            .filter(|p| p.category == category)
            .collect()
    }

    /// Get current value of a parameter
    pub fn get_value(&self, name: &str) -> Option<&ParamValue> {
        self.params.get(name).map(|p| &p.current_value)
    }

    /// Queue a parameter change
    pub fn queue_change(&mut self, param_name: &str, new_value: ParamValue, reason: &str) -> Result<()> {
        let param = self
            .params
            .get(param_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown parameter: {}", param_name))?;

        // Validate bounds
        if let (Some(min), Some(max)) = (&param.min_value, &param.max_value) {
            let is_valid = match (&new_value, min, max) {
                (ParamValue::Integer(v), ParamValue::Integer(min), ParamValue::Integer(max)) => {
                    *v >= *min && *v <= *max
                }
                (
                    ParamValue::UnsignedInteger(v),
                    ParamValue::UnsignedInteger(min),
                    ParamValue::UnsignedInteger(max),
                ) => *v >= *min && *v <= *max,
                (ParamValue::Float(v), ParamValue::Float(min), ParamValue::Float(max)) => {
                    *v >= *min && *v <= *max
                }
                _ => true,
            };

            if !is_valid {
                bail!(
                    "Value {} is out of range [{}, {}] for parameter {}",
                    new_value.as_string(),
                    min.as_string(),
                    max.as_string(),
                    param_name
                );
            }
        }

        self.pending_changes.push(ConfigChange {
            param_name: param_name.to_string(),
            old_value: param.current_value.clone(),
            new_value,
            reason: reason.to_string(),
        });

        Ok(())
    }

    /// Queue a percentage change
    pub fn queue_percentage_change(
        &mut self,
        param_name: &str,
        percentage: f64,
        reason: &str,
    ) -> Result<()> {
        let param = self
            .params
            .get(param_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown parameter: {}", param_name))?;

        let factor = 1.0 + percentage / 100.0;
        let new_value = param.current_value.multiply(factor);

        self.queue_change(param_name, new_value, reason)
    }

    /// Get pending changes
    pub fn pending_changes(&self) -> &[ConfigChange] {
        &self.pending_changes
    }

    /// Clear pending changes
    pub fn clear_pending(&mut self) {
        self.pending_changes.clear();
    }

    /// Create a snapshot of current configuration
    pub fn create_snapshot(&mut self, description: &str) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();

        let values: HashMap<String, ParamValue> = self
            .params
            .iter()
            .map(|(k, v)| (k.clone(), v.current_value.clone()))
            .collect();

        let snapshot = ConfigSnapshot {
            id: id.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            values,
            description: description.to_string(),
        };

        self.snapshots.push(snapshot.clone());
        self.save_snapshot(&snapshot)?;

        Ok(id)
    }

    /// Save a snapshot to disk
    fn save_snapshot(&self, snapshot: &ConfigSnapshot) -> Result<()> {
        let file_path = self.store_path.join(format!("snapshot_{}.json", snapshot.id));
        let json = serde_json::to_string_pretty(snapshot)?;
        fs::write(file_path, json)?;
        Ok(())
    }

    /// Load snapshots from disk
    fn load_snapshots(&mut self) -> Result<()> {
        for entry in fs::read_dir(&self.store_path)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with("snapshot_"))
                .unwrap_or(false)
            {
                let content = fs::read_to_string(&path)?;
                if let Ok(snapshot) = serde_json::from_str::<ConfigSnapshot>(&content) {
                    self.snapshots.push(snapshot);
                }
            }
        }
        self.snapshots.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(())
    }

    /// Apply pending changes
    pub fn apply_pending(&mut self) -> Result<Vec<ConfigChange>> {
        let changes = std::mem::take(&mut self.pending_changes);

        for change in &changes {
            if let Some(param) = self.params.get_mut(&change.param_name) {
                param.current_value = change.new_value.clone();

                // Set environment variable if applicable
                if let Some(ref env_var) = param.env_var {
                    std::env::set_var(env_var, change.new_value.as_string());
                }
            }
        }

        Ok(changes)
    }

    /// Rollback to a snapshot
    pub fn rollback_to(&mut self, snapshot_id: &str) -> Result<Vec<ConfigChange>> {
        let snapshot = self
            .snapshots
            .iter()
            .find(|s| s.id == snapshot_id)
            .ok_or_else(|| anyhow::anyhow!("Snapshot not found: {}", snapshot_id))?
            .clone();

        let mut changes = Vec::new();

        for (name, value) in &snapshot.values {
            if let Some(param) = self.params.get_mut(name) {
                if param.current_value != *value {
                    changes.push(ConfigChange {
                        param_name: name.clone(),
                        old_value: param.current_value.clone(),
                        new_value: value.clone(),
                        reason: format!("Rollback to snapshot {}", snapshot_id),
                    });
                    param.current_value = value.clone();

                    if let Some(ref env_var) = param.env_var {
                        std::env::set_var(env_var, value.as_string());
                    }
                }
            }
        }

        Ok(changes)
    }

    /// Get latest snapshot
    pub fn latest_snapshot(&self) -> Option<&ConfigSnapshot> {
        self.snapshots.last()
    }

    /// Get snapshot by ID
    pub fn get_snapshot(&self, id: &str) -> Option<&ConfigSnapshot> {
        self.snapshots.iter().find(|s| s.id == id)
    }

    /// Generate tuning suggestions based on bottleneck
    pub fn suggest_tuning(&self, bottleneck_stage: &str, severity: &str) -> Vec<TuningSuggestion> {
        let mut suggestions = Vec::new();

        match bottleneck_stage {
            "Block Build" => {
                if severity == "Critical" || severity == "Severe" {
                    suggestions.push(TuningSuggestion {
                        param_name: "miner.block_gas_limit".to_string(),
                        change_type: TuningChangeType::Increase,
                        percentage: 50.0,
                        reason: "Increase block capacity to reduce build time pressure".to_string(),
                        confidence: 0.7,
                    });
                }

                suggestions.push(TuningSuggestion {
                    param_name: "miner.max_txns_per_block".to_string(),
                    change_type: TuningChangeType::Increase,
                    percentage: 25.0,
                    reason: "Allow more transactions per block".to_string(),
                    confidence: 0.6,
                });
            }
            "TxPool Verify" => {
                suggestions.push(TuningSuggestion {
                    param_name: "txpool.max_count".to_string(),
                    change_type: TuningChangeType::Increase,
                    percentage: 100.0,
                    reason: "Larger pool reduces verification pressure".to_string(),
                    confidence: 0.5,
                });
            }
            "VM Execute" => {
                suggestions.push(TuningSuggestion {
                    param_name: "vm.concurrency_level".to_string(),
                    change_type: TuningChangeType::Increase,
                    percentage: 50.0,
                    reason: "More parallel threads for Block-STM".to_string(),
                    confidence: 0.8,
                });
            }
            "State Commit" => {
                suggestions.push(TuningSuggestion {
                    param_name: "storage.cache_size_mb".to_string(),
                    change_type: TuningChangeType::Increase,
                    percentage: 100.0,
                    reason: "Larger cache reduces state commit I/O".to_string(),
                    confidence: 0.7,
                });
            }
            _ => {}
        }

        suggestions
    }

    /// Export current configuration as JSON
    pub fn export_config(&self) -> serde_json::Value {
        let mut config = serde_json::Map::new();

        for (name, param) in &self.params {
            config.insert(
                name.clone(),
                serde_json::json!({
                    "value": param.current_value.as_string(),
                    "category": param.category.as_str(),
                    "description": param.description,
                }),
            );
        }

        serde_json::Value::Object(config)
    }
}

/// A tuning suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuningSuggestion {
    pub param_name: String,
    pub change_type: TuningChangeType,
    pub percentage: f64,
    pub reason: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TuningChangeType {
    Increase,
    Decrease,
    SetValue(ParamValue),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_value_multiply() {
        let value = ParamValue::UnsignedInteger(100);
        let result = value.multiply(1.5);
        assert_eq!(result, ParamValue::UnsignedInteger(150));
    }

    #[test]
    fn test_param_value_add() {
        let value = ParamValue::Integer(100);
        let result = value.add(-50.0);
        assert_eq!(result, ParamValue::Integer(50));
    }
}
