// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use actix::System;
use clap::Parser;
use starcoin_config::MinerClientConfig;
use starcoin_logger::prelude::*;
use starcoin_miner_client::miner::MinerClientService;
use starcoin_miner_client::stratum_client::StratumJobClient;
use starcoin_miner_client::stratum_client_service::{
    StratumClientService, StratumClientServiceServiceFactory,
};
use starcoin_miner_client::ConsensusStrategy;
use starcoin_service_registry::{RegistryAsyncService, RegistryService};
use starcoin_stratum::rpc::LoginRequest;
use starcoin_time_service::RealTimeService;
use std::sync::Arc;

#[derive(Debug, Clone, Parser, Default)]
#[clap(name = "starcoin-miner", about = "Starcoin Miner")]
pub struct StarcoinOpt {
    #[clap(long, short = 'a', default_value = "127.0.0.1:9880")]
    pub server: String,
    #[clap(long, short = 'u')]
    pub user: String,
    #[clap(long, short = 'n', default_value = "1")]
    pub thread_num: u16,
    #[clap(long, short = 'p')]
    pub plugin_path: Option<String>,
    #[clap(long, short = 'g', default_value_t = ConsensusStrategy::Argon)]
    pub algo: ConsensusStrategy,
}

fn main() {
    let _logger_handle = starcoin_logger::init();
    let opts: StarcoinOpt = StarcoinOpt::parse();
    let algo = opts.algo;
    let config = {
        MinerClientConfig {
            server: Some(opts.server.clone()),
            plugin_path: opts.plugin_path,
            miner_thread: opts.thread_num,
            enable_stderr: true,
        }
    };
    let user = opts.user;
    let system = System::with_tokio_rt(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .on_thread_stop(|| info!("starcoin-miner thread stopped"))
            .thread_name("starcoin-miner")
            .build()
            .expect("failed to create tokio runtime for starcoin-miner")
    });
    if let Err(err) = system.block_on(async move {
        let registry = RegistryService::launch();
        registry.put_shared(config).await?;
        let stratum_cli_srv = registry
            .register_by_factory::<StratumClientService, StratumClientServiceServiceFactory>()
            .await?;
        let time_srv = Arc::new(RealTimeService::new());
        let login = LoginRequest {
            login: user.clone(),
            pass: user,
            agent: "stc-miner".into(),
            algo: None,
        };

        let stratum_job_client = StratumJobClient::new(stratum_cli_srv, time_srv, login, algo);
        registry.put_shared(stratum_job_client).await?;
        registry
            .register::<MinerClientService<StratumJobClient>>()
            .await
    }) {
        error!("Failed to set up miner client:{}", err);
    }
    if let Err(err) = system.run() {
        error!("Failed to run miner client:{}", err);
    }
}
