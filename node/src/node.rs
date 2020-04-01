// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_bus::BusActor;
use starcoin_chain::{ChainActor, ChainActorRef};
use starcoin_config::{NodeConfig, PacemakerStrategy};
use starcoin_consensus::{Consensus, ConsensusHeader};
use starcoin_executor::executor::Executor;
use starcoin_genesis::Genesis;
use starcoin_logger::prelude::*;
use starcoin_miner::{miner_client, MinerActor};
use starcoin_network::NetworkActor;
use starcoin_rpc_server::JSONRpcActor;
use starcoin_state_service::ChainStateActor;
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::{BlockStorageOp, StarcoinStorage};
use starcoin_sync::{DownloadActor, ProcessActor, SyncActor};
use starcoin_txpool::TxPoolRef;
use starcoin_txpool_api::TxPoolAsyncService;
use starcoin_types::peer_info::PeerInfo;
use starcoin_wallet_api::WalletAsyncService;
use starcoin_wallet_service::WalletActor;
use std::sync::Arc;
use tokio::runtime::Handle;

pub async fn start<C, H>(config: Arc<NodeConfig>, handle: Handle) -> Result<()>
where
    C: Consensus + 'static,
    H: ConsensusHeader + 'static,
{
    let bus = BusActor::launch();
    let cache_storage = Arc::new(CacheStorage::new());
    let db_storage = Arc::new(DBStorage::new(config.storage.clone().dir()));
    let storage = Arc::new(StarcoinStorage::new(
        cache_storage.clone(),
        db_storage.clone(),
    )?);

    let startup_info = match storage.get_startup_info()? {
        Some(startup_info) => {
            info!("return from db");
            startup_info
        }
        None => {
            let genesis =
                Genesis::new::<Executor, C, StarcoinStorage>(config.clone(), storage.clone())
                    .expect("init genesis fail.");
            genesis.startup_info().clone()
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

    let network = NetworkActor::launch(config.clone(), bus.clone(), handle.clone());

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
    )?;

    let _json_rpc = JSONRpcActor::launch(
        config.clone(),
        txpool.clone(),
        account_service,
        chain_state_service,
    );
    let receiver = if config.miner.pacemaker_strategy == PacemakerStrategy::Ondemand {
        Some(txpool.clone().subscribe_txns().await?)
    } else {
        None
    };
    let _miner = MinerActor::<
        C,
        Executor,
        TxPoolRef,
        ChainActorRef<Executor, C>,
        StarcoinStorage,
        H,
    >::launch(
        config.clone(),
        bus.clone(),
        storage.clone(),
        txpool.clone(),
        chain.clone(),
        receiver,
        default_account,
    );
    let peer_info = Arc::new(PeerInfo::random());
    let process_actor = ProcessActor::<Executor, C>::launch(
        Arc::clone(&peer_info),
        chain.clone(),
        network.clone(),
        bus.clone(),
        storage.clone(),
    )?;
    let download_actor = DownloadActor::launch(peer_info, chain, network.clone(), bus.clone())?;
    let _sync = SyncActor::launch(bus, process_actor, download_actor)?;
    //TODO manager MinerClient by actor.
    let stratum_server = config.miner.stratum_server;
    handle.spawn(miner_client::MinerClient::main_loop(stratum_server));
    Ok(())
}
