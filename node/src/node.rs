// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::{clock::delay_for, prelude::*};
use anyhow::Result;
use futures::StreamExt;
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
use starcoin_sync_api::SyncMetadata;
use starcoin_traits::Consensus;
use starcoin_txpool::{TxPool, TxPoolService};
use starcoin_txpool_api::TxPoolSyncService;
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
    let genesis = match Genesis::load(config.data_dir()) {
        Ok(Some(genesis)) => {
            let expect_genesis = Genesis::build(config.net())?;
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
                let genesis = Genesis::build(config.net())?;
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

    //init default account
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

    let head_block_hash = *startup_info.get_master();

    let txpool = TxPool::start(
        config.tx_pool.clone(),
        storage.clone(),
        head_block_hash,
        bus.clone(),
    );
    let txpool_service = txpool.get_service();
    let txpool_ref = txpool.get_async_service();

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
    let self_info = PeerInfo::new(
        peer_id.clone(),
        head_block.header().number(),
        head_block_info.get_total_difficulty(),
        head_block_hash,
    );
    let network_config = config.clone();
    let network_bus = bus.clone();
    let network_handle = handle.clone();
    let (network,rpc_rx) = Arbiter::new()
        .exec(move || -> (NetworkAsyncService,futures::channel::mpsc::UnboundedReceiver<RawRpcRequestMessage>){
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

    let chain_state_service = ChainStateActor::launch(
        config.clone(),
        bus.clone(),
        storage.clone(),
        Some(head_block.header().state_root()),
    )?;

    let sync_metadata = SyncMetadata::new(config.clone(), bus.clone());

    let chain_config = config.clone();
    let chain_storage = storage.clone();
    let chain_network = network.clone();
    let chain_bus = bus.clone();
    let chain_txpool = txpool_ref.clone();
    let chain_sync_metadata = sync_metadata.clone();

    let chain = Arbiter::new()
        .exec(move || -> Result<ChainActorRef<C>> {
            ChainActor::launch(
                chain_config,
                startup_info,
                chain_storage,
                Some(chain_network),
                chain_bus,
                chain_txpool,
                chain_sync_metadata,
            )
        })
        .await??;

    let pubsub_service = {
        let txn_receiver = txpool_service.subscribe_txns();
        let service = PubSubService::new();
        service.start_transaction_subscription_handler(txn_receiver);
        service.start_chain_notify_handler(bus.clone(), storage.clone());
        service
    };

    let (json_rpc, _io_handler) = RpcActor::launch(
        config.clone(),
        txpool_service,
        chain.clone(),
        account_service,
        chain_state_service,
        Some(pubsub_service),
        Some(network.clone()),
        Some(logger_handle),
    )?;

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
    let sync_txpool = txpool_ref.clone();
    let sync_network = network.clone();
    let sync_storage = storage.clone();
    let sync_sync_metadata = sync_metadata.clone();
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
                sync_sync_metadata,
                rpc_rx,
            )
        })
        .await??;

    delay_for(Duration::from_secs(1)).await;
    bus.clone().broadcast(SyncBegin).await?;

    info!("Waiting sync ......");
    let mut sync_event_receiver = sync_event_receiver_future
        .await
        .expect("Subscribe system event error.");
    let _ = sync_event_receiver.next().await;
    info!("Waiting sync finished.");
    let miner = MinerActor::<C, TxPoolService, ChainActorRef<C>, Storage>::launch(
        config.clone(),
        bus,
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
    Ok(NodeStartHandle {
        _miner_actor: miner,
        _sync_actor: sync,
        _rpc_actor: json_rpc,
        _miner_client: miner_client,
    })
}
