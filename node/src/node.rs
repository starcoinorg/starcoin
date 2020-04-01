// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use bus::BusActor;
use chain::{ChainActor, ChainActorRef};
use consensus::{Consensus, ConsensusHeader};
use executor::TransactionExecutor;
use logger::prelude::*;
use miner::miner_client;
use miner::MinerActor;
use network::NetworkActor;
use starcoin_config::{NodeConfig, PacemakerStrategy};
use starcoin_genesis::Genesis;
use starcoin_rpc_server::JSONRpcActor;
use starcoin_state_service::ChainStateActor;
use starcoin_txpool_api::TxPoolAsyncService;
use starcoin_wallet_api::WalletAsyncService;
use starcoin_wallet_service::WalletActor;
use std::marker::PhantomData;
use std::sync::Arc;
use std::thread::JoinHandle;
use storage::cache_storage::CacheStorage;
use storage::db_storage::DBStorage;
use storage::{BlockStorageOp, StarcoinStorage};
use sync::{DownloadActor, ProcessActor, SyncActor};
use txpool::TxPoolRef;
use types::peer_info::PeerInfo;

pub struct Node<C, H, E>
where
    C: Consensus + Sync + Send + 'static,
    H: ConsensusHeader + Sync + Send + 'static,
    E: TransactionExecutor + Sync + Send + 'static,
{
    config: Arc<NodeConfig>,
    consensus: PhantomData<C>,
    consensus_header: PhantomData<H>,
    executor: PhantomData<E>,
}

impl<C, H, E> Node<C, H, E>
where
    C: Consensus + Sync + Send,
    H: ConsensusHeader + Sync + Send,
    E: TransactionExecutor + Sync + Send,
{
    pub fn new(config: Arc<NodeConfig>) -> Self {
        Self {
            config,
            consensus: Default::default(),
            consensus_header: Default::default(),
            executor: Default::default(),
        }
    }

    pub fn start(self) -> JoinHandle<()> {
        std::thread::spawn(move || self.do_start())
    }

    fn do_start(self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let handle = rt.handle().clone();
        let mut system = System::new("main");

        let fut = async move {
            let node_config = self.config;
            let bus = BusActor::launch();
            let cache_storage = Arc::new(CacheStorage::new());
            let db_storage = Arc::new(DBStorage::new(node_config.storage.clone().dir()));
            let storage =
                Arc::new(StarcoinStorage::new(cache_storage.clone(), db_storage.clone()).unwrap());

            let startup_info = match storage.get_startup_info().unwrap() {
                Some(startup_info) => {
                    info!("return from db");
                    startup_info
                }
                None => {
                    let genesis =
                        Genesis::new::<E, C, StarcoinStorage>(node_config.clone(), storage.clone())
                            .expect("init genesis fail.");
                    genesis.startup_info().clone()
                }
            };
            info!("Start chain with startup info: {:?}", startup_info);

            let account_service = WalletActor::launch(node_config.clone()).unwrap();

            //init default account
            let default_account = match account_service.clone().get_default_account().await.unwrap()
            {
                Some(account) => account_service
                    .clone()
                    .get_account(account.address)
                    .await
                    .unwrap()
                    .unwrap(),
                None => {
                    //TODO only in dev mod ?
                    let wallet_account = account_service
                        .clone()
                        .create_account("".to_string())
                        .await
                        .unwrap();
                    info!("Create default account: {}", wallet_account.address);
                    account_service
                        .clone()
                        .get_account(wallet_account.address)
                        .await
                        .unwrap()
                        .unwrap()
                }
            };

            let txpool = {
                let best_block_id = startup_info.head.get_head();
                TxPoolRef::start(
                    node_config.tx_pool.clone(),
                    storage.clone(),
                    best_block_id,
                    bus.clone(),
                )
            };

            let network = NetworkActor::launch(node_config.clone(), bus.clone(), handle.clone());

            let head_block = storage
                .get_block(startup_info.head.get_head())
                .unwrap()
                .expect("Head block must exist.");

            let chain_state_service = ChainStateActor::launch(
                node_config.clone(),
                bus.clone(),
                storage.clone(),
                Some(head_block.header().state_root()),
            )
            .unwrap();

            let chain = ChainActor::launch(
                node_config.clone(),
                startup_info,
                storage.clone(),
                Some(network.clone()),
                bus.clone(),
                txpool.clone(),
            )
            .unwrap();

            let _json_rpc = JSONRpcActor::launch(
                node_config.clone(),
                txpool.clone(),
                account_service,
                chain_state_service,
            );
            let receiver = if node_config.miner.pacemaker_strategy == PacemakerStrategy::Ondemand {
                Some(txpool.clone().subscribe_txns().await.unwrap())
            } else {
                None
            };
            let _miner =
                MinerActor::<C, E, TxPoolRef, ChainActorRef<E, C>, StarcoinStorage, H>::launch(
                    node_config.clone(),
                    bus.clone(),
                    storage.clone(),
                    txpool.clone(),
                    chain.clone(),
                    receiver,
                    default_account,
                );
            let peer_info = Arc::new(PeerInfo::random());
            let process_actor = ProcessActor::<E, C>::launch(
                Arc::clone(&peer_info),
                chain.clone(),
                network.clone(),
                bus.clone(),
                storage.clone(),
            )
            .unwrap();
            let download_actor = DownloadActor::launch(
                peer_info,
                chain,
                network.clone(),
                bus.clone(),
                storage.clone(),
            )
            .unwrap();
            let _sync = SyncActor::launch(bus, process_actor, download_actor).unwrap();
            handle.spawn(miner_client::MinerClient::main_loop(
                node_config.miner.stratum_server,
            ));
            tokio::signal::ctrl_c().await.unwrap();
            info!("Ctrl-C received, shutting down");
        };

        system.block_on(fut);
        System::current().stop();
    }
}
