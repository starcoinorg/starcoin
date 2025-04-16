// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
pub use starcoin_vm_types::transaction::*;
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

use crate::{identifier::Identifier, language_storage::ModuleId};
use starcoin_vm2_types::transaction::RichTransactionInfo as RichTransactionInfoV2;
use starcoin_vm2_types::vm_error::{AbortLocation, KeptVMStatus};

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

impl From<RichTransactionInfoV2> for RichTransactionInfo {
    fn from(value: RichTransactionInfoV2) -> Self {
        Self {
            block_id: value.block_id,
            block_number: value.block_number,
            transaction_info: TransactionInfo {
                transaction_hash: value.transaction_info.transaction_hash,
                state_root_hash: value.transaction_info.state_root_hash,
                event_root_hash: value.transaction_info.event_root_hash,
                gas_used: value.transaction_info.gas_used,
                status: match value.transaction_info.status {
                    KeptVMStatus::Executed => starcoin_vm_types::vm_status::KeptVMStatus::Executed,
                    KeptVMStatus::OutOfGas => starcoin_vm_types::vm_status::KeptVMStatus::OutOfGas,
                    KeptVMStatus::MoveAbort(Lo, code) => {
                        starcoin_vm_types::vm_status::KeptVMStatus::MoveAbort(lo_convert(Lo), code)
                    }
                    KeptVMStatus::ExecutionFailure {
                        location,
                        function,
                        code_offset,
                        message: _,
                    } => starcoin_vm_types::vm_status::KeptVMStatus::ExecutionFailure {
                        location: lo_convert(location),
                        function,
                        code_offset,
                    },
                    KeptVMStatus::MiscellaneousError => {
                        starcoin_vm_types::vm_status::KeptVMStatus::MiscellaneousError
                    }
                },
            },
            transaction_index: value.transaction_index,
            transaction_global_index: value.transaction_global_index,
        }
    }
}
