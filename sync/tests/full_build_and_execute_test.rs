use std::{collections::HashMap, env::temp_dir, sync::Arc};

use anyhow::{bail, format_err, Ok, Result};
use chrono::Local;
use futures::channel::mpsc;
use starcoin_chain_api::message::{ChainRequest, ChainResponse};
use starcoin_chain_service::ChainReaderService;
use starcoin_config::{
    genesis_config::CustomNetworkID, BaseConfig, ChainNetworkID, NodeConfig, StarcoinOpt,
};
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_executor::VMMetrics;
use starcoin_genesis::Genesis;
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
use starcoin_vm2_vm_types::{
    account_address::AccountAddress, state_view::StateReaderExt, transaction, PeerId,
};
use std::fs::OpenOptions;
use std::io::{self, Write};
use test_helper::run_node_with_all_service;

async fn create_account(
    accoun_number: u32,
    account_service: ServiceRef<AccountService2>,
) -> Result<Vec<AccountInfo>> {
    let mut results = vec![];
    for _ in 0..accoun_number {
        let receiver = match account_service
            .send(AccountRequest::CreateAccount("".to_string()))
            .await??
        {
            AccountResponse::AccountInfo(account) => *account,
            _ => bail!("Unexpect response type."),
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
        _ => bail!("Unexpect response type."),
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
            400_000_00,
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
            _ => bail!("Unexpect response type."),
        }
        let signed_transaction = match account_service
            .send(AccountRequest::SignTxn {
                txn: Box::new(transaction),
                signer: sender_address,
            })
            .await??
        {
            AccountResponse::SignedTxn(signed_transction) => *signed_transction,
            _ => bail!("Unexpect response type."),
        };
        signed_transactions.push(signed_transaction);
    }

    Ok(signed_transactions)
}

async fn get_current_header(
    chain_reader_serivice: ServiceRef<ChainReaderService>,
) -> Result<BlockHeader> {
    let current_header = match chain_reader_serivice
        .send(ChainRequest::CurrentHeader())
        .await??
    {
        ChainResponse::BlockHeader(header) => *header,
        _ => bail!("Unexpect response type."),
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

// #[ignore = "This is a benchmark test, not a unit test"]
#[test]
fn test_full_build_and_execute_in_custom_network() -> Result<()> {
    let mut opt = StarcoinOpt {
        net: Some(ChainNetworkID::Custom(CustomNetworkID::new(
            "my_chain".to_string(),
            ChainId::new(121),
        ))),
        ..Default::default()
    };
    let path = temp_dir();
    opt.base_data_dir = Some(path.clone());
    opt.genesis_config = Some("halley".to_string());

    // will create genesis config files in path
    let _ = BaseConfig::load_with_opt(&opt)?;

    let global_opt = StarcoinOpt {
        net: Some(ChainNetworkID::Custom(CustomNetworkID::new(
            "my_chain".to_string(),
            ChainId::new(121),
        ))),
        base_data_dir: Some(path.clone()),
        ..Default::default()
    };

    // will load genesis config files in path
    let node_config = Arc::new(NodeConfig::load_with_opt(&global_opt)?);

    // let node_config = Arc::new(NodeConfig::random_for_test());
    let node = run_node_with_all_service(node_config.clone()).unwrap();
    // StarcoinVM::set_concurrency_level_once(num_cpus::get());
    let registry = node.registry();
    let storage1 = node.storage();
    let storage2 = node.storage2();

    let account_count: u32 = 2000;
    let initial_balance: u128 = 10000000000;
    let initial_gas_fee: u128 = 4000000000;

    let fut = async move {
        // let generate_block = registry.service_ref::<GenerateBlockEventPacemaker>().await?;
        let log_handler = registry.get_shared::<Arc<LoggerHandle>>().await?;
        log_handler.update_level(LevelFilter::Info);

        let observer = registry.register::<ObserverService>().await?;

        let account_service = registry.service_ref::<AccountService2>().await?;
        let default_account = default_account(account_service.clone()).await?;

        let chain_reader_service = registry.service_ref::<ChainReaderService>().await?;
        loop {
            // generate_block.notify(DeterminedDagBlock)?;
            let current_header = get_current_header(chain_reader_service.clone()).await?;
            let default_account_balance = match get_balance(
                default_account.address,
                storage1.clone(),
                storage2.clone(),
                current_header.id(),
            )
            .await
            {
                std::result::Result::Ok(balance) => balance,
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
                // get enough token to pay for gas
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

        // transfer token from default account to receivers
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

        // transfer token from default account to receivers
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

        Ok(())
    };
    tokio::runtime::Runtime::new().unwrap().block_on(fut)?;

    node.stop_service(ObserverService::service_name().to_string())?;
    node.stop()?;
    // std::fs::remove_dir_all(path)?;

    Ok(())
}

enum TransactionExecutionResult {
    Added(String),
    Rejected(String),
    Culled(String),
    Executed(String),
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
    header: HashValue,
    dag: BlockDAG,
    storage1: Arc<Storage>,
}

impl ObserverService {
    fn new(header: HashValue, dag: BlockDAG, storage1: Arc<Storage>) -> Result<Self> {
        Ok(Self {
            transaction_data: HashMap::new(),
            header,
            dag,
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
                .or_insert_with(Vec::new)
                .push(TransactionExecutionResult::Executed(now.clone()));
        }
        Ok(())
    }

    fn dump_results(&mut self) -> Result<()> {
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open("/Users/jack/Documents/code/rust/flexidag/starcoin-jack/main/starcoin/sync/transaction_results.txt")?;
        for (transaction, results) in &self.transaction_data {
            writeln!(
                file,
                "transaction id: {}, results: {:?}",
                *transaction, results
            )?;
        }
        Ok(())
    }
}

impl ServiceFactory<Self> for ObserverService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let genesis = ctx.get_shared::<Genesis>()?;
        let storage1 = ctx.get_shared::<Arc<Storage>>()?;
        let dag = ctx.get_shared::<BlockDAG>()?;
        Self::new(genesis.block().id(), dag, storage1)
    }
}

impl ActorService for ObserverService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        // ctx.subscribe::<NewHeadBlock>();
        ctx.subscribe::<NewDagBlock>();
        ctx.subscribe::<NewDagBlockFromPeer>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        // ctx.unsubscribe::<NewHeadBlock>();
        ctx.unsubscribe::<NewDagBlock>();
        ctx.unsubscribe::<NewDagBlockFromPeer>();

        if let Err(e) = self.dump_results() {
            error!("failed to dump the reuslts: {:?}", e);
        }
        Ok(())
    }
}

// impl EventHandler<Self, NewHeadBlock> for ObserverService {
//     fn handle_event(&mut self, msg: NewHeadBlock, ctx: &mut ServiceContext<Self>) {
//     }
// }

impl EventHandler<Self, NewDagBlock> for ObserverService {
    fn handle_event(&mut self, msg: NewDagBlock, ctx: &mut ServiceContext<Self>) {
        match self.update_transaction_status(msg.executed_block.block().id()) {
            std::result::Result::Ok(_) => (),
            Err(e) => error!("failed to update transactions status for: {:?}", e),
        }
    }
}

impl EventHandler<Self, NewDagBlockFromPeer> for ObserverService {
    fn handle_event(&mut self, msg: NewDagBlockFromPeer, ctx: &mut ServiceContext<Self>) {
        match self.update_transaction_status(msg.executed_block.id()) {
            std::result::Result::Ok(_) => (),
            Err(e) => error!("failed to update transactions status for: {:?}", e),
        }
    }
}

impl EventHandler<Self, Arc<[(HashValue, TxStatus)]>> for ObserverService {
    fn handle_event(&mut self, msg: Arc<[(HashValue, TxStatus)]>, ctx: &mut ServiceContext<Self>) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        for transaction_event in msg.as_ref() {
            match transaction_event.1 {
                starcoin_types::transaction::TxStatus::Added => self
                    .transaction_data
                    .entry(transaction_event.0)
                    .or_insert_with(Vec::new)
                    .push(TransactionExecutionResult::Added(now.clone())),
                starcoin_types::transaction::TxStatus::Rejected => self
                    .transaction_data
                    .entry(transaction_event.0)
                    .or_insert_with(Vec::new)
                    .push(TransactionExecutionResult::Rejected(now.clone())),
                starcoin_types::transaction::TxStatus::Culled => self
                    .transaction_data
                    .entry(transaction_event.0)
                    .or_insert_with(Vec::new)
                    .push(TransactionExecutionResult::Culled(now.clone())),
                _ => self
                    .transaction_data
                    .entry(transaction_event.0)
                    .or_insert_with(Vec::new)
                    .push(TransactionExecutionResult::Other(format!(
                        "{}({})",
                        transaction_event.1,
                        now.clone()
                    ))),
            }
        }
    }
}
