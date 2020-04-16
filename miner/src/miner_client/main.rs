use config::{ConsensusStrategy, MinerConfig};
use futures::executor;
use logger::{self, prelude::*};
use starcoin_miner::MinerClient;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Default)]
#[structopt(name = "starcoin-miner", about = "Starcoin Miner")]
pub struct StarcoinOpt {
    #[structopt(long, short = "a", default_value = "0.0.0.0:9940")]
    pub stratum_server: String,
    #[structopt(long, short = "n", default_value = "1")]
    pub thread_num: u16,
}

fn main() {
    let _logger_handle = logger::init();
    let opts: StarcoinOpt = StarcoinOpt::from_args();
    let config = {
        let mut cfg = MinerConfig::default();
        cfg.consensus_strategy = ConsensusStrategy::Argon;
        cfg.stratum_server = opts
            .stratum_server
            .parse()
            .expect("Invalid stratum server address");
        cfg.thread_num = opts.thread_num;
        cfg
    };
    executor::block_on(async move {
        match MinerClient::new(config).await {
            Err(e) => error!("Start miner client failed:{:?}", e),
            Ok(mut miner_client) => miner_client.start().await,
        }
    });
}
