// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod stc_transaction;

use crate::{identifier::Identifier, language_storage::ModuleId};
use bcs_ext::Sample;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::inmemory::InMemoryAccumulator;
use starcoin_crypto::hash::{
    CryptoHash, CryptoHasher, PlainCryptoHash, SPARSE_MERKLE_PLACEHOLDER_HASH,
};
use starcoin_crypto::HashValue;
use starcoin_vm2_types::transaction::TransactionInfo as TransactionInfoV2;
use starcoin_vm2_types::vm_error::{AbortLocation, KeptVMStatus};
use starcoin_vm_types::contract_event::ContractEvent;
pub use starcoin_vm_types::transaction::*;
pub use stc_transaction::{StcTransaction, Transaction2};
use std::ops::Deref;

/// try to parse_transaction_argument and auto convert no address 0x hex string to Move's vector<u8>
pub fn parse_transaction_argument_advance(s: &str) -> anyhow::Result<TransactionArgument> {
    let arg = match parse_transaction_argument(s) {
        Ok(arg) => arg,
        Err(e) => {
            //auto convert 0xxx to vector<u8>
            match s.strip_prefix("0x") {
                Some(stripped) => TransactionArgument::U8Vector(hex::decode(stripped)?),
                None => return Err(e),
            }
        }
    };
    Ok(arg)
}

/// `TransactionInfo` is the object we store in the transaction accumulator. It consists of the
/// transaction as well as the execution result of this transaction.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub struct TransactionInfo {
    /// The hash of this transaction.
    pub transaction_hash: HashValue,

    /// The root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    pub state_root_hash: HashValue,

    /// The root hash of Merkle Accumulator storing all events emitted during this transaction.
    pub event_root_hash: HashValue,

    /// The amount of gas used.
    pub gas_used: u64,

    /// The vm status. If it is not `Executed`, this will provide the general error class. Execution
    /// failures and Move abort's receive more detailed information. But other errors are generally
    /// categorized with no status code or other information
    pub status: crate::vm_error::KeptVMStatus,
}

impl TransactionInfo {
    /// Constructs a new `TransactionInfo` object using transaction hash, state root hash and event
    /// root hash.
    pub fn new(
        transaction_hash: HashValue,
        state_root_hash: HashValue,
        events: &[ContractEvent],
        gas_used: u64,
        status: crate::vm_error::KeptVMStatus,
    ) -> TransactionInfo {
        let event_hashes: Vec<_> = events.iter().map(|e| e.crypto_hash()).collect();
        let events_accumulator_hash =
            InMemoryAccumulator::from_leaves(event_hashes.as_slice()).root_hash();
        TransactionInfo {
            transaction_hash,
            state_root_hash,
            event_root_hash: events_accumulator_hash,
            gas_used,
            status,
        }
    }

    pub fn id(&self) -> HashValue {
        self.crypto_hash()
    }

    /// Returns the hash of this transaction.
    pub fn transaction_hash(&self) -> HashValue {
        self.transaction_hash
    }

    /// Returns root hash of Sparse Merkle Tree describing the world state at the end of this
    /// transaction.
    pub fn state_root_hash(&self) -> HashValue {
        self.state_root_hash
    }

    /// Returns the root hash of Merkle Accumulator storing all events emitted during this
    /// transaction.
    pub fn event_root_hash(&self) -> HashValue {
        self.event_root_hash
    }

    /// Returns the amount of gas used by this transaction.
    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn status(&self) -> &crate::vm_error::KeptVMStatus {
        &self.status
    }
}

impl Sample for TransactionInfo {
    fn sample() -> Self {
        Self::new(
            SignedUserTransaction::sample().id(),
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
            &[],
            0,
            crate::vm_error::KeptVMStatus::Executed,
        )
    }
}

/// `RichTransactionInfo` is a wrapper of `TransactionInfo` with more info,
/// such as `block_id`, `block_number` which is the block that include the txn producing the txn info.
/// We cannot put the block_id into txn_info, because txn_info is accumulated into block header.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RichTransactionInfo {
    pub block_id: HashValue,
    pub block_number: u64,
    pub transaction_info: TransactionInfo,
    /// Transaction index in block
    pub transaction_index: u32,
    /// Transaction global index in chain, equivalent to transaction accumulator's leaf index
    pub transaction_global_index: u64,
}

impl Deref for RichTransactionInfo {
    type Target = TransactionInfo;

    fn deref(&self) -> &Self::Target {
        &self.transaction_info
    }
}

impl RichTransactionInfo {
    pub fn new(
        block_id: HashValue,
        block_number: u64,
        transaction_info: TransactionInfo,
        transaction_index: u32,
        transaction_global_index: u64,
    ) -> Self {
        Self {
            block_id,
            block_number,
            transaction_info,
            transaction_index,
            transaction_global_index,
        }
    }

    pub fn block_id(&self) -> HashValue {
        self.block_id
    }

    pub fn txn_info(&self) -> &TransactionInfo {
        &self.transaction_info
    }
}

fn lo_convert(lo: AbortLocation) -> starcoin_vm_types::vm_status::AbortLocation {
    match lo {
        AbortLocation::Script => starcoin_vm_types::vm_status::AbortLocation::Script,
        AbortLocation::Module(module_id) => {
            starcoin_vm_types::vm_status::AbortLocation::Module(ModuleId::new(
                module_id.address.into_bytes().into(),
                // todo: double check, this conversion should never fail.
                Identifier::from_utf8(module_id.name.into_bytes()).unwrap(),
            ))
        }
    }
}

impl From<TransactionInfoV2> for TransactionInfo {
    fn from(value: TransactionInfoV2) -> Self {
        use starcoin_vm_types::vm_status::KeptVMStatus::*;
        Self {
            transaction_hash: value.transaction_hash,
            state_root_hash: value.state_root_hash,
            event_root_hash: value.event_root_hash,
            gas_used: value.gas_used,
            status: match value.status {
                KeptVMStatus::Executed => Executed,
                KeptVMStatus::OutOfGas => OutOfGas,
                KeptVMStatus::MoveAbort(lo, code) => MoveAbort(lo_convert(lo), code),
                KeptVMStatus::ExecutionFailure {
                    location,
                    function,
                    code_offset,
                    message: _,
                } => ExecutionFailure {
                    location: lo_convert(location),
                    function,
                    code_offset,
                },
                KeptVMStatus::MiscellaneousError => MiscellaneousError,
            },
        }
    }
}
