mod agent_loop;
mod analyzer;
mod results;

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use anyhow::{bail, format_err, Result};
use chrono::Local;
use clap::{Parser, ValueEnum, ValueHint};
use results::{ResultsDumper, TransactionExecutionResult};
use starcoin_chain_api::message::{ChainRequest, ChainResponse};
use starcoin_chain_service::ChainReaderService;
use starcoin_config::{BaseConfig, BuiltinNetworkID, ChainNetworkID, NodeConfig, StarcoinOpt};
use starcoin_config::{G_DEV_CONFIG, G_HALLEY_CONFIG, G_PROXIMA_CONFIG};
use starcoin_crypto::HashValue;
use starcoin_logger::{
    prelude::{error, info, warn, LevelFilter},
    LoggerHandle,
};
use starcoin_node::NodeHandle;
use starcoin_pipeline_timing::{clear_timing, disable_timing, enable_timing, global_collector};
use starcoin_service_registry::{
    ActorService, EventHandler, RegistryAsyncService, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_storage::{BlockStore, Storage, Storage2, Store};
use starcoin_transaction_builder::vm2::{
    build_batch_transfer_txn as build_batch_transfer_txn2, raw_peer_to_peer_txn,
};
use starcoin_txpool::TxStatus;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::{
    block::{Block, BlockHeader},
    genesis_config::ChainId,
    multi_transaction::MultiSignedUserTransaction,
    system_events::{MinedBlock, NewHeadBlock},
    transaction::StcTransactionInfo,
};
use starcoin_vm2_account_api::{
    message::{AccountRequest, AccountResponse},
    AccountInfo,
};
use starcoin_vm2_account_service::AccountService as AccountService2;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::{
    account_config::{association_address, G_STC_TOKEN_CODE},
    genesis_config::ChainId as ChainId2,
    transaction::{RawUserTransaction as RawUserTransaction2, SignedUserTransaction},
};
use starcoin_vm2_vm_runtime::starcoin_vm::StarcoinVM;
use starcoin_vm2_vm_types::{account_address::AccountAddress, state_view::StateReaderExt};
use tempfile::TempDir;
use test_helper::run_node_with_all_service;

#[derive(Debug, Parser)]
#[command(about = "Execute the full build-and-execute benchmark outside of tests.")]
struct Cli {
    #[arg(short = 'p', long = "path", value_hint = ValueHint::DirPath)]
    data_dir: Option<PathBuf>,

    #[arg(
        short = 'n',
        long = "network",
        value_name = "NETWORK",
        default_value = "custom",
        value_parser = parse_network_choice,
        help = "Network to run against (custom, test, dev, halley, proxima, barnard, main)."
    )]
    network: NetworkChoice,

    #[arg(
        long = "custom-template",
        value_enum,
        default_value = "halley",
        help = "Template used to generate custom genesis when --network custom (halley or proxima)."
    )]
    custom_template: CustomGenesisTemplate,

    #[arg(
        short = 'c',
        long = "account-count",
        default_value = "4000",
        help = "Number of accounts to create for the benchmark. Higher values (8000+) give more stable TPS results but take longer to fund."
    )]
    account_count: u32,

    #[arg(
        short = 'b',
        long = "initial-balance",
        default_value = "10000000000",
        help = "Initial balance for each account."
    )]
    initial_balance: u128,

    #[arg(
        short = 'g',
        long = "initial-gas-fee",
        default_value = "4000000000",
        help = "Initial gas fee for transactions."
    )]
    initial_gas_fee: u128,

    #[arg(
        long = "gas-price",
        default_value = "1",
        help = "Gas price for transactions."
    )]
    gas_price: u64,

    #[arg(
        long = "max-gas",
        default_value = "40000000",
        help = "Max gas for transactions."
    )]
    max_gas: u64,

    #[arg(
        long = "batch-user-count",
        default_value = "4000",
        help = "Number of users per batch. Each batch uses different users (0 to batch_user_count-1 in first batch, etc). Within each batch, first half sends to second half."
    )]
    batch_user_count: usize,

    #[arg(
        long = "balance-wait-timeout-secs",
        default_value = "600",
        help = "Timeout in seconds while waiting for association balance to become sufficient."
    )]
    balance_wait_timeout_secs: u64,

    #[arg(
        long = "settle-delay-ms",
        default_value = "10000",
        help = "Delay in milliseconds after funding transfers before benchmark starts."
    )]
    settle_delay_ms: u64,

    #[arg(
        long = "preload-batches",
        default_value = "0",
        help = "Number of batches to preload into txpool at start. 0 = preload all batches."
    )]
    preload_batches: usize,

    #[arg(
        long = "txpool-max-count",
        default_value = "0",
        help = "Maximum number of transactions in txpool. 0 = auto (based on account_count)."
    )]
    txpool_max_count: u64,

    #[arg(
        long = "simple-transfer",
        default_value = "false",
        help = "Use simple P2P transfer instead of batch transfer. Lower gas (~700K vs ~13.8M), allowing more txns per block."
    )]
    simple_transfer: bool,

    #[arg(
        long = "agent-mode",
        default_value = "false",
        help = "Run in agent mode with full analysis output (bottleneck detection, optimization suggestions, regression detection)."
    )]
    agent_mode: bool,


}

fn parse_network_choice(value: &str) -> Result<NetworkChoice, String> {
    if value.eq_ignore_ascii_case("custom") {
        return Ok(NetworkChoice::Custom);
    }
    let normalized = value.to_lowercase();
    BuiltinNetworkID::from_str(&normalized)
        .map(NetworkChoice::Builtin)
        .map_err(|_| {
            format!(
                "Unsupported network '{}'. Use one of: custom, test, dev, halley, proxima, barnard, main",
                value
            )
        })
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum NetworkChoice {
    Builtin(BuiltinNetworkID),
    Custom,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum CustomGenesisTemplate {
    Halley,
    Proxima,
}

impl CustomGenesisTemplate {
    fn as_str(self) -> &'static str {
        match self {
            Self::Halley => "halley",
            Self::Proxima => "proxima",
        }
    }
}

impl NetworkChoice {
    fn to_chain_network(self) -> Result<ChainNetworkID> {
        match self {
            Self::Custom => ChainNetworkID::new_custom("my_chain".to_owned(), ChainId::new(121)),
            Self::Builtin(builtin_network_id) => Ok(match builtin_network_id {
                BuiltinNetworkID::Test => ChainNetworkID::Builtin(BuiltinNetworkID::Test),
                BuiltinNetworkID::Dev => ChainNetworkID::Builtin(BuiltinNetworkID::Dev),
                BuiltinNetworkID::Halley => ChainNetworkID::Builtin(BuiltinNetworkID::Halley),
                BuiltinNetworkID::Proxima => ChainNetworkID::Builtin(BuiltinNetworkID::Proxima),
                BuiltinNetworkID::Barnard => ChainNetworkID::Builtin(BuiltinNetworkID::Barnard),
                BuiltinNetworkID::Main => ChainNetworkID::Builtin(BuiltinNetworkID::Main),
            }),
        }
    }

    fn genesis_name(self) -> Option<&'static str> {
        match self {
            Self::Custom => None,
            Self::Builtin(builtin_network_id) => match builtin_network_id {
                BuiltinNetworkID::Test => Some("test"),
                BuiltinNetworkID::Dev => Some("dev"),
                BuiltinNetworkID::Halley => Some("halley"),
                BuiltinNetworkID::Proxima => Some("proxima"),
                BuiltinNetworkID::Barnard => Some("barnard"),
                BuiltinNetworkID::Main => Some("main"),
            },
        }
    }
}

struct DataDir {
    path: PathBuf,
    temp_dir: Option<TempDir>,
}

impl DataDir {
    fn new(path: Option<PathBuf>) -> Result<Self> {
        if let Some(path) = path {
            std::fs::create_dir_all(&path)?;
            Ok(Self {
                path,
                temp_dir: None,
            })
        } else {
            let temp_dir = tempfile::tempdir()?;
            Ok(Self {
                path: temp_dir.path().to_path_buf(),
                temp_dir: Some(temp_dir),
            })
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn close(self) -> Result<()> {
        if let Some(temp_dir) = self.temp_dir {
            temp_dir.close()?;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let network_choice = cli.network;
    let chain_network = network_choice.to_chain_network()?;
    let funding_batch_size = 10usize;
    let estimated_funding_txns = (cli.account_count as usize).div_ceil(funding_batch_size);
    let txpool_max_per_sender_for_bench = (estimated_funding_txns as u64 + 32).max(128);
    let data_dir = DataDir::new(cli.data_dir)?;
    let base_dir = data_dir.path().to_path_buf();
    let custom_genesis = if matches!(network_choice, NetworkChoice::Custom) {
        Some(prepare_custom_genesis_template(
            &base_dir,
            cli.custom_template,
        )?)
    } else {
        None
    };
    if matches!(network_choice, NetworkChoice::Custom) {
        let chain_dir = base_dir.join(chain_network.chain_name());
        if chain_dir.exists() {
            info!(
                "Removing existing custom chain data directory: {:?}",
                chain_dir
            );
            std::fs::remove_dir_all(&chain_dir)?;
        }
    }

    let mut init_opt = StarcoinOpt {
        net: Some(chain_network.clone()),
        base_data_dir: Some(base_dir.clone()),
        ..Default::default()
    };
    init_opt
        .txpool
        .set_max_per_sender(txpool_max_per_sender_for_bench);
    // Set txpool max_count: if 0, auto-calculate based on account_count
    let txpool_max_count = if cli.txpool_max_count == 0 {
        // Auto: account_count / 2 (benchmark txns) + account_count / 10 (funding txns) + buffer
        let auto_count = (cli.account_count as u64 / 2) + (cli.account_count as u64 / 10) + 1000;
        auto_count.max(4096) // At least default
    } else {
        cli.txpool_max_count
    };
    init_opt.txpool.set_max_count(txpool_max_count);
    println!("[Benchmark] Setting txpool max_count to {}", txpool_max_count);
    init_opt.genesis_config = match custom_genesis {
        Some(path) => Some(path),
        None => network_choice.genesis_name().map(ToOwned::to_owned),
    };
    BaseConfig::load_with_opt(&init_opt)?;

    let mut global_opt = StarcoinOpt {
        net: Some(chain_network),
        base_data_dir: Some(base_dir),
        ..Default::default()
    };
    global_opt
        .txpool
        .set_max_per_sender(txpool_max_per_sender_for_bench);
    global_opt.txpool.set_max_count(txpool_max_count);
    global_opt.genesis_config = init_opt.genesis_config.clone();

    let node_config = Arc::new(NodeConfig::load_with_opt(&global_opt)?);
    let node = run_node_with_all_service(node_config.clone())?;

    // Enable VM parallel execution early (before any transactions)
    StarcoinVM::set_concurrency_level(num_cpus::get());

    // Wait for node services to fully initialize (sync status, pacemaker, etc.)
    info!("Waiting for node services to fully initialize...");
    std::thread::sleep(std::time::Duration::from_secs(3));
    info!("Node initialization wait complete.");

    // Enable pipeline timing collection before benchmark
    clear_timing();
    enable_timing();
    info!("[Pipeline Timing] Enabled timing collection");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let bench_result = rt.block_on(execute_benchmark(
        &node,
        cli.account_count,
        cli.initial_balance,
        cli.initial_gas_fee,
        cli.gas_price,
        cli.max_gas,
        cli.batch_user_count,
        cli.balance_wait_timeout_secs,
        cli.settle_delay_ms,
        cli.preload_batches,
        cli.simple_transfer,
    ));

    // Disable pipeline timing collection after benchmark
    disable_timing();
    info!("[Pipeline Timing] Disabled timing collection");

    // Print pipeline timing statistics
    let timing_stats = global_collector().calculate_stage_stats();
    info!("[Pipeline Timing] Stage Statistics:");
    for (stage, stats) in &timing_stats {
        if stats.count > 0 {
            info!(
                "  {}: count={}, avg={:.3}ms, min={:.3}ms, max={:.3}ms, throughput={:.2} txns/s",
                stage, stats.count, stats.avg_ms, stats.min_ms, stats.max_ms, stats.throughput
            );
        }
    }

    // Convert timing stats to HashMap<String, StageTiming> for agent loop (before node.stop() clears data)
    let pipeline_stages: std::collections::HashMap<String, starcoin_pipeline_timing::StageTiming> = 
        timing_stats.into_iter().map(|(k, v)| (k.to_string(), v)).collect();

    // Stop node first - this triggers ObserverService.stopped() which exports benchmark_results.json
    node.stop()?;

    // Run agent loop analysis if enabled (after node.stop() so benchmark_results.json is available)
    if cli.agent_mode {
        info!("[Agent Mode] Starting analysis...");
        
        // Read actual stats from benchmark_results.json (exported by ObserverService.stopped())
        let stats = match std::fs::read_to_string("./benchmark_results.json") {
            Ok(json_content) => {
                match serde_json::from_str::<results::BenchmarkJsonOutput>(&json_content) {
                    Ok(output) => {
                        info!("[Agent Mode] Loaded stats from benchmark_results.json: TPS={:.2}", output.summary.tps);
                        output.summary
                    }
                    Err(e) => {
                        warn!("[Agent Mode] Failed to parse benchmark_results.json: {}, using defaults", e);
                        results::BenchmarkStats::default()
                    }
                }
            }
            Err(e) => {
                warn!("[Agent Mode] Failed to read benchmark_results.json: {}, using defaults", e);
                results::BenchmarkStats::default()
            }
        };
        
        // Generate stateless analysis output
        let output = agent_loop::BenchmarkOutput::from_stats(stats, pipeline_stages);
        info!("\n{}", output);
    }

    let close_result = data_dir.close();
    bench_result?;
    close_result?;
    Ok(())
}

fn prepare_custom_genesis_template(
    base_dir: &Path,
    template: CustomGenesisTemplate,
) -> Result<String> {
    use starcoin_config::genesis_config::vm2;

    let mut genesis_config = match template {
        CustomGenesisTemplate::Halley => G_HALLEY_CONFIG.clone(),
        CustomGenesisTemplate::Proxima => G_PROXIMA_CONFIG.clone(),
    };
    let mut genesis_config2 = match template {
        CustomGenesisTemplate::Halley => vm2::G_HALLEY_CONFIG.clone(),
        CustomGenesisTemplate::Proxima => vm2::G_PROXIMA_CONFIG.clone(),
    };

    // Keep halley/proxima runtime parameters, but inject local association private keys
    // from dev template so the benchmark can batch-fund test accounts.
    genesis_config.association_key_pair = G_DEV_CONFIG.association_key_pair.clone();
    genesis_config2.association_key_pair = vm2::G_DEV_CONFIG.association_key_pair.clone();

    let genesis_path = base_dir.join(format!("bench-custom-template-{}.json", template.as_str()));
    let genesis_path2 = PathBuf::from(format!("{}.2", genesis_path.display()));
    genesis_config.save(genesis_path.as_path())?;
    genesis_config2.save(genesis_path2.as_path())?;
    Ok(genesis_path.to_string_lossy().into_owned())
}

async fn create_account(
    account_number: u32,
    account_service: ServiceRef<AccountService2>,
) -> Result<Vec<AccountInfo>> {
    let mut results = vec![];
    for _ in 0..account_number {
        let receiver = match account_service
            .send(AccountRequest::CreateAccount("".to_string()))
            .await??
        {
            AccountResponse::AccountInfo(account) => *account,
            _ => bail!("Unexpected response type."),
        };
        results.push(receiver);
    }

    Ok(results)
}

/// Get the default account from account service (kept for future use)
#[allow(dead_code)]
async fn default_account(account_service: ServiceRef<AccountService2>) -> Result<AccountInfo> {
    let default_account = match account_service
        .send(AccountRequest::GetDefaultAccount())
        .await??
    {
        AccountResponse::AccountInfoOption(account) => {
            account.ok_or_else(|| format_err!("default account not exist"))?
        }
        _ => bail!("Unexpected response type."),
    };

    Ok(default_account)
}

/// Build and sign transactions using association account from genesis config
/// Batches multiple receivers into single transactions for efficiency
async fn build_association_transfer_transactions(
    receivers: &[AccountInfo],
    amount: u128,
    gas_price: u64,
    max_gas: u64,
    config: Arc<NodeConfig>,
    header_block: &BlockHeader,
    storage1: Arc<Storage>,
    storage2: Arc<Storage2>,
    batch_size: usize,
) -> Result<Vec<SignedUserTransaction>> {
    let expire_time = config.net().time_service().now_secs() + 3600;
    let multi_state = storage1.get_vm_multi_state(header_block.id())?;
    let statedb2 = ChainStateDB::new(storage2.clone(), Some(multi_state.state_root2()));

    let sender_address = association_address();
    let mut next_seq = statedb2
        .get_account_resource(sender_address)?
        .sequence_number();

    let genesis_config2 = config.net().genesis_config2();

    let mut signed_transactions = vec![];

    // Batch receivers into chunks
    for chunk in receivers.chunks(batch_size) {
        let receiver_addresses: Vec<AccountAddress> = chunk.iter().map(|r| r.address).collect();
        let raw_txn = build_batch_transfer_txn2(
            sender_address,
            receiver_addresses,
            next_seq,
            amount,
            gas_price,
            max_gas,
            expire_time,
            header_block.chain_id().id().into(),
        );
        let signed_txn = genesis_config2.sign_with_association(raw_txn)?;
        signed_transactions.push(signed_txn);
        next_seq += 1;
    }

    info!(
        "Built {} batch transactions for {} receivers (batch_size={})",
        signed_transactions.len(),
        receivers.len(),
        batch_size
    );

    Ok(signed_transactions)
}

async fn get_current_header(
    chain_reader_service: ServiceRef<ChainReaderService>,
) -> Result<BlockHeader> {
    let current_header = match chain_reader_service
        .send(ChainRequest::CurrentHeader())
        .await??
    {
        ChainResponse::BlockHeader(header) => *header,
        _ => bail!("Unexpected response type."),
    };
    Ok(current_header)
}

async fn get_txn_status_debug(
    chain_reader_service: ServiceRef<ChainReaderService>,
    txn_hash: HashValue,
) -> Result<Option<String>> {
    let txn_info = match chain_reader_service
        .send(ChainRequest::GetTransactionInfo(txn_hash))
        .await??
    {
        ChainResponse::TransactionInfo(info) => info,
        _ => bail!("Unexpected response type."),
    };

    Ok(txn_info.map(|info| match info.transaction_info {
        StcTransactionInfo::V1(txn_info) => format!("{:?}", txn_info.status()),
        StcTransactionInfo::V2(txn_info) => format!("{:?}", txn_info.status()),
    }))
}

async fn get_balance(
    address: AccountAddress,
    storage1: Arc<Storage>,
    storage2: Arc<Storage2>,
    header_id: HashValue,
) -> Result<u128> {
    let multi_state = storage1.get_vm_multi_state(header_id)?;
    let statedb2 = ChainStateDB::new(storage2.clone(), Some(multi_state.state_root2()));
    let balance = statedb2.get_balance_by_type(address, G_STC_TOKEN_CODE.clone().try_into()?)?;
    Ok(balance)
}

async fn wait_for_sufficient_balance(
    account_count: u32,
    initial_balance: u128,
    initial_gas_fee: u128,
    gas_price: u64,
    max_gas: u64,
    chain_reader_service: ServiceRef<ChainReaderService>,
    storage1: Arc<Storage>,
    storage2: Arc<Storage2>,
    timeout: Duration,
) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if Instant::now() >= deadline {
            bail!(
                "timed out waiting for sufficient association balance after {:?}",
                timeout
            );
        }
        let current_header = get_current_header(chain_reader_service.clone()).await?;
        let association_balance = match get_balance(
            association_address(),
            storage1.clone(),
            storage2.clone(),
            current_header.id(),
        )
        .await
        {
            Ok(balance) => balance,
            Err(e) => {
                info!(
                    "get balance error: {} and waiting for the token initialization",
                    e
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
                continue;
            }
        };
        let per_tx_fee = max_gas as u128 * gas_price as u128;
        let needed_balance =
            account_count as u128 * (initial_balance + per_tx_fee) + initial_gas_fee;
        if association_balance >= needed_balance {
            info!(
                "Association account has sufficient balance: {} >= {}",
                association_balance, needed_balance
            );
            break;
        }
        info!(
            "Waiting for sufficient balance: {} < {}",
            association_balance, needed_balance
        );
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
    }
    Ok(())
}

async fn transfer_to_accounts(
    receivers: &[AccountInfo],
    initial_balance: u128,
    gas_price: u64,
    max_gas: u64,
    config: Arc<NodeConfig>,
    chain_reader_service: ServiceRef<ChainReaderService>,
    txpool: &starcoin_txpool::TxPoolService,
    storage1: Arc<Storage>,
    storage2: Arc<Storage2>,
    batch_size: usize,
) -> Result<()> {
    let signed_transactions = build_association_transfer_transactions(
        receivers,
        initial_balance,
        gas_price,
        max_gas,
        config.clone(),
        &get_current_header(chain_reader_service.clone()).await?,
        storage1.clone(),
        storage2.clone(),
        batch_size,
    )
    .await?;

    let funding_txn_hashes: Vec<HashValue> =
        signed_transactions.iter().map(|txn| txn.id()).collect();
    let import_results = txpool.add_txns_multi_signed(
        signed_transactions
            .into_iter()
            .map(MultiSignedUserTransaction::VM2)
            .collect(),
        false,
        None,
    )?;
    let imported_count = import_results.iter().filter(|r| r.is_ok()).count();
    if imported_count != import_results.len() {
        let sample_errors: Vec<String> = import_results
            .iter()
            .filter_map(|r| r.as_ref().err())
            .take(5)
            .map(|e| format!("{:?}", e))
            .collect();
        bail!(
            "Funding tx import incomplete: imported {}/{}. sample errors: {}",
            imported_count,
            import_results.len(),
            sample_errors.join(", ")
        );
    }

    // Wait for all funding transactions to be processed and balances visible on-chain.
    let start = Instant::now();
    let timeout = Duration::from_secs(180);
    loop {
        let current_header = get_current_header(chain_reader_service.clone()).await?;
        let mut funded_count = 0usize;
        for account in receivers {
            let balance = get_balance(
                account.address,
                storage1.clone(),
                storage2.clone(),
                current_header.id(),
            )
            .await?;
            if balance >= initial_balance {
                funded_count += 1;
            }
        }
        if funded_count == receivers.len() {
            break;
        }

        let mut seen_count = 0usize;
        let mut executed_count = 0usize;
        let mut pending_count = 0usize;
        let mut failed_statuses = vec![];
        for txn_hash in &funding_txn_hashes {
            let status = get_txn_status_debug(chain_reader_service.clone(), *txn_hash).await?;
            if let Some(status) = status {
                seen_count += 1;
                if status.eq_ignore_ascii_case("executed") {
                    executed_count += 1;
                } else if failed_statuses.len() < 5 {
                    failed_statuses.push(format!("{}={}", txn_hash, status));
                }
            } else {
                pending_count += 1;
            }
        }

        if !failed_statuses.is_empty() {
            bail!(
                "Funding transactions executed but failed (showing up to 5): {}",
                failed_statuses.join(", ")
            );
        }

        if start.elapsed() >= timeout {
            bail!(
                "Timed out waiting funding transfer (funded {}/{}). Txn progress: total={}, seen={}, executed={}, pending={}",
                funded_count,
                receivers.len(),
                funding_txn_hashes.len(),
                seen_count,
                executed_count,
                pending_count
            );
        }

        info!(
            "waiting funding transfer: funded {}/{}, txns total={}, seen={}, executed={}, pending={}",
            funded_count,
            receivers.len(),
            funding_txn_hashes.len(),
            seen_count,
            executed_count,
            pending_count
        );
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
    }

    info!(
        "Phase 2 completed: transferred tokens to {} accounts",
        receivers.len()
    );
    Ok(())
}

fn build_transfer_transactions_sync(
    senders: &[AccountInfo],
    receivers: &[AccountInfo],
    amount: u128,
    gas_price: u64,
    max_gas: u64,
    seq_numbers: &mut HashMap<AccountAddress, u64>,
    expire_time: u64,
    chain_id: ChainId2,
    simple_transfer: bool,
) -> Result<Vec<RawUserTransaction2>> {
    if senders.len() != receivers.len() {
        bail!("senders.len() != receivers.len()");
    }

    let mut transactions = vec![];
    for index in 0..senders.len() {
        let sender = &senders[index];
        let receiver = &receivers[index];
        let next_seq = *seq_numbers.get(&sender.address).unwrap_or(&0);
        seq_numbers
            .entry(sender.address)
            .and_modify(|seq| *seq += 1)
            .or_insert(next_seq + 1);

        let transaction = if simple_transfer {
            // Use simple peer-to-peer transfer (~700K gas per txn)
            raw_peer_to_peer_txn(
                sender.address,
                receiver.address,
                amount,
                next_seq,
                gas_price,
                max_gas,
                G_STC_TOKEN_CODE.clone().try_into()?,
                expire_time,
                chain_id,
            )
        } else {
            // Use batch transfer (~13.8M gas per txn)
            build_batch_transfer_txn2(
                sender.address,
                vec![receiver.address],
                next_seq,
                amount,
                gas_price,
                max_gas,
                expire_time,
                chain_id,
            )
        };
        transactions.push(transaction);
    }
    Ok(transactions)
}

struct BenchmarkState {
    accounts: Vec<AccountInfo>,
    gas_price: u64,
    max_gas: u64,
    batch_user_count: usize,
    /// Total number of user transactions to be sent (calculated from accounts and batch_user_count)
    total_txn_count: usize,
    executed_count: AtomicUsize,
    batch_index: AtomicUsize,
    is_completed: AtomicBool,
    chain_id: ChainId2,
    simple_transfer: bool,
    /// Hashes of benchmark transactions (to distinguish from funding transactions)
    txn_hashes: Mutex<HashSet<HashValue>>,
}

impl BenchmarkState {
    fn new(
        accounts: Vec<AccountInfo>,
        gas_price: u64,
        max_gas: u64,
        batch_user_count: usize,
        chain_id: ChainId2,
        simple_transfer: bool,
    ) -> Self {
        // Normalize to a usable batch size:
        // 1) no larger than account count
        // 2) even number so sender/receiver split has equal length
        let mut effective_batch_user_count = batch_user_count.min(accounts.len());
        if effective_batch_user_count % 2 != 0 {
            effective_batch_user_count -= 1;
        }
        if effective_batch_user_count != batch_user_count {
            info!(
                "Adjusted batch_user_count from {} to {} (account_count={})",
                batch_user_count,
                effective_batch_user_count,
                accounts.len()
            );
        }
        if effective_batch_user_count == 0 {
            warn!(
                "Effective batch_user_count is 0 (requested={}, account_count={}), benchmark will submit no user transactions",
                batch_user_count,
                accounts.len()
            );
        }

        // Each batch has batch_user_count/2 senders, each sends 1 transaction
        let total_batches = if effective_batch_user_count > 0 {
            accounts.len() / effective_batch_user_count
        } else {
            0
        };
        let txns_per_batch = effective_batch_user_count / 2;
        let total_txn_count = total_batches * txns_per_batch;

        info!(
            "BenchmarkState: accounts={}, effective_batch_user_count={}, total_batches={}, txns_per_batch={}, total_txn_count={}",
            accounts.len(), effective_batch_user_count, total_batches, txns_per_batch, total_txn_count
        );

        Self {
            accounts,
            gas_price,
            max_gas,
            batch_user_count: effective_batch_user_count,
            total_txn_count,
            executed_count: AtomicUsize::new(0),
            batch_index: AtomicUsize::new(0),
            is_completed: AtomicBool::new(false),
            chain_id,
            simple_transfer,
            txn_hashes: Mutex::new(HashSet::new()),
        }
    }

    /// Add transaction hashes to the benchmark set
    fn add_txn_hashes(&self, hashes: &[HashValue]) {
        let mut set = self.txn_hashes.lock().unwrap();
        set.extend(hashes.iter().copied());
    }

    /// Check if a transaction hash belongs to the benchmark
    fn is_benchmark_txn(&self, hash: &HashValue) -> bool {
        let set = self.txn_hashes.lock().unwrap();
        set.contains(hash)
    }

    /// Build next batch of transactions using different users each time.
    /// Each batch uses batch_user_count users, where first half sends to second half.
    /// Returns None if all batches have been sent.
    fn build_next_batch(&self, expire_time: u64) -> Option<Vec<RawUserTransaction2>> {
        if self.batch_user_count == 0 {
            return None;
        }
        let batch_index = self.batch_index.fetch_add(1, Ordering::SeqCst);
        let start = batch_index * self.batch_user_count;

        // Check if we have enough accounts for this batch
        if start + self.batch_user_count > self.accounts.len() {
            return None;
        }

        let batch_accounts = &self.accounts[start..start + self.batch_user_count];
        let mid = self.batch_user_count / 2;
        let senders = &batch_accounts[..mid];
        let receivers = &batch_accounts[mid..];

        // Each sender's sequence number is 0 since they haven't sent before
        let mut seq_numbers: HashMap<AccountAddress, u64> = HashMap::new();
        for sender in senders {
            seq_numbers.insert(sender.address, 0);
        }

        match build_transfer_transactions_sync(
            senders,
            receivers,
            1,
            self.gas_price,
            self.max_gas,
            &mut seq_numbers,
            expire_time,
            self.chain_id,
            self.simple_transfer,
        ) {
            Ok(txns) => Some(txns),
            Err(e) => {
                error!("Failed to build batch {}: {:?}", batch_index, e);
                None
            }
        }
    }

    /// Check if all batches have been sent
    fn all_batches_sent(&self) -> bool {
        if self.batch_user_count == 0 {
            return true;
        }
        let next_start = self.batch_index.load(Ordering::SeqCst) * self.batch_user_count;
        next_start + self.batch_user_count > self.accounts.len()
    }

    /// Get total number of batches
    fn total_batches(&self) -> usize {
        if self.batch_user_count == 0 {
            0
        } else {
            self.accounts.len() / self.batch_user_count
        }
    }

    /// Check if all transactions have been executed
    fn all_txns_executed(&self) -> bool {
        self.executed_count.load(Ordering::SeqCst) >= self.total_txn_count
    }

    fn add_executed_count(&self, count: usize) -> usize {
        self.executed_count.fetch_add(count, Ordering::SeqCst) + count
    }

    fn mark_completed(&self) {
        self.is_completed.store(true, Ordering::SeqCst);
    }

    fn is_completed(&self) -> bool {
        self.is_completed.load(Ordering::SeqCst)
    }
}

async fn execute_benchmark(
    node: &NodeHandle,
    account_count: u32,
    initial_balance: u128,
    initial_gas_fee: u128,
    gas_price: u64,
    max_gas: u64,
    batch_user_count: usize,
    balance_wait_timeout_secs: u64,
    settle_delay_ms: u64,
    _preload_batches: usize,
    simple_transfer: bool,
) -> Result<()> {
    let registry = node.registry();
    let storage1 = node.storage();

    let fut = async move {
        let storage2 = registry.get_shared::<Arc<Storage2>>().await?;
        let log_handler = registry.get_shared::<Arc<LoggerHandle>>().await?;
        log_handler.update_level(LevelFilter::Info);

        let account_service = registry.service_ref::<AccountService2>().await?;

        let chain_reader_service = registry.service_ref::<ChainReaderService>().await?;

        wait_for_sufficient_balance(
            account_count,
            initial_balance,
            initial_gas_fee,
            gas_price,
            max_gas,
            chain_reader_service.clone(),
            storage1.clone(),
            storage2.clone(),
            Duration::from_secs(balance_wait_timeout_secs),
        )
        .await?;

        let txpool = registry
            .get_shared::<starcoin_txpool::TxPoolService>()
            .await?;

        let config = registry.get_shared::<Arc<NodeConfig>>().await?;

        let receivers = create_account(account_count, account_service.clone()).await?;

        // Funding batch size: 10 receivers per transaction (~13.8M gas)
        let batch_size = 10usize;
        let estimated_funding_txns = receivers.len().div_ceil(batch_size);
        info!(
            "Funding batch plan: receivers={}, batch_size={}, estimated_funding_txns={}, funding_max_gas={}, txpool_max_per_sender={}",
            receivers.len(),
            batch_size,
            estimated_funding_txns,
            max_gas,
            config.tx_pool.max_per_sender()
        );
        transfer_to_accounts(
            &receivers,
            initial_balance,
            gas_price,
            max_gas,
            config.clone(),
            chain_reader_service.clone(),
            &txpool,
            storage1.clone(),
            storage2.clone(),
            batch_size,
        )
        .await?;

        // wait for node/txpool state to settle before observing benchmark traffic
        tokio::time::sleep(tokio::time::Duration::from_millis(settle_delay_ms)).await;

        let current_header = get_current_header(chain_reader_service.clone()).await?;
        let chain_id = ChainId2::new(current_header.chain_id().id());

        let benchmark_state = Arc::new(BenchmarkState::new(
            receivers.clone(),
            gas_price,
            max_gas,
            batch_user_count,
            chain_id,
            simple_transfer,
        ));

        info!(
            "Benchmark configured: {} accounts, {} users per batch, {} total batches, {} total txns, simple_transfer={}",
            receivers.len(),
            batch_user_count,
            benchmark_state.total_batches(),
            benchmark_state.total_txn_count,
            simple_transfer
        );

        registry.put_shared(benchmark_state.clone()).await?;

        let observer = registry.register::<ObserverService>().await?;

        let receiver = txpool.subscribe_txns();
        observer.add_event_stream(receiver)?;

        info!(
            "Starting benchmark with {} user transactions",
            benchmark_state.total_txn_count
        );

        // Build ALL transactions first, then sign and import them all at once
        let total_batches = benchmark_state.total_batches();
        info!(
            "Building all {} batches of transactions before signing",
            total_batches
        );

        let expire_time = config.net().time_service().now_secs() + 3600;
        let mut all_transactions: Vec<RawUserTransaction2> = Vec::new();
        
        for _ in 0..total_batches {
            if let Some(batch) = benchmark_state.build_next_batch(expire_time) {
                all_transactions.extend(batch);
            }
        }
        
        info!(
            "Built {} total transactions, now signing all in parallel...",
            all_transactions.len()
        );
        
        // Sign and import ALL transactions at once
        let txn_hashes =
            sign_and_import_transactions(&all_transactions, &account_service, &txpool).await?;
        benchmark_state.add_txn_hashes(&txn_hashes);
        
        info!(
            "Preloading complete: {} transactions in txpool",
            txn_hashes.len()
        );

        // Wait for benchmark to complete:
        // 1. All batches have been sent, AND
        // 2. All transactions have been executed, OR
        // 3. Txpool is empty for more than 10 seconds after all batches sent
        let mut txpool_empty_since: Option<std::time::Instant> = None;
        const TXPOOL_EMPTY_TIMEOUT_SECS: u64 = 10;

        loop {
            if benchmark_state.is_completed() {
                break;
            }

            // Check if all batches sent and all transactions executed
            if benchmark_state.all_batches_sent() && benchmark_state.all_txns_executed() {
                info!(
                    "All batches sent and all {}/{} transactions executed, completing benchmark",
                    benchmark_state.executed_count.load(Ordering::SeqCst),
                    benchmark_state.total_txn_count
                );
                benchmark_state.mark_completed();
                break;
            }

            // Check txpool empty timeout after all batches are sent
            if benchmark_state.all_batches_sent() {
                let pending_count = txpool.status().txn_count;
                if pending_count == 0 {
                    match txpool_empty_since {
                        None => {
                            txpool_empty_since = Some(std::time::Instant::now());
                            info!(
                                "Txpool is empty, starting {} second timeout (executed: {}/{})",
                                TXPOOL_EMPTY_TIMEOUT_SECS,
                                benchmark_state.executed_count.load(Ordering::SeqCst),
                                benchmark_state.total_txn_count
                            );
                        }
                        Some(since) => {
                            if since.elapsed().as_secs() >= TXPOOL_EMPTY_TIMEOUT_SECS {
                                info!(
                                    "Txpool empty for {} seconds, completing benchmark (executed: {}/{})",
                                    TXPOOL_EMPTY_TIMEOUT_SECS,
                                    benchmark_state.executed_count.load(Ordering::SeqCst),
                                    benchmark_state.total_txn_count
                                );
                                benchmark_state.mark_completed();
                                break;
                            }
                        }
                    }
                } else {
                    // Txpool has transactions, reset the timer
                    if txpool_empty_since.is_some() {
                        info!(
                            "Txpool has {} pending transactions, resetting timeout",
                            pending_count
                        );
                    }
                    txpool_empty_since = None;
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        info!(
            "Benchmark completed: {}/{} user transactions executed",
            benchmark_state.executed_count.load(Ordering::SeqCst),
            benchmark_state.total_txn_count
        );

        Ok::<(), anyhow::Error>(())
    };

    fut.await
}

async fn sign_and_import_transactions(
    transactions: &[RawUserTransaction2],
    account_service: &ServiceRef<AccountService2>,
    txpool: &starcoin_txpool::TxPoolService,
) -> Result<Vec<HashValue>> {
    use futures::future::join_all;

    // First, unlock all unique sender accounts in parallel
    let unique_senders: std::collections::HashSet<_> =
        transactions.iter().map(|t| t.sender()).collect();

    let unlock_futures: Vec<_> = unique_senders
        .iter()
        .map(|sender| {
            let account_service = account_service.clone();
            let sender = *sender;
            async move {
                account_service
                    .send(AccountRequest::UnlockAccount(
                        sender,
                        "".to_string(),
                        std::time::Duration::from_secs(3600),
                    ))
                    .await
            }
        })
        .collect();

    let _unlock_results = join_all(unlock_futures).await;

    // Now sign all transactions in parallel
    let sign_futures: Vec<_> = transactions
        .iter()
        .map(|txn| {
            let account_service = account_service.clone();
            let txn = txn.clone();
            async move {
                let sender_address = txn.sender();
                match account_service
                    .send(AccountRequest::SignTxn {
                        txn: Box::new(txn),
                        signer: sender_address,
                    })
                    .await??
                {
                    AccountResponse::SignedTxn(signed_transaction) => {
                        Ok::<_, anyhow::Error>(*signed_transaction)
                    }
                    _ => bail!("Unexpected response type."),
                }
            }
        })
        .collect();

    let sign_results = join_all(sign_futures).await;

    let mut signed_transactions = vec![];
    let mut txn_hashes = vec![];

    for result in sign_results {
        let signed_transaction = result?;
        txn_hashes.push(signed_transaction.id());
        signed_transactions.push(signed_transaction);
    }

    txpool.add_txns_multi_signed(
        signed_transactions
            .into_iter()
            .map(MultiSignedUserTransaction::VM2)
            .collect(),
        false,
        None,
    )?;

    Ok(txn_hashes)
}

fn sign_and_import_transactions_sync(
    transactions: &[RawUserTransaction2],
    account_service: &ServiceRef<AccountService2>,
    txpool: &starcoin_txpool::TxPoolService,
) -> Result<Vec<HashValue>> {
    use futures::future::join_all;

    let transactions = transactions.to_vec();
    let account_service = account_service.clone();
    let txpool = txpool.clone();

    // Spawn a new thread to avoid deadlock since we're already in a block_on context
    let handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(num_cpus::get())
            .enable_all()
            .build()
            .expect("Failed to create runtime");

        rt.block_on(async {
            // First, unlock all unique sender accounts in parallel
            let unique_senders: std::collections::HashSet<_> =
                transactions.iter().map(|t| t.sender()).collect();

            let unlock_futures: Vec<_> = unique_senders
                .iter()
                .map(|sender| {
                    let account_service = account_service.clone();
                    let sender = *sender;
                    async move {
                        account_service
                            .send(AccountRequest::UnlockAccount(
                                sender,
                                "".to_string(),
                                std::time::Duration::from_secs(3600),
                            ))
                            .await
                    }
                })
                .collect();

            let _unlock_results = join_all(unlock_futures).await;

            // Now sign all transactions in parallel
            let sign_futures: Vec<_> = transactions
                .iter()
                .map(|txn| {
                    let account_service = account_service.clone();
                    let txn = txn.clone();
                    async move {
                        let sender_address = txn.sender();
                        match account_service
                            .send(AccountRequest::SignTxn {
                                txn: Box::new(txn),
                                signer: sender_address,
                            })
                            .await??
                        {
                            AccountResponse::SignedTxn(signed_transaction) => {
                                Ok::<_, anyhow::Error>(*signed_transaction)
                            }
                            _ => bail!("Unexpected response type."),
                        }
                    }
                })
                .collect();

            let sign_results = join_all(sign_futures).await;

            let mut signed_transactions = vec![];
            let mut txn_hashes = vec![];

            for result in sign_results {
                let signed_transaction = result?;
                txn_hashes.push(signed_transaction.id());
                signed_transactions.push(signed_transaction);
            }

            txpool.add_txns_multi_signed(
                signed_transactions
                    .into_iter()
                    .map(MultiSignedUserTransaction::VM2)
                    .collect(),
                false,
                None,
            )?;

            Ok::<Vec<HashValue>, anyhow::Error>(txn_hashes)
        })
    });

    handle.join().map_err(|_| format_err!("Thread panicked"))?
}

struct ObserverService {
    transaction_data: HashMap<HashValue, Vec<TransactionExecutionResult>>,
    storage1: Arc<Storage>,
    benchmark_state: Option<Arc<BenchmarkState>>,
    account_service: Option<ServiceRef<AccountService2>>,
    txpool: Option<starcoin_txpool::TxPoolService>,
    config: Option<Arc<NodeConfig>>,
}

impl ObserverService {
    fn new(storage1: Arc<Storage>) -> Result<Self> {
        Ok(Self {
            transaction_data: HashMap::new(),
            storage1,
            benchmark_state: None,
            account_service: None,
            txpool: None,
            config: None,
        })
    }

    fn try_submit_next_batch(&self) -> Result<()> {
        let state = match &self.benchmark_state {
            Some(s) => s,
            None => return Ok(()),
        };
        let account_service = match &self.account_service {
            Some(s) => s,
            None => return Ok(()),
        };
        let txpool = match &self.txpool {
            Some(t) => t,
            None => return Ok(()),
        };
        let config = match &self.config {
            Some(c) => c,
            None => return Ok(()),
        };

        if state.is_completed() || state.all_batches_sent() {
            return Ok(());
        }

        let expire_time = config.net().time_service().now_secs() + 3600;

        // Build next batch - each batch uses different users with seq=0
        let batch = match state.build_next_batch(expire_time) {
            Some(b) => b,
            None => {
                info!("All batches have been sent");
                return Ok(());
            }
        };

        let batch_index = state.batch_index.load(Ordering::SeqCst);
        let txn_hashes = sign_and_import_transactions_sync(&batch, account_service, txpool)?;
        state.add_txn_hashes(&txn_hashes);
        info!(
            "Submitted batch {}/{}: {} transactions",
            batch_index,
            state.total_batches(),
            batch.len()
        );

        Ok(())
    }

    fn update_transaction_status(
        &mut self,
        new_header: HashValue,
        connected_time_ms: u64,
    ) -> Result<usize> {
        let block = self
            .storage1
            .get_block_by_hash(new_header)?
            .ok_or_else(|| format_err!("block not found: {:?}", new_header))?;
        let block_number = block.header().number();
        let block_id = block.header().id();
        let block_timestamp_ms = block.header().timestamp();

        // Count benchmark transactions only (not funding transactions)
        let mut benchmark_txn_count = 0usize;
        
        for transaction in &block.body.transactions2 {
            let txn_hash = transaction.id();
            
            // Only record benchmark transactions for TPS calculation
            if let Some(ref state) = self.benchmark_state {
                if state.is_benchmark_txn(&txn_hash) {
                    self.transaction_data
                        .entry(txn_hash)
                        .or_default()
                        .push(TransactionExecutionResult::Executed(
                            connected_time_ms,
                            block_number,
                            block_id,
                            block_timestamp_ms,
                        ));
                    benchmark_txn_count += 1;
                }
            }
        }

        if let Some(ref state) = self.benchmark_state {
            if benchmark_txn_count > 0 {
                let total = state.add_executed_count(benchmark_txn_count);
                info!(
                    "Block {} executed {} benchmark txns, total: {}/{}",
                    block_number, benchmark_txn_count, total, state.total_txn_count
                );
                // Mark completed when all transactions are executed
                if state.all_batches_sent() && total >= state.total_txn_count {
                    state.mark_completed();
                }
                return Ok(benchmark_txn_count);
            }
        }

        Ok(0)
    }

    fn record_mined_event(&mut self, block: &Block) -> Result<()> {
        let mined_time_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;
        let block_number = block.header().number();
        let block_id = block.header().id();

        // Record mined event only for benchmark transactions
        let mut benchmark_txn_count = 0usize;
        
        for transaction in &block.body.transactions2 {
            let txn_hash = transaction.id();
            
            // Only record benchmark transactions
            if let Some(ref state) = self.benchmark_state {
                if state.is_benchmark_txn(&txn_hash) {
                    self.transaction_data
                        .entry(txn_hash)
                        .or_default()
                        .push(TransactionExecutionResult::Mined(
                            mined_time_ms,
                            block_number,
                            block_id,
                        ));
                    benchmark_txn_count += 1;
                }
            }
        }

        if benchmark_txn_count > 0 {
            info!(
                "Block {} mined with {} benchmark txns at {}",
                block_number, benchmark_txn_count, mined_time_ms
            );
        }

        Ok(())
    }

    fn dump_results(&self) -> Result<()> {
        let dumper = ResultsDumper::new(&self.transaction_data);

        // Calculate and log statistics
        let stats = dumper.calculate_stats();
        info!("\n{}", stats);

        // Print top 10 blocks with highest latency
        let top_latency_blocks = dumper.get_top_latency_blocks(10);
        if !top_latency_blocks.is_empty() {
            info!(
                "Top {} blocks with highest latency (deduplicated by block_id):",
                top_latency_blocks.len()
            );
            for (i, (block_id, block_number, latency_ms)) in top_latency_blocks.iter().enumerate() {
                info!(
                    "  #{}: block_number={}, latency={:.2}ms, block_id={}",
                    i + 1,
                    block_number,
                    latency_ms,
                    block_id
                );
            }
        }

        // Export to JSON for AI agent loop with pipeline timing data from global collector
        dumper.export_json("./benchmark_results.json")?;
        
        dumper.dump_results()
    }
}

impl ServiceFactory<Self> for ObserverService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let storage1 = ctx.get_shared::<Arc<Storage>>()?;
        let mut service = Self::new(storage1)?;
        service.benchmark_state = ctx.get_shared::<Arc<BenchmarkState>>().ok();
        service.account_service = ctx.service_ref_opt::<AccountService2>()?.cloned();
        service.txpool = ctx.get_shared::<starcoin_txpool::TxPoolService>().ok();
        service.config = ctx.get_shared::<Arc<NodeConfig>>().ok();
        Ok(service)
    }
}

impl ActorService for ObserverService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        // ctx.subscribe::<NewDagBlock>();
        // ctx.subscribe::<NewDagBlockFromPeer>();
        ctx.subscribe::<MinedBlock>();
        ctx.subscribe::<NewHeadBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        // ctx.unsubscribe::<NewDagBlock>();
        // ctx.unsubscribe::<NewDagBlockFromPeer>();
        ctx.unsubscribe::<MinedBlock>();
        ctx.unsubscribe::<NewHeadBlock>();

        if let Err(e) = self.dump_results() {
            error!("failed to dump the results: {:?}", e);
        }
        Ok(())
    }
}

impl EventHandler<Self, NewHeadBlock> for ObserverService {
    fn handle_event(&mut self, msg: NewHeadBlock, _ctx: &mut ServiceContext<Self>) {
        let connected_time_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;
        if let Err(e) =
            self.update_transaction_status(msg.executed_block.block().id(), connected_time_ms)
        {
            error!("failed to update transactions status: {:?}", e);
        }

        // Check if completed
        if let Some(ref state) = self.benchmark_state {
            if state.is_completed() || state.all_batches_sent() {
                return;
            }
        }
        // Submit next batch of transactions after processing the block
        if let Err(e) = self.try_submit_next_batch() {
            error!("failed to submit next batch: {:?}", e);
        }
    }
}

impl EventHandler<Self, MinedBlock> for ObserverService {
    fn handle_event(&mut self, msg: MinedBlock, _ctx: &mut ServiceContext<Self>) {
        if let Err(e) = self.record_mined_event(msg.0.as_ref()) {
            error!("failed to record mined event: {:?}", e);
        }
    }
}

// impl EventHandler<Self, NewDagBlock> for ObserverService {
//     fn handle_event(&mut self, msg: NewDagBlock, _ctx: &mut ServiceContext<Self>) {
//         if let Err(e) = self.update_transaction_status(
//             msg.executed_block.block().id(),
//             msg.connected_time_ms,
//         ) {
//             error!("failed to update transactions status: {:?}", e);
//         }
//         // Check if completed
//         if let Some(ref state) = self.benchmark_state {
//             if state.is_completed() || state.all_batches_sent() {
//                 return;
//             }
//         }
//         // Submit next batch of transactions after processing the block
//         if let Err(e) = self.try_submit_next_batch() {
//             error!("failed to submit next batch: {:?}", e);
//         }
//     }
// }

// impl EventHandler<Self, NewDagBlockFromPeer> for ObserverService {
//     fn handle_event(&mut self, msg: NewDagBlockFromPeer, _ctx: &mut ServiceContext<Self>) {
//         if let Err(e) = self.update_transaction_status(msg.executed_block.id()) {
//             error!("failed to update transactions status: {:?}", e);
//         }
//     }
// }

impl EventHandler<Self, Arc<[(HashValue, TxStatus)]>> for ObserverService {
    fn handle_event(&mut self, msg: Arc<[(HashValue, TxStatus)]>, _ctx: &mut ServiceContext<Self>) {
        let now_str = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        for transaction_event in msg.as_ref() {
            match &transaction_event.1 {
                starcoin_types::transaction::TxStatus::Added => self
                    .transaction_data
                    .entry(transaction_event.0)
                    .or_default()
                    .push(TransactionExecutionResult::Added(now_ms)),
                starcoin_types::transaction::TxStatus::Rejected => self
                    .transaction_data
                    .entry(transaction_event.0)
                    .or_default()
                    .push(TransactionExecutionResult::Rejected(now_str.clone())),
                starcoin_types::transaction::TxStatus::Culled => self
                    .transaction_data
                    .entry(transaction_event.0)
                    .or_default()
                    .push(TransactionExecutionResult::Culled(now_str.clone())),
                _ => self
                    .transaction_data
                    .entry(transaction_event.0)
                    .or_default()
                    .push(TransactionExecutionResult::Other(format!(
                        "{}({})",
                        transaction_event.1,
                        now_str.clone()
                    ))),
            }
        }
    }
}
