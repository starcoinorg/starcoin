use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use starcoin_logger::prelude::*;
use starcoin_stratumd::node_rpc::build_async_rpc_client;
use starcoin_stratumd::pplns::PplnsRuntime;
use starcoin_stratumd::{StratumLimits, StratumPplnsConfig};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

mod gateway;
mod server;
mod verify_settlement;

use gateway::App;
use server::run_stratum_server;
use verify_settlement::{run_verify_settlement, VerifySettlementOpt};

#[derive(Parser, Debug, Clone)]
#[command(name = "starcoin_stratumd")]
#[command(about = "Standalone Stratum gateway process")]
struct Cli {
    #[clap(subcommand)]
    command: Option<Command>,

    #[clap(flatten)]
    run: RunOpt,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    VerifySettlement(VerifySettlementOpt),
}

#[derive(Args, Debug, Clone)]
struct RunOpt {
    #[arg(long, default_value = "0.0.0.0:9888")]
    listen: std::net::SocketAddr,

    #[arg(long, default_value = "ws://127.0.0.1:9870")]
    node_rpc: String,

    #[arg(long, default_value_t = 500)]
    job_poll_ms: u64,

    #[arg(long, default_value_t = 600)]
    share_dedup_window_secs: u64,

    #[arg(long, default_value_t = 120)]
    stale_window_secs: u64,

    #[arg(long, default_value_t = 10)]
    share_rate_window_secs: u64,

    #[arg(long, default_value_t = 200)]
    max_shares_per_window: u32,

    #[arg(long, default_value_t = 60)]
    max_invalid_shares: u32,

    #[arg(long, default_value_t = 120)]
    max_job_misses: u32,

    #[arg(long, default_value_t = 300)]
    max_stale_shares: u32,

    #[arg(long, default_value_t = 1024)]
    max_workers_per_account: usize,

    #[arg(long, default_value_t = false)]
    pplns_enabled: bool,

    #[arg(long, default_value_t = 20_000)]
    pplns_window_shares: u64,

    #[arg(long, default_value_t = 6)]
    pplns_confirmations: u64,

    #[arg(long, default_value_t = 10)]
    pplns_settlement_interval_secs: u64,

    #[arg(long, default_value_t = 3_600)]
    pplns_batch_period_secs: u64,

    #[arg(long, default_value_t = 160_000)]
    pplns_max_retained_shares: u64,

    #[arg(long, default_value_t = 4_096)]
    pplns_max_retained_candidates: usize,

    #[arg(long)]
    pplns_database_url: Option<String>,
}

fn main() -> Result<()> {
    let _logger = starcoin_logger::init();
    let cli = Cli::parse();

    if let Some(Command::VerifySettlement(verify_opt)) = cli.command {
        return run_verify_settlement(&verify_opt);
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_gateway(cli.run))
}

async fn run_gateway(opt: RunOpt) -> Result<()> {
    let limits = StratumLimits {
        share_dedup_window_secs: opt.share_dedup_window_secs,
        stale_window_secs: opt.stale_window_secs,
        share_rate_window_secs: opt.share_rate_window_secs,
        max_shares_per_window: opt.max_shares_per_window,
        max_invalid_shares: opt.max_invalid_shares,
        max_job_misses: opt.max_job_misses,
        max_stale_shares: opt.max_stale_shares,
        max_workers_per_account: opt.max_workers_per_account,
    };
    info!(
        target: "stratum_server",
        "limits: dedup={}s stale={}s rate={}s max_rate={} max_invalid={} max_job_miss={} max_stale={} max_workers={}",
        limits.share_dedup_window_secs,
        limits.stale_window_secs,
        limits.share_rate_window_secs,
        limits.max_shares_per_window,
        limits.max_invalid_shares,
        limits.max_job_misses,
        limits.max_stale_shares,
        limits.max_workers_per_account
    );

    let rpc = build_async_rpc_client(&opt.node_rpc).await?;
    let pplns = build_pplns_runtime(&opt)?;
    let app = App::new(
        rpc,
        limits,
        Duration::from_millis(opt.job_poll_ms.max(100)),
        pplns,
    );

    let poll_app = app.clone();
    tokio::spawn(async move {
        poll_app.run_job_subscribe_loop().await;
    });

    if app.settlement_enabled().await {
        let settlement_app = app.clone();
        let interval_secs = opt.pplns_settlement_interval_secs;
        tokio::spawn(async move {
            settlement_app
                .run_pplns_settlement_loop(interval_secs)
                .await;
        });
    }

    run_stratum_server(opt.listen, app).await
}

fn build_pplns_runtime(opt: &RunOpt) -> Result<Option<Arc<Mutex<PplnsRuntime>>>> {
    if !opt.pplns_enabled {
        return Ok(None);
    }
    let config = StratumPplnsConfig {
        enabled: true,
        ingest_enabled: true,
        settlement_enabled: true,
        window_shares: opt.pplns_window_shares.max(1),
        confirmations: opt.pplns_confirmations.max(1),
        settlement_interval_secs: opt.pplns_settlement_interval_secs.max(1),
        batch_period_secs: opt.pplns_batch_period_secs.max(60),
        max_retained_shares: opt
            .pplns_max_retained_shares
            .max(opt.pplns_window_shares.max(1)),
        max_retained_candidates: opt.pplns_max_retained_candidates.max(64),
        database_url: opt.pplns_database_url.clone(),
    };
    let runtime = PplnsRuntime::new(config)?;
    info!(
        target: "stratum_server",
        "pplns enabled: ingest={}, settlement={}, window_shares={}, confirmations={}, interval_secs={}, batch_period_secs={}",
        runtime.ingest_enabled(),
        runtime.settlement_enabled(),
        runtime.config().window_shares,
        runtime.config().confirmations,
        runtime.config().settlement_interval_secs,
        runtime.config().batch_period_secs
    );
    Ok(Some(Arc::new(Mutex::new(runtime))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_stratumd::pplns_store::{CandidateRecord, CandidateStatus, ShareRecord};
    use std::collections::HashMap;

    fn candidate(account: &str) -> CandidateRecord {
        CandidateRecord {
            block_hash: "0x1".to_string(),
            block_number: 1,
            account: account.to_string(),
            worker_id: "w1".to_string(),
            anchor_share_seq: 1,
            found_at_millis: 1,
            status: CandidateStatus::Pending,
            reward: None,
            settled_at_millis: None,
        }
    }

    fn share(account: &str, difficulty: u64) -> ShareRecord {
        ShareRecord {
            seq: 1,
            account: account.to_string(),
            worker_id: "w".to_string(),
            difficulty,
            accepted_at_millis: 1,
        }
    }

    fn sum_credits(credits: &HashMap<String, u128>) -> u128 {
        credits.values().copied().sum()
    }

    #[test]
    fn test_allocate_credits_weighted_sum_matches_reward() {
        let credits = PplnsRuntime::allocate_credits(
            &candidate("winner"),
            &[share("a", 1), share("b", 3)],
            1_000,
        );
        assert_eq!(credits.get("a"), Some(&250));
        assert_eq!(credits.get("b"), Some(&750));
        assert_eq!(sum_credits(&credits), 1_000);
    }

    #[test]
    fn test_allocate_credits_remainder_goes_to_candidate_account() {
        let credits = PplnsRuntime::allocate_credits(
            &candidate("winner"),
            &[share("a", 1), share("b", 1)],
            101,
        );
        assert_eq!(credits.get("a"), Some(&50));
        assert_eq!(credits.get("b"), Some(&50));
        assert_eq!(credits.get("winner"), Some(&1));
        assert_eq!(sum_credits(&credits), 101);
    }

    #[test]
    fn test_allocate_credits_empty_window_falls_back_to_candidate() {
        let credits = PplnsRuntime::allocate_credits(&candidate("winner"), &[], 777);
        assert_eq!(credits.get("winner"), Some(&777));
        assert_eq!(sum_credits(&credits), 777);
    }
}
