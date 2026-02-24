use starcoin_types::U256;

pub mod codec;
pub mod diff_manager;
pub mod pplns_store;
pub mod rpc;

#[derive(Debug, Clone)]
pub struct StratumLimits {
    pub share_dedup_window_secs: u64,
    pub stale_window_secs: u64,
    pub share_rate_window_secs: u64,
    pub max_shares_per_window: u32,
    pub max_invalid_shares: u32,
    pub max_job_misses: u32,
    pub max_stale_shares: u32,
    pub max_workers_per_account: usize,
}

#[derive(Debug, Clone)]
pub struct StratumPplnsConfig {
    pub enabled: bool,
    pub ingest_enabled: bool,
    pub settlement_enabled: bool,
    pub window_shares: u64,
    pub confirmations: u64,
    pub settlement_interval_secs: u64,
    pub batch_period_secs: u64,
    pub max_retained_shares: u64,
    pub max_retained_candidates: usize,
    pub database_url: Option<String>,
}

pub fn difficulty_to_target_hex(difficulty: U256) -> String {
    let target = format!("{:x}", U256::from(u64::MAX) / difficulty);
    let mut temp = "0".repeat(16 - target.len());
    temp.push_str(&target);
    let mut t = hex::decode(temp).expect("Decode target never failed");
    t.reverse();
    hex::encode(&t)
}

pub fn target_hex_to_difficulty(target: &str) -> anyhow::Result<U256> {
    let mut temp = hex::decode(target)?;
    temp.reverse();
    let temp = hex::encode(temp);
    let temp = U256::from_str_radix(&temp, 16)?;
    Ok(U256::from(u64::MAX) / temp)
}
