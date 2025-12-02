mod results;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use anyhow::{bail, format_err, Result};
use chrono::Local;
use clap::{Parser, ValueHint};
use futures::channel::mpsc;
use results::{ResultsDumper, TransactionExecutionResult};
use starcoin_chain_api::message::{ChainRequest, ChainResponse};
use starcoin_chain_service::ChainReaderService;
use starcoin_config::{
    genesis_config::CustomNetworkID, BaseConfig, BuiltinNetworkID, ChainNetworkID, NodeConfig,
    StarcoinOpt,
};
use starcoin_crypto::HashValue;
use starcoin_logger::{
    prelude::{error, info, LevelFilter},
    LoggerHandle,
};
use starcoin_node::NodeHandle;
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
    system_events::{NewDagBlock, NewDagBlockFromPeer, NewHeadBlock},
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

    #[arg(
        short = 'c',
        long = "account-count",
        default_value = "20",
        help = "Number of accounts to create for the benchmark."
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
        default_value = "100000",
        help = "Gas price for transactions (max: 10000)."
    )]
    gas_price: u64,

    #[arg(
        long = "max-gas",
        default_value = "40000000",
        help = "Max gas for transactions."
    )]
    max_gas: u64,
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
            Self::Builtin(builtin_network_id) => match builtin_network_id {
                BuiltinNetworkID::Test => ChainNetworkID::Builtin(BuiltinNetworkID::Test),
                BuiltinNetworkID::Dev => ChainNetworkID::Builtin(BuiltinNetworkID::Dev),
                BuiltinNetworkID::Halley => ChainNetworkID::Builtin(BuiltinNetworkID::Halley),
                BuiltinNetworkID::Proxima => ChainNetworkID::Builtin(BuiltinNetworkID::Proxima),
                BuiltinNetworkID::Barnard => ChainNetworkID::Builtin(BuiltinNetworkID::Barnard),
                BuiltinNetworkID::Main => ChainNetworkID::Builtin(BuiltinNetworkID::Main),
            },
        }
    }

    fn genesis_name(self) -> &'static str {
        match self {
            Self::Custom | Self::Builtin(BuiltinNetworkID::Halley) => "halley",
            Self::Builtin(builtin_network_id) => match builtin_network_id {
                BuiltinNetworkID::Test => "test",
                BuiltinNetworkID::Dev => "dev",
                BuiltinNetworkID::Halley => "halley",
                BuiltinNetworkID::Proxima => "proxima",
                BuiltinNetworkID::Barnard => "barnard",
                BuiltinNetworkID::Main => "main",
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
    let node = run_node_with_all_service(node_config.clone())?;

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
    ));

    node.stop()?;

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
    gas_price: u64,
    max_gas: u64,
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
            gas_price,
            max_gas,
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
    info!("jacktest: balance of {} is {}", address, balance);
    Ok(balance)
}

/// Phase 1: Wait for blocks to be mined and default account to have sufficient balance
async fn wait_for_sufficient_balance(
    default_account: &AccountInfo,
    account_count: u32,
    initial_balance: u128,
    initial_gas_fee: u128,
    gas_price: u64,
    max_gas: u64,
    chain_reader_service: ServiceRef<ChainReaderService>,
    storage1: Arc<Storage>,
    storage2: Arc<Storage2>,
) -> Result<()> {
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
        let per_tx_fee = max_gas as u128 * gas_price as u128;
        let needed_balance =
            account_count as u128 * (initial_balance + per_tx_fee) + initial_gas_fee;
        if default_account_balance >= needed_balance {
            info!(
                "Default account has sufficient balance: {} >= {}",
                default_account_balance, needed_balance
            );
            break;
        }
        info!(
            "Waiting for sufficient balance: {} < {}",
            default_account_balance, needed_balance
        );
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
    }
    Ok(())
}

/// Phase 2: Transfer tokens from default account to created accounts
async fn transfer_to_accounts(
    default_account: &AccountInfo,
    receivers: &[AccountInfo],
    initial_balance: u128,
    gas_price: u64,
    max_gas: u64,
    account_service: ServiceRef<AccountService2>,
    config: Arc<NodeConfig>,
    chain_reader_service: ServiceRef<ChainReaderService>,
    txpool: &starcoin_txpool::TxPoolService,
    storage1: Arc<Storage>,
    storage2: Arc<Storage2>,
) -> Result<()> {
    let signed_transactions = build_transaction_to_send_token_to_account(
        &vec![default_account.clone(); receivers.len()],
        receivers,
        initial_balance,
        gas_price,
        max_gas,
        account_service,
        config.clone(),
        &get_current_header(chain_reader_service).await?,
        storage1,
        storage2,
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

    // Wait for all transactions to be processed
    loop {
        let transactions = txpool
            .inner
            .get_pending(100, config.net().time_service().now_secs())?;
        if transactions.is_empty() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
    }

    info!("Phase 2 completed: transferred tokens to {} accounts", receivers.len());
    Ok(())
}

/// Phase 3: Transfer tokens between created accounts
async fn transfer_between_accounts(
    receivers: &[AccountInfo],
    gas_price: u64,
    max_gas: u64,
    account_service: ServiceRef<AccountService2>,
    config: Arc<NodeConfig>,
    chain_reader_service: ServiceRef<ChainReaderService>,
    txpool: &starcoin_txpool::TxPoolService,
    storage1: Arc<Storage>,
    storage2: Arc<Storage2>,
) -> Result<()> {
    StarcoinVM::set_concurrency_level(num_cpus::get());

    let mid = receivers.len() / 2;
    let signed_transactions = build_transaction_to_send_token_to_account(
        &receivers[..mid],
        &receivers[mid..],
        1,
        gas_price,
        max_gas,
        account_service,
        config.clone(),
        &get_current_header(chain_reader_service).await?,
        storage1,
        storage2,
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

    // Wait for all transactions to be processed
    loop {
        let transactions = txpool
            .inner
            .get_pending(100, config.net().time_service().now_secs())?;
        if transactions.is_empty() {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
    }

    info!("Phase 3 completed: transferred tokens between {} accounts", receivers.len());
    Ok(())
}

async fn execute_benchmark(
    node: &NodeHandle,
    account_count: u32,
    initial_balance: u128,
    initial_gas_fee: u128,
    gas_price: u64,
    max_gas: u64,
) -> Result<()> {
    let registry = node.registry();
    let storage1 = node.storage();
    let storage2 = node.storage2();

    let fut = async move {
        let log_handler = registry.get_shared::<Arc<LoggerHandle>>().await?;
        log_handler.update_level(LevelFilter::Info);

        let observer = registry.register::<ObserverService>().await?;

        let account_service = registry.service_ref::<AccountService2>().await?;
        let default_account = default_account(account_service.clone()).await?;

        let chain_reader_service = registry.service_ref::<ChainReaderService>().await?;

        wait_for_sufficient_balance(
            &default_account,
            account_count,
            initial_balance,
            initial_gas_fee,
            gas_price,
            max_gas,
            chain_reader_service.clone(),
            storage1.clone(),
            storage2.clone(),
        )
        .await?;

        let txpool = registry
            .get_shared::<starcoin_txpool::TxPoolService>()
            .await?;

        let (sender, receiver) = mpsc::unbounded::<Arc<[(HashValue, TxStatus)]>>();

        txpool.inner.queue().add_full_listener(sender);
        observer.add_event_stream(receiver)?;

        let config = registry.get_shared::<Arc<NodeConfig>>().await?;

        let receivers = create_account(account_count, account_service.clone()).await?;

        transfer_to_accounts(
            &default_account,
            &receivers,
            initial_balance,
            gas_price,
            max_gas,
            account_service.clone(),
            config.clone(),
            chain_reader_service.clone(),
            &txpool,
            storage1.clone(),
            storage2.clone(),
        )
        .await?;

        transfer_between_accounts(
            &receivers,
            gas_price,
            max_gas,
            account_service.clone(),
            config.clone(),
            chain_reader_service.clone(),
            &txpool,
            storage1.clone(),
            storage2.clone(),
        )
        .await?;

        Ok::<(), anyhow::Error>(())
    };

    fut.await
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
        let block_number = block.header().number();
        for transaction in block.body.transactions2 {
            self.transaction_data
                .entry(transaction.id())
                .or_default()
                .push(TransactionExecutionResult::Executed(now.clone(), block_number));
        }
        Ok(())
    }

    fn dump_results(&self) -> Result<()> {
        let dumper = ResultsDumper::new(&self.transaction_data);
        dumper.dump_results()
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
        ctx.subscribe::<NewHeadBlock>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<NewDagBlock>();
        ctx.unsubscribe::<NewDagBlockFromPeer>();
        ctx.unsubscribe::<NewHeadBlock>();

        if let Err(e) = self.dump_results() {
            error!("failed to dump the results: {:?}", e);
        }
        Ok(())
    }
}

impl EventHandler<Self, NewHeadBlock> for ObserverService {
    fn handle_event(&mut self, msg: NewHeadBlock, _ctx: &mut ServiceContext<Self>) {
       
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
