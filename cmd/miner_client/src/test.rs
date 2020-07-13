use crate::miner::MinerClientActor;
use actix::Actor;
use actix_rt::System;
use bus::BusActor;
use config::MinerConfig;
use config::NodeConfig;
use consensus::argon::ArgonConsensus;
use futures_timer::Delay;
use logger::prelude::*;
use sc_stratum::{PushWorkHandler, Stratum};
use starcoin_miner::{
    miner::{MineCtx, Miner},
    stratum::StratumManager,
};
use std::sync::Arc;
use std::time::Duration;
use types::block::{Block, BlockBody, BlockHeader, BlockTemplate};
use types::U256;

#[test]
fn test_stratum_client() {
    ::logger::init_for_test();
    let mut system = System::new("test");
    system.block_on(async {
        let mut miner_config = MinerConfig::default();
        miner_config.consensus_strategy = config::ConsensusStrategy::Argon(4);
        let conf = Arc::new(NodeConfig::random_for_test());
        let mut miner = Miner::<ArgonConsensus>::new(BusActor::launch(), conf);
        let stratum = {
            let dispatcher = Arc::new(StratumManager::new(miner.clone()));
            Stratum::start(&miner_config.stratum_server, dispatcher, None).unwrap()
        };
        Delay::new(Duration::from_millis(3000)).await;
        info!("started stratum server");
        let mine_ctx = {
            let header = BlockHeader::random();
            let body = BlockBody::default();
            let block = Block::new(header, body);
            let block_template = BlockTemplate::from_block(block);
            let difficulty: U256 = 1.into();
            MineCtx::new(block_template, difficulty)
        };
        let _addr = MinerClientActor::new(miner_config).start();
        miner.set_mint_job(mine_ctx);
        for _ in 1..10 {
            stratum.push_work_all(miner.get_mint_job()).unwrap();
            Delay::new(Duration::from_millis(2000)).await;
        }
    });
}
