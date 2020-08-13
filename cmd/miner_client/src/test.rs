// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::miner::MinerClientActor;
use crate::stratum::{parse_response, process_request};
use actix::Actor;
use bus::{Bus, BusActor};
use futures_timer::Delay;
use logger::prelude::*;
use sc_stratum::{PushWorkHandler, Stratum};
use starcoin_config::NodeConfig;
use starcoin_miner::{
    miner::{MineCtx, Miner},
    stratum::StratumManager,
};
use starcoin_types::{
    block::{Block, BlockBody, BlockHeader, BlockTemplate},
    system_events::MinedBlock,
    U256,
};
use std::sync::Arc;
use std::time::Duration;

#[stest::test]
async fn test_stratum_client() {
    let conf = Arc::new(NodeConfig::random_for_test());
    let miner_config = conf.miner.clone();
    let bus = BusActor::launch();
    let mut miner = Miner::new(bus.clone(), conf.clone());
    let stratum = {
        let dispatcher = Arc::new(StratumManager::new(miner.clone()));
        Stratum::start(&miner_config.stratum_server, dispatcher, None).unwrap()
    };
    Delay::new(Duration::from_millis(1000)).await;
    info!("started stratum server");
    let event_receiver = bus.oneshot::<MinedBlock>();
    let mine_ctx = {
        let header = BlockHeader::random();
        let body = BlockBody::default();
        let block = Block::new(header, body);
        let block_template = BlockTemplate::from_block(block);
        let difficulty: U256 = 1.into();
        MineCtx::new(block_template, difficulty)
    };
    let _addr =
        MinerClientActor::new(miner_config.client_config.clone(), conf.net().consensus()).start();
    //wait client connect to server.
    Delay::new(Duration::from_millis(1000)).await;

    miner.set_mint_job(mine_ctx);
    stratum
        .push_work_all(miner.get_mint_job().unwrap())
        .unwrap();
    let _mint_block = event_receiver.await.expect("expect MintBlock");
}

#[test]
fn test_json() {
    let json_str = r#"
        {"jsonrpc":"2.0","result":true,"id":0}
    "#;
    let result = parse_response::<bool>(json_str);
    assert!(result.is_ok(), "parse response error: {:?}", result.err());

    let json_str = r#"
    { "id": 19, "method": "mining.notify", "params": ["e419ff9f57cc615f1b9ee900097f6ce34ad5eaff61eda78414efa1c3fa9e8200","1"] }
    "#;
    let result = process_request(json_str);
    assert!(result.is_ok(), "process request fail:{:?}", result.err());
}
