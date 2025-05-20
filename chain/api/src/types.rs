// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::EventWithProof;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::proof::AccumulatorProof;
use starcoin_state_api::StateWithProof;
use starcoin_types::contract_event::StcContractEvent;
use starcoin_types::transaction::RichTransactionInfo;
use starcoin_vm2_state_api::StateWithProof as StateWithProof2;

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct StcEventWithProof {
    pub event: StcContractEvent,
    pub proof: AccumulatorProof,
}

impl From<EventWithProof> for StcEventWithProof {
    fn from(event_with_proof: EventWithProof) -> Self {
        Self {
            event: event_with_proof.event.into(),
            proof: event_with_proof.proof,
        }
    }
}

impl TryFrom<StcEventWithProof> for EventWithProof {
    type Error = anyhow::Error;
    fn try_from(value: StcEventWithProof) -> Result<Self, Self::Error> {
        Ok(Self {
            event: value.event.try_into()?,
            proof: value.proof,
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum StcStateWithProof {
    V1(StateWithProof),
    V2(StateWithProof2),
}

impl From<StateWithProof> for StcStateWithProof {
    fn from(state_with_proof: StateWithProof) -> Self {
        Self::V1(state_with_proof)
    }
}

impl From<StateWithProof2> for StcStateWithProof {
    fn from(state_with_proof: StateWithProof2) -> Self {
        Self::V2(state_with_proof)
    }
}

impl TryFrom<StcStateWithProof> for StateWithProof {
    type Error = anyhow::Error;
    fn try_from(value: StcStateWithProof) -> Result<Self, Self::Error> {
        match value {
            StcStateWithProof::V1(state_with_proof) => Ok(state_with_proof),
            StcStateWithProof::V2(_) => anyhow::bail!("V2 StateWithProof cannot be convert to V1"),
        }
    }
}

impl TryFrom<StcStateWithProof> for StateWithProof2 {
    type Error = anyhow::Error;
    fn try_from(value: StcStateWithProof) -> Result<Self, Self::Error> {
        match value {
            StcStateWithProof::V1(_) => anyhow::bail!("V1 StateWithProof cannot be convert to V2"),
            StcStateWithProof::V2(state_with_proof) => Ok(state_with_proof),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct StcTransactionInfoWithProof {
    pub transaction_info: RichTransactionInfo,
    pub proof: AccumulatorProof,
    pub event_proof: Option<StcEventWithProof>,
    pub state_proof: Option<StcStateWithProof>,
}

impl From<super::TransactionInfoWithProof> for StcTransactionInfoWithProof {
    fn from(proof: super::TransactionInfoWithProof) -> Self {
        Self {
            transaction_info: proof.transaction_info,
            proof: proof.proof,
            event_proof: proof.event_proof.map(Into::into),
            state_proof: proof.state_proof.map(Into::into),
        }
    }
}

impl TryFrom<StcTransactionInfoWithProof> for super::TransactionInfoWithProof {
    type Error = anyhow::Error;
    fn try_from(value: StcTransactionInfoWithProof) -> Result<Self, Self::Error> {
        Ok(Self {
            transaction_info: value.transaction_info,
            proof: value.proof,
            event_proof: value.event_proof.map(TryFrom::try_from).transpose()?,
            state_proof: value.state_proof.map(TryFrom::try_from).transpose()?,
        })
    }
}
