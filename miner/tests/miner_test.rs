// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bus::Bus;
use consensus::Consensus;
use crypto::hash::PlainCryptoHash;
use futures::executor::block_on;
use starcoin_config::NodeConfig;
use std::sync::Arc;
use types::{
    block::BlockTemplate,
    system_events::{GenerateBlockEvent, MintBlockEvent, NewHeadBlock, SubmitSealEvent},
};

#[stest::test]
fn test_miner() {
    let mut config = NodeConfig::random_for_test();
    config.miner.enable_miner_client = false;
    let handle = test_helper::run_node_by_config(Arc::new(config)).unwrap();
    let fut = async move {
        let bus = handle.start_handle().bus.clone();
        let new_block_receiver = bus.clone().oneshot::<NewHeadBlock>().await.unwrap();
        bus.clone()
            .broadcast(GenerateBlockEvent::new(false))
            .await
            .unwrap();
        // mint client handle mint block event
        let mint_block_event = bus
            .clone()
            .oneshot::<MintBlockEvent>()
            .await
            .unwrap()
            .await
            .unwrap();
        let nonce = handle
            .start_handle()
            .config
            .net()
            .consensus()
            .solve_consensus_nonce(mint_block_event.minting_hash, mint_block_event.difficulty);
        // mint client submit seal
        bus.broadcast(SubmitSealEvent {
            nonce,
            header_hash: mint_block_event.minting_hash,
        })
        .await
        .unwrap();
        let mined_block = new_block_receiver.await.unwrap().0.get_block().clone();
        assert_eq!(mined_block.header.nonce, nonce);
        let raw_header =
            BlockTemplate::from_block(mined_block).as_raw_block_header(mint_block_event.difficulty);
        assert_eq!(mint_block_event.minting_hash, raw_header.crypto_hash());
        handle.stop().unwrap();
    };
    block_on(fut);
}
