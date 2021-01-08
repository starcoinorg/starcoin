// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::MINER_METRICS;
use crate::BlockHeaderExtra;
use starcoin_metrics::HistogramTimer;
use types::block::{Block, BlockTemplate};

pub struct MintTask {
    pub(crate) minting_blob: Vec<u8>,
    block_template: BlockTemplate,
    metrics_timer: HistogramTimer,
}

impl std::fmt::Debug for MintTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MintTask")
            .field("mining_blob", &self.minting_blob)
            .field("difficulty", &self.block_template.difficulty)
            .field("block_template.number", &self.block_template.number)
            .field(
                "block_template.parent_hash",
                &self.block_template.parent_hash,
            )
            .finish()
    }
}

impl MintTask {
    pub fn new(block_template: BlockTemplate) -> MintTask {
        let minting_blob = block_template.as_pow_header_blob();
        let metrics_timer = MINER_METRICS
            .block_mint_time
            .with_label_values(&["mint"])
            .start_timer();
        MintTask {
            minting_blob,
            block_template,
            metrics_timer,
        }
    }

    pub fn finish(self, nonce: u32, extra: BlockHeaderExtra) -> Block {
        let block = self.block_template.into_block(nonce, extra);
        self.metrics_timer.observe_duration();
        block
    }
}
