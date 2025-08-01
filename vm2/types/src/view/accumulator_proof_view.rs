// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::proof::AccumulatorProof;
use starcoin_crypto::HashValue;

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccumulatorProofView {
    pub siblings: Vec<HashValue>,
}

impl From<AccumulatorProof> for AccumulatorProofView {
    fn from(origin: AccumulatorProof) -> Self {
        Self {
            siblings: origin.siblings,
        }
    }
}

impl From<AccumulatorProofView> for AccumulatorProof {
    fn from(view: AccumulatorProofView) -> Self {
        Self {
            siblings: view.siblings,
        }
    }
}
