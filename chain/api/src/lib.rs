// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2
#![deny(clippy::integer_arithmetic)]

use serde::{Deserialize, Serialize};
use starcoin_accumulator::proof::AccumulatorProof;
use starcoin_state_api::{StateProof, StateWithProof};
use starcoin_vm_types::transaction::SignedUserTransaction;

mod chain;
mod errors;
pub mod message;
mod service;

#[derive(Clone, Debug)]
pub struct ExcludedTxns {
    pub discarded_txns: Vec<SignedUserTransaction>,
    pub untouched_txns: Vec<SignedUserTransaction>,
}

pub use chain::{Chain, ChainReader, ChainWriter, ExecutedBlock, MintedUncleNumber, VerifiedBlock};
pub use errors::*;
pub use service::{ChainAsyncService, ReadableChainService, WriteableChainService};
use starcoin_vm_types::contract_event::ContractEvent;

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct EventWithProof {
    pub event: ContractEvent,
    pub proof: AccumulatorProof,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TransactionProof {
    pub txn_proof: AccumulatorProof,
    pub event_proof: Option<EventWithProof>,
    pub state_proof: Option<StateWithProof>,
}
