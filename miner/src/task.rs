// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::MinerMetrics;
use crate::BlockHeaderExtra;
use starcoin_metrics::HistogramTimer;
use types::block::{Block, BlockTemplate};

pub struct MintTask {
    pub(crate) minting_blob: Vec<u8>,
    pub(crate) block_template: BlockTemplate,
    metrics_timer: Option<HistogramTimer>,
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
    pub fn new(block_template: BlockTemplate, metrics: Option<MinerMetrics>) -> MintTask {
        let minting_blob = block_template.as_pow_header_blob();
        let metrics_timer = metrics
            .as_ref()
            .map(|metrics| metrics.block_mint_time.start_timer());
        MintTask {
            minting_blob,
            block_template,
            metrics_timer,
        }
    }

    pub fn finish(self, nonce: u32, extra: BlockHeaderExtra) -> Block {
        let block = self.block_template.into_block(nonce, extra);
        if let Some(metrics_timer) = self.metrics_timer {
            metrics_timer.observe_duration();
        }
        block
    }
}
