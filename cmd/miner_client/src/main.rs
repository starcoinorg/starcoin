// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use starcoin_config::{ConsensusStrategy, MinerClientConfig};
use starcoin_miner_client::job_client::JobRpcClient;
use starcoin_miner_client::miner::MinerClient;
use starcoin_rpc_client::RpcClient;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Default)]
#[structopt(name = "starcoin-miner", about = "Starcoin Miner")]
pub struct StarcoinOpt {
    #[structopt(long, short = "a", default_value = "127.0.0.1:9870")]
    pub server: String,
    #[structopt(long, short = "n", default_value = "1")]
    pub thread_num: u16,
    #[structopt(long, short = "c", default_value = "argon")]
    pub consensus: ConsensusStrategy,
}

fn main() {
    let _logger_handle = logger::init();
    let opts: StarcoinOpt = StarcoinOpt::from_args();
    let config = {
        MinerClientConfig {
            server: Some(opts.server.clone()),
            thread_num: opts.thread_num,
            enable_stderr: true,
        }
    };
    let mut rt = tokio_compat::runtime::Runtime::new().unwrap();
    let client = RpcClient::connect_websocket(&format!("ws://{}", opts.server), &mut rt).unwrap();
    rt.block_on_std(async move {
        let job_client = JobRpcClient::new(client);
        let mut miner_client = MinerClient::new(config, opts.consensus, job_client);
        miner_client.start().await.unwrap();
    });
}
