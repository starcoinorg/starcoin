use std::{collections::HashMap, env::temp_dir, sync::Arc};

use anyhow::{bail, format_err, Ok, Result};
use rayon::vec;
use starcoin_chain_api::message::{ChainRequest, ChainResponse};
use starcoin_chain_service::ChainReaderService;
use starcoin_config::{
    genesis_config::CustomNetworkID, BaseConfig, ChainNetworkID, NodeConfig, StarcoinOpt,
};
use starcoin_logger::{
    prelude::{info, LevelFilter},
    LoggerHandle,
};
use starcoin_miner::generate_block_event_pacemaker::GenerateBlockEventPacemaker;
use starcoin_service_registry::{RegistryAsyncService, ServiceRef};
use starcoin_storage::{Storage, Storage2, Store};
use starcoin_transaction_builder::vm2::build_batch_transfer_txn as build_batch_transfer_txn2;
use starcoin_types::{
    block::BlockHeader, genesis_config::ChainId, multi_transaction::MultiSignedUserTransaction,
    system_events::DeterminedDagBlock,
};
use starcoin_vm2_account_api::{
    message::{AccountRequest, AccountResponse},
    AccountInfo,
};
use starcoin_vm2_account_service::AccountService as AccountService2;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::transaction::SignedUserTransaction;
use starcoin_vm2_vm_runtime::starcoin_vm::StarcoinVM;
use starcoin_vm2_vm_types::state_view::StateReaderExt;
use test_helper::run_node_by_config;

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
    sender: Vec<AccountInfo>,
    receiver: Vec<AccountInfo>,
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
    for s in &sender {
        let next_seq = statedb2
            .get_account_resource(s.address.clone())?
            .sequence_number();
        next_seq_map.entry(s.address().clone()).or_insert(next_seq);
    }

    let mut signed_transactions = vec![];
    for index in 0..receiver.len() {
        let sender_address = sender.get(index).unwrap().address;
        let next_seq = *next_seq_map.get(&sender_address).unwrap();
        next_seq_map
            .entry(sender_address.clone())
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
                sender_address.clone(),
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

#[test]
fn test_transaction_in_custom_network() -> Result<()> {
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
    let _ = BaseConfig::load_with_opt(&opt)?;

    let global_opt = StarcoinOpt {
        net: Some(ChainNetworkID::Custom(CustomNetworkID::new(
            "my_chain".to_string(),
            ChainId::new(121),
        ))),
        base_data_dir: Some(path.clone()),
        ..Default::default()
    };

    opt.genesis_config = Some(
        path.join("my_chain/genesis_config2.json")
            .to_str()
            .unwrap()
            .to_string(),
    );
    let node_config = Arc::new(NodeConfig::load_with_opt(&global_opt)?);

    // let node_config = Arc::new(NodeConfig::random_for_test());
    let node = run_node_by_config(node_config.clone()).unwrap();
    // StarcoinVM::set_concurrency_level_once(num_cpus::get());
    let registry = node.registry();
    let storage1 = node.storage();
    let storage2 = node.storage2();

    let genesis = node.genesis();

    let fut = async move {
        // let generate_block = registry.service_ref::<GenerateBlockEventPacemaker>().await?;
        let log_handler = registry.get_shared::<Arc<LoggerHandle>>().await?;
        log_handler.update_level(LevelFilter::Info);

        let chain_reader_service = registry.service_ref::<ChainReaderService>().await?;
        loop {
            // generate_block.notify(DeterminedDagBlock)?;
            let current_header = get_current_header(chain_reader_service.clone()).await?;
            info!("jacktest: current_header: {}", current_header.number());
            if current_header.number() > 300 {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        }

        let txpool = registry
            .get_shared::<starcoin_txpool::TxPoolService>()
            .await?;
        let account_service = registry.service_ref::<AccountService2>().await?;
        let config = registry.get_shared::<Arc<NodeConfig>>().await?;

        let default_account = default_account(account_service.clone()).await?;
        let receivers = create_account(2000, account_service.clone()).await?;
        let signed_transactions = build_transaction_to_send_token_to_account(
            vec![default_account; receivers.len()],
            receivers,
            100,
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
        // let transactions = txpool
        //     .inner
        //     .get_pending(100, config.net().time_service().now_secs())?;
        // info!(
        //     "jacktest: transaction in halley pending: {}",
        //     transactions.len()
        // );
        // for txn in transactions {
        //     match txn.signed() {
        //         MultiSignedUserTransaction::VM1(signed_user_transaction) => info!(
        //             "jacktest: transaction1 in halley pending: id: {}",
        //             signed_user_transaction.id()
        //         ),
        //         MultiSignedUserTransaction::VM2(signed_user_transaction) => info!(
        //             "jacktest: transaction2 in halley pending: id: {}",
        //             signed_user_transaction.id()
        //         ),
        //     }
        // }

        loop {
            // let executed_block = node.generate_block()?;
            // info!(
            //     "jacktest: transaction in halley executed block: {}",
            //     executed_block.body.transactions2.len()
            // );
            let transactions = txpool
                .inner
                .get_pending(100, config.net().time_service().now_secs())?;
            if transactions.is_empty() {
                break;
            }
        }

        // StarcoinVM::set_concurrency_level_once(num_cpus::get());

        Ok(())
    };
    tokio::runtime::Runtime::new().unwrap().block_on(fut)?;

    Ok(())
}
