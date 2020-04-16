// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::{clock::delay_for, prelude::*};
use anyhow::{bail, Result};
use starcoin_bus::{Bus, BusActor};
use starcoin_chain::{ChainActor, ChainActorRef};
use starcoin_config::{NodeConfig, PacemakerStrategy};
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use starcoin_logger::LoggerHandle;
use starcoin_miner::MinerActor;
use starcoin_miner::MinerClientActor;
use starcoin_network::NetworkActor;
use starcoin_rpc_server::RpcActor;
use starcoin_state_service::ChainStateActor;
use starcoin_storage::block_info::BlockInfoStore;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::{storage::StorageInstance, BlockStore, Storage};
use starcoin_sync::SyncActor;
use starcoin_sync_api::SyncMetadata;
use starcoin_traits::{Consensus, ConsensusHeader};
use starcoin_txpool::TxPoolRef;
use starcoin_txpool_api::TxPoolAsyncService;
use starcoin_types::peer_info::PeerInfo;
use starcoin_types::system_events::SystemEvents;
use starcoin_wallet_api::WalletAsyncService;
use starcoin_wallet_service::WalletActor;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::stream::StreamExt;

pub struct NodeStartHandle<C, H>
where
    C: Consensus + 'static,
    H: ConsensusHeader + 'static,
{
    _miner_actor: Addr<MinerActor<C, TxPoolRef, ChainActorRef<C>, Storage, H>>,
    _sync_actor: Addr<SyncActor<C>>,
    _rpc_actor: Addr<RpcActor>,
    _miner_client: Addr<MinerClientActor>,
}

pub async fn start<C, H>(
    config: Arc<NodeConfig>,
    logger_handle: Arc<LoggerHandle>,
    handle: Handle,
) -> Result<NodeStartHandle<C, H>>
where
    C: Consensus + 'static,
    H: ConsensusHeader + 'static,
{
    let bus = BusActor::launch();

    let sync_event_receiver_future = bus.clone().channel::<SystemEvents>();

    let cache_storage = Arc::new(CacheStorage::new());
    let db_storage = Arc::new(DBStorage::new(config.storage.clone().dir()));
    let storage = Arc::new(
        Storage::new(StorageInstance::new_cache_and_db_instance(
            cache_storage.clone(),
            db_storage.clone(),
        ))
        .unwrap(),
    );

    let sync_metadata = SyncMetadata::new(config.clone(), bus.clone());

    let (startup_info, genesis_hash) = match storage.get_startup_info()? {
        Some(startup_info) => {
            info!("Get startup info from db");
            info!("Check genesis file.");
            // Genesis may be change in dev and halley network.
            let genesis = Genesis::load(config.data_dir())?
                .expect("Load genesis file must exist in data_dir.");
            let expect_genesis = Genesis::build(config.net())?;
            if genesis.block().header().id() != expect_genesis.block().header().id() {
                bail!("Genesis version mismatch, please clean you data_dir.")
            }
            //TODO verify genesis block in db.
            (startup_info, genesis.block().header().id())
        }
        None => {
            let genesis = match Genesis::load(config.data_dir())? {
                Some(genesis) => {
                    info!("Load genesis from data_dir: {}", genesis);
                    genesis
                }
                None => {
                    let genesis = Genesis::build(config.net())?;
                    info!("Build genesis: {}", genesis);
                    genesis.save(config.data_dir())?;
                    genesis
                }
            };
            let genesis_hash = genesis.block().header().id();
            let startup_info = genesis.execute(storage.clone())?;
            (startup_info, genesis_hash)
        }
    };
    info!("Start chain with startup info: {:?}", startup_info);

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

    let txpool = {
        let best_block_id = startup_info.head.get_head();
        TxPoolRef::start(
            config.tx_pool.clone(),
            storage.clone(),
            best_block_id,
            bus.clone(),
        )
    };
    let head_block_hash = startup_info.head.get_head();
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
        head_block_info.get_total_difficult(),
        startup_info.head.get_head(),
    );
    let network = NetworkActor::launch(
        config.clone(),
        bus.clone(),
        handle.clone(),
        genesis_hash,
        self_info,
    );

    let head_block = storage
        .get_block(startup_info.head.get_head())?
        .expect("Head block must exist.");

    let chain_state_service = ChainStateActor::launch(
        config.clone(),
        bus.clone(),
        storage.clone(),
        Some(head_block.header().state_root()),
    )?;

    let chain = ChainActor::launch(
        config.clone(),
        startup_info,
        storage.clone(),
        Some(network.clone()),
        bus.clone(),
        txpool.clone(),
        sync_metadata.clone(),
    )?;

    let (json_rpc, _io_handler) = RpcActor::launch(
        config.clone(),
        txpool.clone(),
        chain.clone(),
        account_service,
        chain_state_service,
        Some(network.clone()),
        Some(logger_handle),
    )?;
    let receiver = if config.miner.pacemaker_strategy == PacemakerStrategy::Ondemand {
        Some(txpool.clone().subscribe_txns().await?)
    } else {
        None
    };

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
    let sync = SyncActor::launch(
        config.clone(),
        bus.clone(),
        peer_id,
        chain.clone(),
        txpool.clone(),
        network.clone(),
        storage.clone(),
        sync_metadata.clone(),
    )?;

    delay_for(Duration::from_secs(1)).await;
    bus.clone().broadcast(SystemEvents::SyncBegin()).await?;

    info!("Waiting sync ......");
    let mut sync_event_receiver = sync_event_receiver_future
        .await
        .expect("Subscribe system event error.");
    sync_event_receiver.any(|event| event.is_sync_done()).await;
    info!("Waiting sync finished.");
    let miner = MinerActor::<C, TxPoolRef, ChainActorRef<C>, Storage, H>::launch(
        config.clone(),
        bus,
        storage.clone(),
        txpool.clone(),
        chain.clone(),
        receiver,
        default_account,
    )?;
    let miner_client = MinerClientActor::new(config.miner.clone()).start();
    Ok(NodeStartHandle {
        _miner_actor: miner,
        _sync_actor: sync,
        _rpc_actor: json_rpc,
        _miner_client: miner_client,
    })
}
