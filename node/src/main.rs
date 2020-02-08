// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use bus::BusActor;
use config::NodeConfig;
use consensus::ConsensusActor;
use json_rpc::JSONRpcActor;
use network::NetworkActor;
use std::path::PathBuf;
use structopt::StructOpt;
use txpool::TxPoolActor;

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

    let config = NodeConfig::load_or_default(args.config.as_ref().map(PathBuf::as_path));
    let bus = BusActor::launch();
    let network = NetworkActor::launch(&config, bus.clone()).unwrap();
    let _consensus = ConsensusActor::launch(&config, network.clone());
    let txpool_actor_ref = TxPoolActor::launch(&config, bus, network).unwrap();
    let _json_rpc = JSONRpcActor::launch(&config, txpool_actor_ref);
    let _logger = args.no_logging;
    tokio::signal::ctrl_c().await.unwrap();
    println!("Ctrl-C received, shutting down");
    System::current().stop();
}
