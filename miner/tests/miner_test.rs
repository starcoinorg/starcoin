// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use actix::Actor;
use actix_rt::System;
use bus::BusActor;
use chain::ChainActor;
use config::NodeConfig;
use logger::prelude::*;
use network::network::NetworkAsyncService;
use network::network::PeerMsgBroadcasterActor;
use starcoin_account_api::AccountInfo;
use starcoin_genesis::Genesis;
use starcoin_miner::MinerActor;
use starcoin_miner::MinerClientActor;
use starcoin_network_rpc_api::gen_client::get_rpc_info;
use starcoin_state_service::ChainStateActor;
use std::sync::Arc;
use storage::Storage;
use sync::SyncActor;
use tokio::time::{delay_for, Duration};
use traits::ChainAsyncService;
use txpool::{TxPool, TxPoolService};
use types::{
    account_address,
    peer_info::{PeerId, PeerInfo, RpcInfo},
};

#[ignore]
#[test]
fn test_miner_with_ondemand_pacemaker() {
    ::logger::init_for_test();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut system = System::new("test");

    let fut = async move {
        let peer_id = Arc::new(PeerId::random());
        let conf = NodeConfig::random_for_test();
        let config = Arc::new(conf);
        let bus = BusActor::launch();

        let key_pair = config.network.network_keypair();
        let _address = account_address::from_public_key(&key_pair.public_key);

        let (storage, startup_info, genesis_hash) =
            Genesis::init_storage_for_test(config.net()).expect("init storage by genesis fail.");

        let txpool = {
            let best_block_id = *startup_info.get_master();
            TxPool::start(config.clone(), storage.clone(), best_block_id, bus.clone())
        };

        let txpool_service = txpool.get_service();

        let mut rpc_proto_info = Vec::new();
        let chain_rpc_proto_info = get_rpc_info();
        rpc_proto_info.push((
            chain_rpc_proto_info.0.into(),
            RpcInfo::new(chain_rpc_proto_info.1),
        ));

        let (network, rx) = NetworkAsyncService::start(
            config.clone(),
            bus.clone(),
            genesis_hash,
            PeerInfo::new_only_proto(rpc_proto_info),
        );
        let _msg_broadcaster = PeerMsgBroadcasterActor::launch(network.clone(), bus.clone());

        let chain = ChainActor::launch(
            config.clone(),
            startup_info.clone(),
            storage.clone(),
            txpool_service.clone(),
        )
        .unwrap();

        let state_service = ChainStateActor::launch(bus.clone(), storage.clone(), None).unwrap();
        // network rpc server
        network_rpc::start_network_rpc_server(
            rx,
            chain.clone(),
            storage.clone(),
            state_service,
            txpool_service.clone(),
        )
        .unwrap();
        let miner_account = AccountInfo::random();
        let _miner = MinerActor::<TxPoolService, Storage>::launch(
            config.clone(),
            bus.clone(),
            storage.clone(),
            txpool_service.clone(),
            miner_account,
            startup_info.clone(),
        );
        MinerClientActor::new(config.miner.client_config.clone(), config.net().consensus()).start();
        let _sync = SyncActor::launch(
            config.clone(),
            bus,
            peer_id,
            chain.clone(),
            txpool.get_service(),
            network.clone(),
            storage.clone(),
            startup_info,
        )
        .unwrap();

        delay_for(Duration::from_millis(6 * 10 * 1000)).await;

        let number = chain
            .clone()
            .master_head_header()
            .await
            .unwrap()
            .unwrap()
            .number();
        info!("{}", number);
        assert!(number > 0);

        delay_for(Duration::from_millis(1000)).await;
    };
    system.block_on(fut);
    drop(rt);
}
