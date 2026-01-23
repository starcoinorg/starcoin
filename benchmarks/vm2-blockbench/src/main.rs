use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use clap::{Parser, ValueHint};
use starcoin_logger::prelude::*;
use starcoin_node::node::NodeService;
use starcoin_node_api::node_service::NodeAsyncService;
use starcoin_service_registry::bus::BusService;
use starcoin_service_registry::RegistryAsyncService;
use tokio::runtime::Builder;
use tokio::time::{sleep, Duration};
use vm2_blockbench::{load_configs, prepare_accounts, prepare_transfers, DataDir};

#[derive(Debug, Parser)]
#[command(about = "vm2-blockbench: prepare accounts and funding")]
struct Cli {
    #[arg(short = 'p', long = "path", value_hint = ValueHint::DirPath, help = "data dir (default ~/.starcoin/vm2-blockbench)")]
    data_dir: Option<PathBuf>,

    #[arg(
        long = "submit-batch",
        default_value = "2000",
        help = "transactions per txpool submission batch"
    )]
    submit_batch: usize,

    #[arg(
        long = "total-txns",
        default_value = "20000",
        help = "total transfer txns to submit (across batches)"
    )]
    total_txns: usize,

    #[arg(
        long = "account-count",
        help = "total accounts to prepare (default submit_batch * 2)"
    )]
    account_count: Option<u32>,

    #[arg(
        long = "gas-price",
        default_value = "1",
        help = "gas price for funding txns"
    )]
    gas_price: u64,

    #[arg(
        long = "max-gas",
        default_value = "40000000",
        help = "max gas for funding txns (clamped to chain limit)"
    )]
    max_gas: u64,

    #[arg(
        long = "vm-concurrency",
        help = "override VM concurrency (default num_cpus)"
    )]
    vm_concurrency: Option<usize>,

    #[arg(
        long = "tps-block-window",
        default_value = "0",
        help = "number of latest confirmed blocks used to compute window TPS (0 to disable)"
    )]
    tps_block_window: u64,

    #[arg(
        long = "post-prepare-start-from-head",
        default_value_t = true,
        help = "count confirmed txns only from the head block right after account preparation"
    )]
    post_prepare_start_from_head: bool,

    #[arg(
        long = "mint-per-batch",
        default_value_t = true,
        help = "trigger block production immediately after each submit batch"
    )]
    mint_per_batch: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let data_dir = DataDir::new(cli.data_dir)?;
    let mut node_config = load_configs(&data_dir)?;
    // Derive workload parameters up front so config tweaks take effect before node launch.
    let submit_batch = cli.submit_batch.max(1);
    let total_txns = cli.total_txns.max(1);
    let derived_account_count = u32::try_from(submit_batch.saturating_mul(2)).unwrap_or(u32::MAX);
    let account_count = cli.account_count.unwrap_or(derived_account_count).max(1);
    // Bump txpool limits to accommodate the intended workload.
    let cfg_mut = Arc::make_mut(&mut node_config);
    let desired_max = (total_txns as u64).saturating_add(submit_batch as u64);
    let max_count = desired_max.max(100_000);
    cfg_mut.tx_pool.set_max_count(max_count);
    cfg_mut.tx_pool.set_tx_propagate_interval(1);
    cfg_mut
        .tx_pool
        .set_max_per_sender((total_txns as u64).max(100_000));

    // Default: bench target at info, Starcoin modules at warn, global warn.
    let default_filter = "vm2-blockbench=info,warn".to_string();
    let _log_spec = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        std::env::set_var("RUST_LOG", default_filter.clone());
        default_filter.clone()
    });
    // `default_level` here is just a fallback; actual levels come from RUST_LOG.
    let logger_handle = starcoin_logger::init_with_default_level("warn", None);
    let node = NodeService::launch(node_config.clone(), logger_handle)?;

    // Use a dedicated runtime for async orchestration, then drop it before stopping the node
    // to avoid dropping nested runtimes inside an async context.
    let rt = Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(async {
        node.node_service().stop_pacemaker().await?;
        let registry = node.registry();

        let concurrency = cli.vm_concurrency.unwrap_or_else(num_cpus::get);
        starcoin_vm2_vm_runtime::starcoin_vm::StarcoinVM::set_concurrency_level(concurrency);

        // Respect chain maximum gas units (40M on Proxima configs).
        let chain_max_gas: u64 = 40_000_000;
        let max_gas = cli.max_gas.min(chain_max_gas);

        // Bump txpool limits to accommodate the intended workload.
        let cfg_mut = Arc::make_mut(&mut node_config);
        // cap at 100_000 by default unless total_txns is even higher.
        let desired_max = (total_txns as u64).saturating_add(submit_batch as u64);
        let max_count = desired_max.max(100_000);
        cfg_mut.tx_pool.set_max_count(max_count);

        let account_svc = registry
            .service_ref::<starcoin_vm2_account_service::AccountService>()
            .await?;
        let chain_reader = registry
            .service_ref::<starcoin_chain_service::ChainReaderService>()
            .await?;
        let bus = registry.service_ref::<BusService>().await?;
        let miner = if cli.mint_per_batch {
            Some(registry.service_ref::<starcoin_miner::MinerService>().await?)
        } else {
            None
        };
        let txpool = registry
            .get_shared::<starcoin_txpool::TxPoolService>()
            .await?;
        let storage1 = registry
            .get_shared::<Arc<starcoin_storage::Storage>>()
            .await?;
        let storage2 = registry
            .get_shared::<Arc<starcoin_storage::Storage2>>()
            .await?;

        node.node_service().start_pacemaker().await?;

        // Capture the head block number right after account preparation if requested.
        let mut post_prepare_start_block: Option<u64> = None;

        info!(
            target: "vm2-blockbench",
            "prepare accounts: count={} (derived={} submit_batch={}) gas_price={} max_gas={}",
            account_count,
            derived_account_count,
            submit_batch,
            cli.gas_price,
            cli.max_gas
        );
        let accounts = prepare_accounts(
            account_count,
            cli.gas_price,
            max_gas,
            node_config.clone(),
            account_svc.clone(),
            chain_reader.clone(),
            txpool.clone(),
            storage1.clone(),
            storage2.clone(),
        )
            .await?;
        if cli.post_prepare_start_from_head {
            let head = vm2_blockbench::head_block(chain_reader.clone()).await?;
            post_prepare_start_block = Some(head.header.number());
        }

        info!(target: "vm2-blockbench", "accounts ready: {}", accounts.len());
        if cli.mint_per_batch {
            // Switch to manual block triggers so each submit batch mints immediately.
            node.node_service().stop_pacemaker().await?;
        }
        let transfer_amount: u128 = 1_000_000; // micro-STC
        let transfer_max_gas = 1_000_000u64.min(chain_max_gas);
        let transfer_gas_price = cli.gas_price.max(1);
        let transfer_stats = prepare_transfers(
            &accounts,
            transfer_amount,
            transfer_gas_price,
            transfer_max_gas,
            submit_batch,
            total_txns,
            cli.tps_block_window,
            post_prepare_start_block,
            node_config.clone(),
            account_svc,
            chain_reader.clone(),
            bus,
            txpool,
            miner,
            storage1,
            storage2,
        )
        .await?;
        info!(
            target: "vm2-blockbench",
            "transfer txns submitted: {}, executed: {}, duration_secs: {:.3}, tps: {:.1}",
            transfer_stats.submitted,
            transfer_stats.executed,
            transfer_stats.duration_secs,
            transfer_stats.tps
        );
        if let Some(window) = transfer_stats.block_window {
            info!(
                target: "vm2-blockbench",
                "recent {} blocks: txns={}, duration_secs: {:.3}, tps: {:.1}",
                window.block_count,
                window.txn_count,
                window.duration_secs,
                window.tps
            );
        }
        if let Some(post) = transfer_stats.post_prepare_blocks {
            info!(
                target: "vm2-blockbench",
                "blocks after account prep ({} -> {}]: txns={}",
                post.start_block,
                post.end_block,
                post.txn_count
            );
        }

        // Stop block production/miner pipeline explicitly to avoid late blocks during shutdown.
        stop_service_quiet(
            &node,
            "starcoin_miner::generate_block_event_pacemaker::GenerateBlockEventPacemaker",
        )
        .await;
        stop_service_quiet(
            &node,
            "starcoin_miner::create_block_template::block_builder_service::BlockBuilderService",
        )
        .await;
        stop_service_quiet(
            &node,
            "starcoin_miner::create_block_template::new_header_service::NewHeaderService",
        )
        .await;
        stop_service_quiet(&node, "starcoin_miner::MinerService").await;
        stop_service_quiet(
            &node,
            "starcoin_miner_client::miner::MinerClientService<starcoin_miner_client::job_bus_client::JobBusClient>",
        )
        .await;

        // Give the pipeline a short window to flush any in-flight commits before shutdown.
        sleep(Duration::from_millis(500)).await;
        Ok::<(), anyhow::Error>(())
    })?;
    drop(rt);

    node.stop()?;
    Ok(())
}

async fn stop_service_quiet(node: &starcoin_node::NodeHandle, name: &str) {
    if let Err(e) = node.node_service().stop_service(name.to_string()).await {
        warn!("stop service {} error: {:?}", name, e);
    }
}
