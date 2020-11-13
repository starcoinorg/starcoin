// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use actix::System;
use logger::prelude::*;
use starcoin_config::MinerClientConfig;
use starcoin_miner_client::job_client::JobRpcClient;
use starcoin_miner_client::miner::MinerClientService;
use starcoin_rpc_client::RpcClient;
use starcoin_service_registry::{RegistryAsyncService, RegistryService};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Default)]
#[structopt(name = "starcoin-miner", about = "Starcoin Miner")]
pub struct StarcoinOpt {
    #[structopt(long, short = "a", default_value = "127.0.0.1:9870")]
    pub server: String,
    #[structopt(long, short = "n", default_value = "1")]
    pub thread_num: u16,
    #[structopt(long, short = "p")]
    pub plugin_path: Option<String>,
}

fn main() {
    let _logger_handle = logger::init();
    let opts: StarcoinOpt = StarcoinOpt::from_args();
    let config = {
        MinerClientConfig {
            server: Some(opts.server.clone()),
            plugin_path: opts.plugin_path,
            thread_num: opts.thread_num,
            enable_stderr: true,
        }
    };

    let mut rt = tokio_compat::runtime::Runtime::new().unwrap();
    let client = match RpcClient::connect_websocket(&format!("ws://{}", opts.server), &mut rt) {
        Ok(c) => c,
        Err(err) => {
            error!(
                "Failed to connect to starcoin node: {}, error: {}",
                opts.server, err
            );
            std::process::exit(-1);
        }
    };

    let mut system = System::builder()
        .stop_on_panic(true)
        .name("starcoin-miner")
        .build();
    let registry = RegistryService::launch();
    if let Err(err) = system.block_on(async move {
        let job_client = JobRpcClient::new(client);
        registry.put_shared(config).await?;
        registry.put_shared(job_client).await?;
        registry
            .register::<MinerClientService<JobRpcClient>>()
            .await
    }) {
        error!("Failed to set up miner client:{}", err);
    }
    if let Err(err) = system.run() {
        error!("Failed to run miner client:{}", err);
    }
}
