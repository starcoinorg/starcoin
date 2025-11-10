use std::{
    collections::HashMap, error::Error, fs::OpenOptions, io::Write, path::{Path, PathBuf}, str::FromStr, sync::Arc
};

use anyhow::{bail, format_err, Result};
use chrono::{Local, NaiveDateTime};
use clap::{Parser, ValueEnum, ValueHint};
use futures::channel::mpsc;
use plotters::prelude::*;
use starcoin_chain_api::message::{ChainRequest, ChainResponse};
use starcoin_chain_service::ChainReaderService;
use starcoin_config::{
    genesis_config::CustomNetworkID, BaseConfig, BuiltinNetworkID, ChainNetworkID, NodeConfig, StarcoinOpt
};
use starcoin_crypto::HashValue;
use starcoin_logger::{
    prelude::{error, info, LevelFilter},
    LoggerHandle,
};
use starcoin_service_registry::{
    ActorService, EventHandler, RegistryAsyncService, ServiceContext, ServiceFactory, ServiceRef,
};
use starcoin_storage::{BlockStore, Storage, Storage2, Store};
use starcoin_transaction_builder::vm2::build_batch_transfer_txn as build_batch_transfer_txn2;
use starcoin_txpool::TxStatus;
use starcoin_types::{
    block::BlockHeader,
    genesis_config::ChainId,
    multi_transaction::MultiSignedUserTransaction,
    system_events::{NewDagBlock, NewDagBlockFromPeer},
};
use starcoin_vm2_account_api::{
    message::{AccountRequest, AccountResponse},
    AccountInfo,
};
use starcoin_vm2_account_service::AccountService as AccountService2;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::{account_config::G_STC_TOKEN_CODE, transaction::SignedUserTransaction};
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

impl NetworkChoice {
    fn to_chain_network(self) -> ChainNetworkID {
        match self {
            Self::Custom => ChainNetworkID::Custom(CustomNetworkID::new(
                "my_chain".to_owned(),
                ChainId::new(121),
            )),
            Self::Builtin(builtin_network_id) => {
                match builtin_network_id {
                    BuiltinNetworkID::Test => ChainNetworkID::Builtin(BuiltinNetworkID::Test),
                    BuiltinNetworkID::Dev => ChainNetworkID::Builtin(BuiltinNetworkID::Dev),
                    BuiltinNetworkID::Halley => ChainNetworkID::Builtin(BuiltinNetworkID::Halley),
                    BuiltinNetworkID::Proxima => ChainNetworkID::Builtin(BuiltinNetworkID::Proxima),
                    BuiltinNetworkID::Barnard => ChainNetworkID::Builtin(BuiltinNetworkID::Barnard),
                    BuiltinNetworkID::Main => ChainNetworkID::Builtin(BuiltinNetworkID::Main),
                }
            }
        }
    }

    fn genesis_name(self) -> &'static str {
        match self {
            Self::Custom | Self::Builtin(BuiltinNetworkID::Halley) => "halley",
            Self::Builtin(builtin_network_id) => {
                match builtin_network_id {
                    BuiltinNetworkID::Test => "test",
                    BuiltinNetworkID::Dev => "dev",
                    BuiltinNetworkID::Halley => "halley",
                    BuiltinNetworkID::Proxima => "proxima",
                    BuiltinNetworkID::Barnard => "barnard",
                    BuiltinNetworkID::Main => "main",
                }
            }
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let network_choice = cli.network;
    let chain_network = network_choice.to_chain_network();
    let data_dir = DataDir::new(cli.data_dir)?;
    let base_dir = data_dir.path().to_path_buf();

    let mut init_opt = StarcoinOpt {
        net: Some(chain_network.clone()),
        base_data_dir: Some(base_dir.clone()),
        ..Default::default()
    };
    init_opt.genesis_config = Some(network_choice.genesis_name().to_owned());
    BaseConfig::load_with_opt(&init_opt)?;

    let mut global_opt = StarcoinOpt {
        net: Some(chain_network),
        base_data_dir: Some(base_dir),
        ..Default::default()
    };
    global_opt.genesis_config = init_opt.genesis_config.clone();

    let node_config = Arc::new(NodeConfig::load_with_opt(&global_opt)?);
    let bench_result = run_benchmark(node_config).await;
    let close_result = data_dir.close();
    bench_result?;
    close_result?;
    Ok(())
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

async fn build_transaction_to_send_token_to_account(
    sender: &[AccountInfo],
    receiver: &[AccountInfo],
    amount: u128,
    account_service: ServiceRef<AccountService2>,
    config: Arc<NodeConfig>,
    header_block: &BlockHeader,
    storage1: Arc<Storage>,
    storage2: Arc<Storage2>,
) -> Result<Vec<SignedUserTransaction>> {
    if sender.len() != receiver.len() {
        bail!("sender.len() != receiver.len()");
    }
    let expire_time = config.net().time_service().now_secs() + 3600;
    let multi_state = storage1.get_vm_multi_state(header_block.id())?;
    let statedb2 = ChainStateDB::new(storage2.clone(), Some(multi_state.state_root2()));
    let mut next_seq_map = HashMap::new();
    for s in sender {
        let next_seq = statedb2
            .get_account_resource(*s.address())?
            .sequence_number();
        next_seq_map.entry(*s.address()).or_insert(next_seq);
    }

    let mut signed_transactions = vec![];
    for index in 0..receiver.len() {
        let sender_address = sender.get(index).unwrap().address;
        let next_seq = *next_seq_map.get(&sender_address).unwrap();
        next_seq_map
            .entry(sender_address)
            .and_modify(|next_seq| *next_seq += 1);
        let transaction = build_batch_transfer_txn2(
            sender.get(index).unwrap().address,
            vec![receiver.get(index).unwrap().address],
            next_seq,
            amount,
            1,
            40_000_000,
            expire_time,
            header_block.chain_id().id().into(),
        );
        match account_service
            .send(AccountRequest::UnlockAccount(
                sender_address,
                "".to_string(),
                std::time::Duration::from_secs(100),
            ))
            .await??
        {
            AccountResponse::AccountInfo(_) => (),
            _ => bail!("Unexpected response type."),
        }
        let signed_transaction = match account_service
            .send(AccountRequest::SignTxn {
                txn: Box::new(transaction),
                signer: sender_address,
            })
            .await??
        {
            AccountResponse::SignedTxn(signed_transaction) => *signed_transaction,
            _ => bail!("Unexpected response type."),
        };
        signed_transactions.push(signed_transaction);
    }

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

async fn run_benchmark(node_config: Arc<NodeConfig>) -> Result<()> {
    let node = run_node_with_all_service(node_config.clone())?;
    let registry = node.registry();
    let storage1 = node.storage();
    let storage2 = node.storage2();

    let account_count: u32 = 20;
    let initial_balance: u128 = 10_000_000_000;
    let initial_gas_fee: u128 = 4_000_000_000;

    let fut = async move {
        let log_handler = registry.get_shared::<Arc<LoggerHandle>>().await?;
        log_handler.update_level(LevelFilter::Info);

        let observer = registry.register::<ObserverService>().await?;

        let account_service = registry.service_ref::<AccountService2>().await?;
        let default_account = default_account(account_service.clone()).await?;

        let chain_reader_service = registry.service_ref::<ChainReaderService>().await?;
        loop {
            let current_header = get_current_header(chain_reader_service.clone()).await?;
            let default_account_balance = match get_balance(
                default_account.address,
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
            if default_account_balance > account_count as u128 * initial_balance + initial_gas_fee {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        }

        let txpool = registry
            .get_shared::<starcoin_txpool::TxPoolService>()
            .await?;

        let (sender, receiver) = mpsc::unbounded::<Arc<[(HashValue, TxStatus)]>>();

        txpool.inner.queue().add_full_listener(sender);
        observer.add_event_stream(receiver)?;

        let config = registry.get_shared::<Arc<NodeConfig>>().await?;

        let receivers = create_account(account_count, account_service.clone()).await?;

        let signed_transactions = build_transaction_to_send_token_to_account(
            &vec![default_account; receivers.len()],
            &receivers,
            initial_balance,
            account_service.clone(),
            config.clone(),
            &get_current_header(chain_reader_service.clone()).await?,
            storage1.clone(),
            storage2.clone(),
        )
        .await?;

        txpool.inner.import_txns(
            signed_transactions
                .into_iter()
                .map(MultiSignedUserTransaction::VM2)
                .collect(),
            false,
            None,
        )?;

        loop {
            let transactions = txpool
                .inner
                .get_pending(100, config.net().time_service().now_secs())?;
            if transactions.is_empty() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        }

        StarcoinVM::set_concurrency_level_once(num_cpus::get());

        let mid = receivers.len() / 2;
        let signed_transactions = build_transaction_to_send_token_to_account(
            &receivers[..mid],
            &receivers[mid..],
            1,
            account_service.clone(),
            config.clone(),
            &get_current_header(chain_reader_service.clone()).await?,
            storage1.clone(),
            storage2.clone(),
        )
        .await?;

        txpool.inner.import_txns(
            signed_transactions
                .into_iter()
                .map(MultiSignedUserTransaction::VM2)
                .collect(),
            false,
            None,
        )?;

        loop {
            let transactions = txpool
                .inner
                .get_pending(100, config.net().time_service().now_secs())?;
            if transactions.is_empty() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
        }

        Ok::<(), anyhow::Error>(())
    };

    fut.await?;

    node.stop_service(ObserverService::service_name().to_string())?;
    node.stop()?;

    Ok(())
}

enum TransactionExecutionResult {
    Added(String),
    Rejected(String),
    Culled(String),
    Executed(String),
    #[allow(dead_code)]
    ExecutedNotInMain(String),
    Other(String),
}

impl std::fmt::Debug for TransactionExecutionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionExecutionResult::Added(op_time) => {
                write!(f, "TransactionExecutionResult::Added({})", op_time)
            }
            TransactionExecutionResult::Rejected(op_time) => {
                write!(f, "TransactionExecutionResult::Rejected({})", op_time)
            }
            TransactionExecutionResult::Culled(op_time) => {
                write!(f, "TransactionExecutionResult::Culled({})", op_time)
            }
            TransactionExecutionResult::Executed(op_time) => {
                write!(f, "TransactionExecutionResult::Executed({})", op_time)
            }
            TransactionExecutionResult::ExecutedNotInMain(op_time) => write!(
                f,
                "TransactionExecutionResult::ExecutedNotInMain({})",
                op_time
            ),
            TransactionExecutionResult::Other(op_time) => {
                write!(f, "TransactionExecutionResult::Other({})", op_time)
            }
        }
    }
}

struct ObserverService {
    transaction_data: HashMap<HashValue, Vec<TransactionExecutionResult>>,
    storage1: Arc<Storage>,
}

impl ObserverService {
    fn new(storage1: Arc<Storage>) -> Result<Self> {
        Ok(Self {
            transaction_data: HashMap::new(),
            storage1,
        })
    }

    fn update_transaction_status(&mut self, new_header: HashValue) -> Result<()> {
        let block = self
            .storage1
            .get_block_by_hash(new_header)?
            .ok_or_else(|| format_err!("block not found: {:?}", new_header))?;
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        for transaction in block.body.transactions2 {
            self.transaction_data
                .entry(transaction.id())
                .or_default()
                .push(TransactionExecutionResult::Executed(now.clone()));
        }
        Ok(())
    }

    fn dump_results(&mut self) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("./transaction_results.txt")?;
        for (transaction, results) in &self.transaction_data {
            writeln!(
                file,
                "transaction id: {}, results: {:?}",
                *transaction, results
            )?;
        }
        match self.export_latency_timeline_svg("./latency_timeline.svg") {
            Ok(_) => (),
            Err(e) => {
                error!("failed to export latency timeline svg: {}", e);
                return Err(format_err!("failed to export latency timeline svg: {}", e));
            }
        }
        Ok(())
    }

    fn collect_executions(&self) -> Vec<(Option<f64>, f64)> {
        let fmt = "%Y-%m-%d %H:%M:%S%.3f";
        let mut result = Vec::new();

        for events in self.transaction_data.values() {
            let mut added_times = Vec::new();
            let mut executed_times = Vec::new();

            for ev in events {
                match ev {
                    TransactionExecutionResult::Added(ts) => {
                        if let Ok(t) = NaiveDateTime::parse_from_str(ts, fmt) {
                            added_times.push(t);
                        }
                    }
                    TransactionExecutionResult::Executed(ts)
                    | TransactionExecutionResult::ExecutedNotInMain(ts) => {
                        if let Ok(t) = NaiveDateTime::parse_from_str(ts, fmt) {
                            executed_times.push(t);
                        }
                    }
                    _ => {}
                }
            }

            if added_times.is_empty() {
                continue;
            }

            if executed_times.is_empty() {
                result.push((None, f64::INFINITY));
                continue;
            }

            let first_add = added_times[0];
            for exec in executed_times {
                let delay = exec - first_add;
                if let Some(us) = delay.num_microseconds() {
                    let ms = us as f64 / 1000.0;
                    if ms >= 0.0 {
                        result.push((Some(exec.and_utc().timestamp_millis() as f64), ms));
                    }
                }
            }
        }

        result.sort_by(|a, b| match (a.0, b.0) {
            (Some(t1), Some(t2)) => t1.partial_cmp(&t2).unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });

        result
    }

    pub fn export_latency_timeline_svg(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let data = self.collect_executions();
        if data.is_empty() {
            return Ok(());
        }

        let min_ts = data
            .iter()
            .filter_map(|(t, _)| *t)
            .fold(f64::INFINITY, f64::min);
        let max_ts = data
            .iter()
            .filter_map(|(t, _)| *t)
            .fold(f64::NEG_INFINITY, f64::max);

        let max_latency = data
            .iter()
            .filter(|(_, d)| d.is_finite())
            .map(|(_, d)| *d)
            .fold(0.0, f64::max)
            .max(1.0);

        let root = SVGBackend::new(file_path, (1600, 800)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Add → Execute Latency Timeline (included unexecuted transactions)",
                ("sans-serif", 28),
            )
            .margin(20)
            .x_label_area_size(80)
            .y_label_area_size(70)
            .build_cartesian_2d(min_ts..(max_ts + 2000.0), 0f64..max_latency)?;

        chart
            .configure_mesh()
            .x_desc("Execution Time (ms since start)")
            .y_desc("Latency (ms)")
            .axis_desc_style(("sans-serif", 20))
            .label_style(("sans-serif", 14))
            .draw()?;

        let bar_width = 200.0;

        chart.draw_series(data.iter().map(|(opt_t, delay)| {
            let color = if delay.is_finite() {
                RGBColor(50, 100, 220).filled()
            } else {
                RED.filled()
            };

            if let Some(ts) = opt_t {
                Rectangle::new(
                    [
                        (ts - bar_width / 2.0, 0.0),
                        (ts + bar_width / 2.0, delay.min(max_latency)),
                    ],
                    color,
                )
            } else {
                Rectangle::new(
                    [(max_ts + 1000.0, 0.0), (max_ts + 1500.0, max_latency * 0.9)],
                    color,
                )
            }
        }))?;

        root.present()?;
        Ok(())
    }
}

impl ServiceFactory<Self> for ObserverService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        Self::new(ctx.get_shared::<Arc<Storage>>()?)
    }
}

impl ActorService for ObserverService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<NewDagBlock>();
        ctx.subscribe::<NewDagBlockFromPeer>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<NewDagBlock>();
        ctx.unsubscribe::<NewDagBlockFromPeer>();

        if let Err(e) = self.dump_results() {
            error!("failed to dump the results: {:?}", e);
        }
        Ok(())
    }
}

impl EventHandler<Self, NewDagBlock> for ObserverService {
    fn handle_event(&mut self, msg: NewDagBlock, _ctx: &mut ServiceContext<Self>) {
        if let Err(e) = self.update_transaction_status(msg.executed_block.block().id()) {
            error!("failed to update transactions status: {:?}", e);
        }
    }
}

impl EventHandler<Self, NewDagBlockFromPeer> for ObserverService {
    fn handle_event(&mut self, msg: NewDagBlockFromPeer, _ctx: &mut ServiceContext<Self>) {
        if let Err(e) = self.update_transaction_status(msg.executed_block.id()) {
            error!("failed to update transactions status: {:?}", e);
        }
    }
}

impl EventHandler<Self, Arc<[(HashValue, TxStatus)]>> for ObserverService {
    fn handle_event(&mut self, msg: Arc<[(HashValue, TxStatus)]>, _ctx: &mut ServiceContext<Self>) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        for transaction_event in msg.as_ref() {
            match transaction_event.1 {
                starcoin_types::transaction::TxStatus::Added => self
                    .transaction_data
                    .entry(transaction_event.0)
                    .or_default()
                    .push(TransactionExecutionResult::Added(now.clone())),
                starcoin_types::transaction::TxStatus::Rejected => self
                    .transaction_data
                    .entry(transaction_event.0)
                    .or_default()
                    .push(TransactionExecutionResult::Rejected(now.clone())),
                starcoin_types::transaction::TxStatus::Culled => self
                    .transaction_data
                    .entry(transaction_event.0)
                    .or_default()
                    .push(TransactionExecutionResult::Culled(now.clone())),
                _ => self
                    .transaction_data
                    .entry(transaction_event.0)
                    .or_default()
                    .push(TransactionExecutionResult::Other(format!(
                        "{}({})",
                        transaction_event.1,
                        now.clone()
                    ))),
            }
        }
    }
}
