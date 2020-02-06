// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use config::NodeConfig;
use consensus::ConsensusActor;
use network::NetworkActor;
use std::path::PathBuf;
use structopt::StructOpt;

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
    let network = NetworkActor::launch(&config).unwrap();
    let _consensus = ConsensusActor::launch(&config, network);
    let _logger = args.no_logging;
    tokio::signal::ctrl_c().await.unwrap();
    println!("Ctrl-C received, shutting down");
    System::current().stop();
}
