// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2
#![deny(clippy::arithmetic_side_effects)]

use anyhow::{bail, format_err, Result};
use serde::{Deserialize, Serialize};
use starcoin_accumulator::proof::AccumulatorProof;
use starcoin_state_api::StateWithProof;
use starcoin_types::{
    multi_transaction::MultiSignedUserTransaction, transaction::RichTransactionInfo,
};
use starcoin_vm2_state_api::StateWithProof as StateWithProof2;
use starcoin_vm2_vm_types::{
    access_path::AccessPath as AccessPath2, contract_event::ContractEvent as ContractEvent2,
};

mod chain;
mod errors;
pub mod message;
mod service;

pub use chain::{Chain, ChainReader, ChainWriter, ExecutedBlock, MintedUncleNumber, VerifiedBlock};
pub use errors::*;
pub use service::{ChainAsyncService, ReadableChainService, WriteableChainService};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::contract_event::ContractEvent;

#[derive(Clone, Debug)]
pub struct ExcludedTxns {
    pub discarded_txns: Vec<MultiSignedUserTransaction>,
    pub untouched_txns: Vec<MultiSignedUserTransaction>,
}

impl ExcludedTxns {
    pub fn absorb(mut self, mut other: ExcludedTxns) -> Self {
        self.discarded_txns.append(&mut other.discarded_txns);
        self.untouched_txns.append(&mut other.untouched_txns);
        self
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct EventWithProof {
    pub event: ContractEvent,
    pub proof: AccumulatorProof,
}

impl EventWithProof {
    pub fn verify(&self, expect_root: HashValue, event_index: u64) -> Result<()> {
        self.proof
            .verify(expect_root, self.event.crypto_hash(), event_index)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TransactionInfoWithProof {
    pub transaction_info: RichTransactionInfo,
    pub proof: AccumulatorProof,
    pub event_proof: Option<EventWithProof>,
    pub state_proof: Option<StateWithProof>,
}

impl TransactionInfoWithProof {
    pub fn verify(
        &self,
        expect_root: HashValue,
        transaction_index: u64,
        event_index: Option<u64>,
        access_path: Option<AccessPath>,
    ) -> Result<()> {
        self.proof
            .verify(expect_root, self.transaction_info.id(), transaction_index)
            .map_err(|e| format_err!("transaction info proof verify failed: {}", e))?;
        match (self.event_proof.as_ref(), event_index) {
            (Some(event_proof), Some(event_index)) => {
                event_proof
                    .verify(self.transaction_info.event_root_hash(), event_index)
                    .map_err(|e| format_err!("event proof verify failed: {}", e))?;
            }
            (Some(_), None) => {
                // skip
            }
            (None, None) => {
                // skip
            }
            (None, Some(event_index)) => {
                bail!(
                    "TransactionInfoWithProof's event_proof is None, cannot verify event_index: {}",
                    event_index
                );
            }
        };
        match (self.state_proof.as_ref(), access_path) {
            (Some(state_proof), Some(access_path)) => {
                state_proof
                    .verify(self.transaction_info.state_root_hash(), access_path)
                    .map_err(|e| format_err!("state proof verify failed: {}", e))?;
            }
            (Some(_), None) => {
                // skip
            }
            (None, None) => {
                // skip
            }
            (None, Some(access_path)) => {
                bail!(
                    "TransactionInfoWithProof's state_proof is None, cannot verify access_path: {}",
                    access_path
                );
            }
        };
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct EventWithProof2 {
    pub event: ContractEvent2,
    pub proof: AccumulatorProof,
}

impl EventWithProof2 {
    pub fn verify(&self, expect_root: HashValue, event_index: u64) -> Result<()> {
        self.proof
            .verify(expect_root, self.event.crypto_hash(), event_index)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TransactionInfoWithProof2 {
    pub transaction_info: RichTransactionInfo,
    pub proof: AccumulatorProof,
    pub event_proof: Option<EventWithProof2>,
    pub state_proof: Option<StateWithProof2>,
}

impl TransactionInfoWithProof2 {
    pub fn verify(
        &self,
        expect_root: HashValue,
        transaction_index: u64,
        event_index: Option<u64>,
        access_path: Option<AccessPath2>,
    ) -> Result<()> {
        self.proof
            .verify(expect_root, self.transaction_info.id(), transaction_index)
            .map_err(|e| format_err!("transaction info proof verify failed: {}", e))?;
        match (self.event_proof.as_ref(), event_index) {
            (Some(event_proof), Some(event_index)) => {
                event_proof
                    .verify(self.transaction_info.event_root_hash(), event_index)
                    .map_err(|e| format_err!("event proof verify failed: {}", e))?;
            }
            (Some(_), None) => {
                // skip
            }
            (None, None) => {
                // skip
            }
            (None, Some(event_index)) => {
                bail!(
                    "TransactionInfoWithProof2's event_proof is None, cannot verify event_index: {}",
                    event_index
                );
            }
        };
        match (self.state_proof.as_ref(), access_path) {
            (Some(state_proof), Some(access_path)) => {
                state_proof
                    .verify(self.transaction_info.state_root_hash(), access_path)
                    .map_err(|e| format_err!("state proof verify failed: {}", e))?;
            }
            (Some(_), None) => {
                // skip
            }
            (None, None) => {
                // skip
            }
            (None, Some(access_path)) => {
                bail!(
                    "TransactionInfoWithProof's state_proof is None, cannot verify access_path: {}",
                    access_path
                );
            }
        };
        Ok(())
    }
}
