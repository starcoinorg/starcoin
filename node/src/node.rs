// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::{clock::delay_for, prelude::*};
use anyhow::Result;
use futures::StreamExt;
use starcoin_block_relayer::BlockRelayer;
use starcoin_bus::{Bus, BusActor};
use starcoin_chain::{ChainActor, ChainActorRef};
use starcoin_config::NodeConfig;
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use starcoin_logger::LoggerHandle;
use starcoin_miner::MinerActor;
use starcoin_miner::MinerClientActor;
use starcoin_network::{NetworkActor, NetworkAsyncService, RawRpcRequestMessage};
use starcoin_rpc_server::module::PubSubService;
use starcoin_rpc_server::RpcActor;
use starcoin_state_service::ChainStateActor;
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::{storage::StorageInstance, BlockStore, Storage};
use starcoin_sync::SyncActor;
use starcoin_traits::Consensus;
use starcoin_txpool::{TxPool, TxPoolService};
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::account_config::association_address;
use starcoin_types::peer_info::PeerInfo;
use starcoin_types::system_events::{SyncBegin, SyncDone};
use starcoin_wallet_api::WalletAsyncService;
use starcoin_wallet_service::WalletActor;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Handle;

/// This exit code means is that the node failed to start and required human intervention.
/// Node start script can do auto task when meet this exist code.
static EXIT_CODE_NEED_HELP: i32 = 120;

pub struct NodeStartHandle<C>
where
    C: Consensus + 'static,
{
    _miner_actor: Addr<MinerActor<C, TxPoolService, ChainActorRef<C>, Storage>>,
    _sync_actor: Addr<SyncActor<C>>,
    _rpc_actor: Addr<RpcActor>,
    _miner_client: Option<Addr<MinerClientActor>>,
}

//TODO this method should in Genesis.
fn load_and_check_genesis(config: &NodeConfig, init: bool) -> Result<Genesis> {
    let genesis = match Genesis::load_from_dir(config.data_dir()) {
        Ok(Some(genesis)) => {
            let expect_genesis = Genesis::load(config.net())?;
            if genesis.block().header().id() != expect_genesis.block().header().id() {
                error!("Genesis version mismatch, please clean you data_dir.");
                std::process::exit(EXIT_CODE_NEED_HELP);
            }
            genesis
        }
        Err(e) => {
            error!("Genesis file load error: {:?}", e);
            std::process::exit(EXIT_CODE_NEED_HELP);
        }
        Ok(None) => {
            if init {
                let genesis = Genesis::load(config.net())?;
                genesis.save(config.data_dir())?;
                info!("Build and save new genesis: {}", genesis);
                genesis
            } else {
                error!("Genesis file not exist, please clean you data_dir.");
                std::process::exit(EXIT_CODE_NEED_HELP);
            }
        }
    };
    Ok(genesis)
}

pub async fn start<C>(
    config: Arc<NodeConfig>,
    logger_handle: Arc<LoggerHandle>,
    handle: Handle,
) -> Result<NodeStartHandle<C>>
where
    C: Consensus + 'static,
{
    let bus = BusActor::launch();

    let sync_event_receiver_future = bus.clone().channel::<SyncDone>();
    debug!("init storage.");
    let cache_storage = Arc::new(CacheStorage::new());
    let db_storage = Arc::new(DBStorage::new(config.storage.clone().dir()));
    let storage = Arc::new(Storage::new(StorageInstance::new_cache_and_db_instance(
        cache_storage.clone(),
        db_storage.clone(),
    ))?);
    debug!("load startup_info.");
    let (startup_info, genesis_hash) = match storage.get_startup_info() {
        Ok(Some(startup_info)) => {
            info!("Get startup info from db");
            info!("Check genesis file.");
            let genesis = load_and_check_genesis(&config, false)?;
            match storage.get_block(genesis.block().header().id())? {
                Some(block) => {
                    if *genesis.block() == block {
                        info!("Check genesis db block ok!");
                    } else {
                        error!("Genesis db storage mismatch, please clean you data_dir.");
                        std::process::exit(EXIT_CODE_NEED_HELP);
                    }
                }
                _ => {
                    error!("Genesis block is not exist in db storage.");
                    std::process::exit(EXIT_CODE_NEED_HELP);
                }
            }
            (startup_info, genesis.block().header().id())
        }
        Ok(None) => {
            let genesis = load_and_check_genesis(&config, true)?;
            let genesis_hash = genesis.block().header().id();
            let startup_info = genesis.execute(storage.clone())?;
            (startup_info, genesis_hash)
        }
        Err(e) => {
            error!(
                "Load startup info fail: {:?}, please check or clean data_dir.",
                e
            );
            std::process::exit(EXIT_CODE_NEED_HELP);
        }
    };
    info!("Start chain with startup info: {}", startup_info);

    let account_service = WalletActor::launch(config.clone())?;

    //Init default account
    let default_account = match account_service.clone().get_default_account().await? {
        Some(account) => account,
        None => {
            //TODO only in dev mod ?
            let wallet_account = account_service
                .clone()
                .create_account("".to_string())
                .await?;
            info!("Create default account: {}", wallet_account.address);
            wallet_account
        }
    };

    //Only dev network pre_mine_config contains private_key.
    if let Some(pre_mine_config) = config.net().get_config().pre_mine_config.as_ref() {
        if let Some(private_key) = pre_mine_config.private_key.as_ref() {
            let association_account = account_service
                .clone()
                .get_account(association_address())
                .await?;
            if association_account.is_none() {
                account_service
                    .clone()
                    .import_account(
                        association_address(),
                        private_key.to_bytes().to_vec(),
                        "".to_string(),
                    )
                    .await?;
                info!("Import association account to wallet.");
            }
        }
    }

    let head_block_hash = *startup_info.get_master();

    let txpool = TxPool::start(
        config.tx_pool.clone(),
        storage.clone(),
        head_block_hash,
        bus.clone(),
    );
    let txpool_service = txpool.get_service();

    let head_block = match storage.get_block(head_block_hash)? {
        Some(block) => block,
        None => panic!("can't get block by hash {}", head_block_hash),
    };
    let head_block_info = match storage.get_block_info(head_block_hash)? {
        Some(block_info) => block_info,
        None => panic!("can't get block info by hash {}", head_block_hash),
    };
    let peer_id = config
        .network
        .self_peer_id
        .clone()
        .expect("Self peer_id must has been set.");
    let mut rpc_proto_info = Vec::new();
    let sync_rpc_proto_info = starcoin_sync::helper::sync_rpc_info();
    rpc_proto_info.push((sync_rpc_proto_info.0.into(), sync_rpc_proto_info.1));
    let self_info = PeerInfo::new_with_proto(
        peer_id.clone(),
        head_block_info.get_total_difficulty(),
        head_block.header().clone(),
        rpc_proto_info,
    );
    let network_config = config.clone();
    let network_bus = bus.clone();
    let network_handle = handle.clone();
    let (network, rpc_rx) = Arbiter::new()
        .exec(move || -> (NetworkAsyncService, futures::channel::mpsc::UnboundedReceiver<RawRpcRequestMessage>){
            NetworkActor::launch(
                network_config,
                network_bus,
                network_handle,
                genesis_hash,
                self_info,
            )
        })
        .await?;

    let head_block = storage
        .get_block(*startup_info.get_master())?
        .expect("Head block must exist.");
    let _block_relayer = BlockRelayer::new(bus.clone(), txpool.get_service(), network.clone())?;
    let chain_state_service = ChainStateActor::launch(
        config.clone(),
        bus.clone(),
        storage.clone(),
        Some(head_block.header().state_root()),
    )?;

    let chain_config = config.clone();
    let chain_storage = storage.clone();
    let chain_bus = bus.clone();
    let chain_txpool_service = txpool_service.clone();

    let chain = Arbiter::new()
        .exec(move || -> Result<ChainActorRef<C>> {
            ChainActor::launch(
                chain_config,
                startup_info,
                chain_storage,
                chain_bus,
                chain_txpool_service,
            )
        })
        .await??;

    info!("Self peer_id is: {}", peer_id.to_base58());
    info!(
        "Self address is: {}",
        config
            .network
            .self_address
            .as_ref()
            .expect("Self connect address must has been set.")
    );
    let peer_id = Arc::new(peer_id);
    let sync_config = config.clone();
    let sync_bus = bus.clone();
    let sync_chain = chain.clone();
    let sync_txpool = txpool_service.clone();
    let sync_network = network.clone();
    let sync_storage = storage.clone();
    let sync = Arbiter::new()
        .exec(move || -> Result<Addr<SyncActor<C>>> {
            SyncActor::launch(
                sync_config,
                sync_bus,
                peer_id,
                sync_chain,
                sync_txpool,
                sync_network,
                sync_storage,
                rpc_rx,
            )
        })
        .await??;

    delay_for(Duration::from_secs(1)).await;

    let waiting_sync = !(config.network.disable_seed
        || (config.network.seeds.is_empty() && config.net().get_config().boot_nodes.is_empty()));
    if waiting_sync {
        bus.clone().broadcast(SyncBegin).await?;

        info!("Waiting sync ......");
        let mut sync_event_receiver = sync_event_receiver_future
            .await
            .expect("Subscribe system event error.");

        let _ = sync_event_receiver.next().await;
        info!("Waiting sync finished.");
    }

    let miner = MinerActor::<C, TxPoolService, ChainActorRef<C>, Storage>::launch(
        config.clone(),
        bus.clone(),
        storage.clone(),
        txpool.get_service(),
        chain.clone(),
        default_account,
    )?;
    let miner_client = if config.miner.enable {
        Some(MinerClientActor::new(config.miner.clone()).start())
    } else {
        None
    };

    let pubsub_service = {
        let txn_receiver = txpool_service.subscribe_txns();
        let service = PubSubService::new();
        service.start_transaction_subscription_handler(txn_receiver);
        service.start_chain_notify_handler(bus, storage.clone());
        service
    };

    let (json_rpc, _io_handler) = RpcActor::launch(
        config,
        txpool_service,
        chain,
        account_service,
        chain_state_service,
        Some(pubsub_service),
        Some(network),
        Some(logger_handle),
    )?;

    Ok(NodeStartHandle {
        _miner_actor: miner,
        _sync_actor: sync,
        _rpc_actor: json_rpc,
        _miner_client: miner_client,
    })
}
