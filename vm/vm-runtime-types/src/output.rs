// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::change_set::VMChangeSet;
use move_core_types::vm_status::{StatusCode, VMStatus};
use starcoin_vm_types::transaction::TransactionOutput;
use starcoin_vm_types::{fee_statement::FeeStatement, transaction::TransactionStatus};
use std::collections::BTreeMap;

/// Output produced by the VM after executing a transaction.
///
/// **WARNING**: This type should only be used inside the VM. For storage backends,
/// use `TransactionOutput`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VMOutput {
    change_set: VMChangeSet,
    fee_statement: FeeStatement,
    status: TransactionStatus,
}

impl VMOutput {
    pub fn new(
        change_set: VMChangeSet,
        fee_statement: FeeStatement,
        status: TransactionStatus,
    ) -> Self {
        Self {
            change_set,
            fee_statement,
            status,
        }
    }

    pub fn empty_with_status(status: TransactionStatus) -> Self {
        Self {
            change_set: VMChangeSet::empty(),
            fee_statement: FeeStatement::zero(),
            status,
        }
    }

    pub fn unpack(self) -> (VMChangeSet, u64, TransactionStatus) {
        (self.change_set, self.fee_statement.gas_used(), self.status)
    }

    pub fn unpack_with_fee_statement(self) -> (VMChangeSet, FeeStatement, TransactionStatus) {
        (self.change_set, self.fee_statement, self.status)
    }

    pub fn change_set(&self) -> &VMChangeSet {
        &self.change_set
    }

    pub fn change_set_mut(&mut self) -> &mut VMChangeSet {
        &mut self.change_set
    }

    pub fn gas_used(&self) -> u64 {
        self.fee_statement.gas_used()
    }

    pub fn fee_statement(&self) -> &FeeStatement {
        &self.fee_statement
    }

    pub fn status(&self) -> &TransactionStatus {
        &self.status
    }

    /// Constructs `TransactionOutput`
    pub fn into_transaction_output(self) -> anyhow::Result<TransactionOutput, VMStatus> {
        let (change_set, fee_statement, status) = self.unpack_with_fee_statement();
        let output = VMOutput::new(change_set, fee_statement, status);
        Self::convert_to_transaction_output(output).map_err(|e| {
            VMStatus::error(
                StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                Some(e.to_string()),
            )
        })
    }

    fn convert_to_transaction_output(output: VMOutput) -> anyhow::Result<TransactionOutput> {
        let (vm_change_set, gas_used, status) = output.unpack();
        let (write_set, events) = vm_change_set.try_into_storage_change_set()?.into_inner();
        Ok(TransactionOutput::new(
            BTreeMap::default(),
            write_set,
            events,
            gas_used,
            status,
        ))
    }
}
