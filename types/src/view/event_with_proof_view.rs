// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{view::accumulator_proof_view::AccumulatorProofView, view::str_view::StrView};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EventWithProofView {
    /// event is serialized bytes in bcs format.
    pub event: StrView<Vec<u8>>,
    pub proof: AccumulatorProofView,
}
//
// impl From<EventWithProof> for EventWithProofView {
//     fn from(origin: EventWithProof) -> Self {
//         Self {
//             event: StrView(origin.event.encode().expect("encode event should success")),
//             proof: origin.proof.into(),
//         }
//     }
// }
//
// impl TryFrom<EventWithProofView> for EventWithProof {
//     type Error = anyhow::Error;
//
//     fn try_from(value: EventWithProofView) -> Result<Self, Self::Error> {
//         Ok(Self {
//             event: ContractEvent::decode(value.event.0.as_slice())?,
//             proof: value.proof.into(),
//         })
//     }
// }
