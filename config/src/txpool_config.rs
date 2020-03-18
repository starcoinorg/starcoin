use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct TxPoolConfig {
    /// Maximal number of transactions in the pool.
    pub max_count: u64,
    /// Maximal number of transactions from single sender.
    pub max_per_sender: u64,
    /// Maximal memory usage.
    pub max_mem_usage: u64,
    /// Minimal allowed gas price.
    pub minimal_gas_price: u64,
    /// Maximal gas limit for a single transaction.
    #[serde(skip)]
    pub tx_gas_limit: u64,
}

impl Default for TxPoolConfig {
    fn default() -> Self {
        Self {
            max_count: 1024,
            max_per_sender: 16,
            max_mem_usage: 64 * 1024 * 1024, // 64M
            minimal_gas_price: 0,
            tx_gas_limit: u64::max_value(),
        }
    }
}

impl TxPoolConfig {
    pub fn random_for_test() -> Self {
        Self::default()
    }

    pub fn load(&mut self) -> Result<()> {
        // TODO: add validate logic
        Ok(())
    }
}
