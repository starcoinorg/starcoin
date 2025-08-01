// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{block::BlockInfo, view::accumulator_info_view::AccumulatorInfoView};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_uint::U256;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BlockInfoView {
    /// Block hash
    pub block_hash: HashValue,
    /// The total difficulty.
    #[schemars(with = "String")]
    pub total_difficulty: U256,
    /// The transaction accumulator info
    pub txn_accumulator_info: AccumulatorInfoView,
    /// The block accumulator info.
    pub block_accumulator_info: AccumulatorInfoView,
}

impl BlockInfoView {
    pub fn into_info(self) -> BlockInfo {
        BlockInfo::new(
            self.block_hash,
            self.total_difficulty,
            self.txn_accumulator_info.into_info(),
            self.block_accumulator_info.into_info(),
        )
    }
}

impl From<BlockInfo> for BlockInfoView {
    fn from(block_info: BlockInfo) -> Self {
        Self {
            block_hash: block_info.block_id,
            total_difficulty: block_info.total_difficulty,
            txn_accumulator_info: block_info.txn_accumulator_info.into(),
            block_accumulator_info: block_info.block_accumulator_info.into(),
        }
    }
}
