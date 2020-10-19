// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::MINER_METRICS;
use crypto::hash::PlainCryptoHash;
use crypto::HashValue;
use starcoin_metrics::HistogramTimer;
use types::{
    block::{Block, BlockTemplate},
    U256,
};

pub struct MintTask {
    pub(crate) mining_hash: HashValue,
    block_template: BlockTemplate,
    difficulty: U256,
    metrics_timer: HistogramTimer,
}

impl MintTask {
    pub fn new(block_template: BlockTemplate, difficulty: U256) -> MintTask {
        let mining_hash = block_template.as_raw_block_header(difficulty).crypto_hash();
        let metrics_timer = MINER_METRICS
            .block_mint_time
            .with_label_values(&["mint"])
            .start_timer();
        MintTask {
            mining_hash,
            block_template,
            difficulty,
            metrics_timer,
        }
    }

    pub fn finish(self, nonce: u64) -> Block {
        let block = self.block_template.into_block(nonce, self.difficulty);
        self.metrics_timer.observe_duration();
        block
    }
}
