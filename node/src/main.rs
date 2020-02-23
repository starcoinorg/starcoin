// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use bus::BusActor;
use chain::ChainActor;
use config::NodeConfig;
use consensus::{dummy::DummyConsensus, Consensus};
use executor::{mock_executor::MockExecutor, TransactionExecutor};
use json_rpc::JSONRpcActor;
use miner::MinerActor;
use network::NetworkActor;
use std::path::PathBuf;
use std::sync::Arc;
use storage::{memory_storage::MemoryStorage, StarcoinStorage};
use structopt::StructOpt;
use txpool::{TxPoolActor, TxPoolRef};

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
    let args = Args::from_args();

    let config = Arc::new(NodeConfig::load_or_default(
        args.config.as_ref().map(PathBuf::as_path),
    ));
    let bus = BusActor::launch();
    let repo = Arc::new(MemoryStorage::new());
    let storage = Arc::new(StarcoinStorage::new(repo).unwrap());
    let txpool = TxPoolActor::launch(config.clone(), bus.clone(), storage.clone()).unwrap();
    let _chain = ChainActor::launch(config.clone(), storage.clone()).unwrap();
    let _network = NetworkActor::launch(config.clone(), bus.clone(), txpool.clone()).unwrap();
    let _json_rpc = JSONRpcActor::launch(config.clone(), txpool.clone());
    let _miner = MinerActor::<DummyConsensus, MockExecutor, TxPoolRef>::launch(
        config.clone(),
        bus.clone(),
        storage.clone(),
        txpool.clone(),
    );
    let _logger = args.no_logging;
    tokio::signal::ctrl_c().await.unwrap();
    println!("Ctrl-C received, shutting down");
    System::current().stop();
}
