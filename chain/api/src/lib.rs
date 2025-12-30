// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2
#![deny(clippy::arithmetic_side_effects)]

use anyhow::{bail, format_err, Result};
use bcs_ext::BCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::proof::AccumulatorProof;
use starcoin_types::{
    multi_access_path::MultiAccessPath,
    multi_transaction::MultiSignedUserTransaction,
    transaction::{StcRichTransactionInfo, StcTransactionInfo},
};
use starcoin_vm2_vm_types::contract_event::ContractEvent as ContractEvent2;

mod chain;
mod errors;
pub mod message;
pub mod range_locate;
mod service;

pub use chain::{
    Chain, ChainReader, ChainWriter, ExecutedBlock, MintedUncleNumber, MultiStateProof,
    VerifiedBlock,
};
pub use errors::*;
pub use service::{ChainAsyncService, ReadableChainService};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_vm_types::contract_event::ContractEvent;

use starcoin_vm2_types::view::{
    AccumulatorProofView as AccumulatorProofView2, EventWithProofView as EventWithProofView2,
    StrView as StrView2,
};

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
    pub transaction_info: StcRichTransactionInfo,
    pub proof: AccumulatorProof,
    pub final_proof: AccumulatorProof,
    pub event_proof: Option<MultiEventWithProof>,
    pub state_proof: Option<MultiStateProof>,
    pub final_state_proof: Option<MultiStateProof>,
}

impl TransactionInfoWithProof {
    pub fn event_root_hash(&self) -> HashValue {
        match &self.transaction_info.transaction_info {
            StcTransactionInfo::V1(transaction_info) => transaction_info.event_root_hash(),
            StcTransactionInfo::V2(transaction_info) => transaction_info.event_root_hash(),
        }
    }

    pub fn state_root_hash(&self) -> Option<HashValue> {
        match &self.transaction_info.transaction_info {
            StcTransactionInfo::V1(transaction_info) => transaction_info.state_root_hash(),
            StcTransactionInfo::V2(transaction_info) => transaction_info.state_root_hash(),
        }
    }
    pub fn verify(
        &self,
        expect_root: HashValue,
        transaction_index: u64,
        final_transaction_index: u64,
        final_transaction_info_id: HashValue,
        event_index: Option<u64>,
        access_path: Option<MultiAccessPath>,
        final_access_path: Option<MultiAccessPath>,
        final_state_root: Option<HashValue>,
    ) -> Result<()> {
        self.proof
            .verify(
                expect_root,
                self.transaction_info.inner_transaction_info_id(),
                transaction_index,
            )
            .map_err(|e| format_err!("transaction info proof verify failed: {}", e))?;
        self.final_proof
            .verify(
                expect_root,
                final_transaction_info_id,
                final_transaction_index,
            )
            .map_err(|e| format_err!("final transaction info proof verify failed: {}", e))?;

        match (self.event_proof.as_ref(), event_index) {
            (Some(event_proof), Some(event_index)) => {
                event_proof
                    .verify(self.event_root_hash(), event_index)
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
                    .verify(self.state_root_hash().ok_or_else(|| format_err!("state root is none maybe it is not the last transaction of a block?, its id is {}", self.transaction_info.transaction_hash()))?, access_path)
                    .map_err(|e| format_err!("state proof verify failed: {}", e))?;
            }
            (Some(_), None) | (None, None) => {
                // skip
            }
            (None, Some(_access_path)) => {
                bail!("TransactionInfoWithProof's state_proof is None, cannot verify access_path");
            }
        };
        self.verify_final_state_root(final_state_root, final_access_path)?;
        Ok(())
    }

    fn verify_final_state_root(
        &self,
        final_state_root: Option<HashValue>,
        final_access_path: Option<MultiAccessPath>,
    ) -> Result<()> {
        match (
            self.final_state_proof.as_ref(),
            final_state_root,
            final_access_path,
        ) {
            (Some(final_state_proof), Some(final_state_root), Some(final_access_path)) => {
                final_state_proof
                    .verify(final_state_root, final_access_path)
                    .map_err(|e| format_err!("state proof verify failed: {}", e))?;
            }
            (Some(_), Some(_), None) => {
                bail!("final_access_path is None, cannot verify final_state_proof");
            }
            (Some(_), None, Some(_)) => {
                bail!("final_state_root is None, cannot verify final_state_proof with provided final_access_path");
            }
            (None, Some(_), Some(_)) | (None, None, Some(_)) => {
                bail!("TransactionInfoWithProof's final_state_proof is None, cannot verify final_access_path");
            }
            (None, Some(_), None) => {
                bail!("TransactionInfoWithProof's final_state_proof is None, cannot verify final_state_root");
            }
            (Some(_), None, None) | (None, None, None) => {}
        }
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

impl From<EventWithProof2> for EventWithProofView2 {
    fn from(origin: EventWithProof2) -> Self {
        Self {
            event: StrView2(origin.event.encode().expect("encode event should succeed")),
            proof: AccumulatorProofView2 {
                siblings: origin.proof.siblings().to_vec(),
            },
        }
    }
}

impl TryFrom<EventWithProofView2> for EventWithProof2 {
    type Error = anyhow::Error;

    fn try_from(value: EventWithProofView2) -> Result<Self, Self::Error> {
        Ok(EventWithProof2 {
            event: ContractEvent2::decode(value.event.0.as_slice())?,
            proof: AccumulatorProof::new(value.proof.siblings),
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum MultiEventWithProof {
    VM1(EventWithProof),
    VM2(EventWithProof2),
}

impl MultiEventWithProof {
    pub(crate) fn verify(&self, event_root_hash: HashValue, event_index: u64) -> Result<()> {
        match self {
            Self::VM1(event_with_proof) => event_with_proof.verify(event_root_hash, event_index),
            Self::VM2(event_with_proof) => event_with_proof.verify(event_root_hash, event_index),
        }
    }
}
