// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use bus::BusActor;
use chain::{ChainActor, ChainActorRef};
use config::{NodeConfig, PacemakerStrategy};
use consensus::{dummy::DummyConsensus, Consensus};
use crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use crypto::{test_utils::KeyPair, Uniform};
use executor::{mock_executor::MockExecutor, TransactionExecutor};
use json_rpc::JSONRpcActor;
use logger::prelude::*;
use miner::MinerActor;
use network::NetworkActor;
use starcoin_genesis::Genesis;
use std::path::PathBuf;
use std::sync::Arc;
use storage::{memory_storage::MemoryStorage, BlockChainStore, BlockStorageOp, StarcoinStorage};
use structopt::StructOpt;
use sync::{DownloadActor, ProcessActor, SyncActor};
use traits::TxPoolAsyncService;
use txpool::TxPoolRef;
use txpool::{CachedSeqNumberClient, TxPool};
use types::peer_info::PeerInfo;

#[derive(Debug, StructOpt)]
#[structopt(about = "Starcoin Node")]
struct Args {
    #[structopt(short = "f", long, parse(from_os_str))]
    /// Path to NodeConfig
    config: Option<PathBuf>,
    #[structopt(short = "d", long)]
    /// Disable logging
    no_logging: bool,
}

#[actix_rt::main]
async fn main() {
    logger::init();
    let args = Args::from_args();

    let node_config = Arc::new(NodeConfig::load_or_default(
        args.config.as_ref().map(PathBuf::as_path),
    ));

    let keypair = gen_keypair();
    let bus = BusActor::launch();
    let repo = Arc::new(MemoryStorage::new());
    let storage = Arc::new(StarcoinStorage::new(repo).unwrap());
    let startup_info = match storage.get_startup_info().unwrap() {
        Some(startup_info) => startup_info,
        None => {
            let genesis =
                Genesis::new::<MockExecutor, StarcoinStorage>(node_config.clone(), storage.clone())
                    .expect("init genesis fail.");
            genesis.startup_info().clone()
        }
    };
    info!("Start chain with startup info: {:?}", startup_info);
    let seq_number_client = CachedSeqNumberClient::new(storage.clone());
    let txpool = TxPool::start(seq_number_client);
    //node config
    // let mut config = NodeConfig::default();
    // config.network.listen = format!("/ip4/127.0.0.1/tcp/{}", config::get_available_port());
    // let node_config = Arc::new(config);
    let network = NetworkActor::launch(node_config.clone(), bus.clone(), txpool.clone(), keypair);
    let chain = ChainActor::launch(
        node_config.clone(),
        startup_info,
        storage.clone(),
        Some(network.clone()),
        bus.clone(),
        txpool.clone(),
    )
    .unwrap();
    let _json_rpc = JSONRpcActor::launch(node_config.clone(), txpool.clone());
    let receiver = if node_config.miner.pacemaker_strategy == PacemakerStrategy::Ondemand {
        Some(txpool.clone().subscribe_txns().await.unwrap())
    } else {
        None
    };
    let _miner = MinerActor::<
        DummyConsensus,
        MockExecutor,
        TxPoolRef,
        ChainActorRef<ChainActor>,
        StarcoinStorage,
    >::launch(
        node_config.clone(),
        bus.clone(),
        storage.clone(),
        txpool.clone(),
        chain.clone(),
        receiver,
    );
    let peer_info = Arc::new(PeerInfo::random());
    let process_actor = ProcessActor::launch(
        Arc::clone(&peer_info),
        chain.clone(),
        network.clone(),
        bus.clone(),
    )
    .unwrap();
    let download_actor =
        DownloadActor::launch(peer_info, chain, network.clone(), bus.clone()).unwrap();
    let _sync = SyncActor::launch(bus, process_actor, download_actor).unwrap();
    let _logger = args.no_logging;
    tokio::signal::ctrl_c().await.unwrap();
    info!("Ctrl-C received, shutting down");
    System::current().stop();
}

fn gen_keypair() -> Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> {
    use rand::prelude::*;

    let mut seed_rng = rand::rngs::OsRng::new().expect("can't access OsRng");
    let seed_buf: [u8; 32] = seed_rng.gen();
    let mut rng0: StdRng = SeedableRng::from_seed(seed_buf);
    let account_keypair: Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> =
        Arc::new(KeyPair::generate_for_testing(&mut rng0));
    account_keypair
}
