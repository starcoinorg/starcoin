// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::str_view::StrView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_crypto::HashValue;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AccumulatorInfoView {
    /// Accumulator root hash
    pub accumulator_root: HashValue,
    /// Frozen subtree roots of this accumulator.
    pub frozen_subtree_roots: Vec<HashValue>,
    /// The total number of leaves in this accumulator.
    pub num_leaves: StrView<u64>,
    /// The total number of nodes in this accumulator.
    pub num_nodes: StrView<u64>,
}

impl AccumulatorInfoView {
    pub fn into_info(self) -> AccumulatorInfo {
        AccumulatorInfo::new(
            self.accumulator_root,
            self.frozen_subtree_roots,
            self.num_leaves.0,
            self.num_nodes.0,
        )
    }
}

impl From<AccumulatorInfo> for AccumulatorInfoView {
    fn from(info: AccumulatorInfo) -> Self {
        Self {
            accumulator_root: info.accumulator_root,
            frozen_subtree_roots: info.frozen_subtree_roots.clone(),
            num_leaves: info.num_leaves.into(),
            num_nodes: info.num_nodes.into(),
        }
    }
}
