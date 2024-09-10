// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::change_set::VMChangeSet;
use move_core_types::vm_status::{StatusCode, VMStatus};
use starcoin_vm_types::transaction::TransactionOutput;
use starcoin_vm_types::{fee_statement::FeeStatement, transaction::TransactionStatus};
use std::collections::BTreeMap;
use starcoin_aggregator::resolver::AggregatorV1Resolver;
use starcoin_aggregator::types::code_invariant_error;
use starcoin_vm_types::aggregator::PanicError;
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::write_set::WriteOp;

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

    /// Materializes delta sets.
    /// Guarantees that if deltas are materialized successfully, the output
    /// has an empty delta set.
    /// TODO[agg_v2](cleanup) Consolidate materialization paths. See either:
    /// - if we can/should move try_materialize_aggregator_v1_delta_set into
    ///   executor.rs
    /// - move all materialization (including delayed fields) into change_set
    pub fn try_materialize(
        &mut self,
        resolver: &impl AggregatorV1Resolver,
    ) -> anyhow::Result<(), VMStatus> {
        // First, check if output of transaction should be discarded or delta
        // change set is empty. In both cases, we do not need to apply any
        // deltas and can return immediately.
        if self.status().is_discarded()
            || (self.change_set().aggregator_v1_delta_set().is_empty()
            && self.change_set().delayed_field_change_set().is_empty())
        {
            return Ok(());
        }

        self.change_set
            .try_materialize_aggregator_v1_delta_set(resolver)?;

        Ok(())
    }

    /// Same as `try_materialize` but also constructs `TransactionOutput`.
    pub fn try_materialize_into_transaction_output(
        mut self,
        resolver: &impl AggregatorV1Resolver,
    ) -> anyhow::Result<TransactionOutput, VMStatus> {
        self.try_materialize(resolver)?;
        Self::convert_to_transaction_output(self).map_err(|e| {
            VMStatus::error(
                StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                Some(e.to_string()),
            )
        })
    }

    /// Constructs `TransactionOutput`, without doing `try_materialize`
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

    fn convert_to_transaction_output(
        materialized_output: VMOutput,
    ) -> Result<TransactionOutput, PanicError> {
        let (vm_change_set, gas_used, status) = materialized_output.unpack();
        let (write_set, events) = vm_change_set.try_into_storage_change_set()?.into_inner();
        Ok(TransactionOutput::new(BTreeMap::default(),write_set, events, gas_used, status))
    }

    /// Updates the VMChangeSet based on the input aggregator v1 deltas, patched resource write set,
    /// patched events, and generates TransactionOutput
    pub fn into_transaction_output_with_materialized_write_set(
        mut self,
        materialized_aggregator_v1_deltas: Vec<(StateKey, WriteOp)>,
        patched_resource_write_set: Vec<(StateKey, WriteOp)>,
        patched_events: Vec<ContractEvent>,
    ) -> Result<TransactionOutput, PanicError> {
        // materialize aggregator V1 deltas into writes
        if materialized_aggregator_v1_deltas.len()
            != self.change_set().aggregator_v1_delta_set().len()
        {
            return Err(code_invariant_error(
                "Different number of materialized deltas and deltas in the output.",
            ));
        }
        if !materialized_aggregator_v1_deltas
            .iter()
            .all(|(k, _)| self.change_set().aggregator_v1_delta_set().contains_key(k))
        {
            return Err(code_invariant_error(
                "Materialized aggregator writes contain a key which does not exist in delta set.",
            ));
        }
        self.change_set
            .extend_aggregator_v1_write_set(materialized_aggregator_v1_deltas.into_iter());
        // TODO[agg_v2](cleanup) move all drains to happen when getting what to materialize.
        let _ = self.change_set.drain_aggregator_v1_delta_set();

        // materialize delayed fields into resource writes
        self.change_set
            .extend_resource_write_set(patched_resource_write_set.into_iter())?;
        let _ = self.change_set.drain_delayed_field_change_set();

        // materialize delayed fields into events
        if patched_events.len() != self.change_set().events().len() {
            return Err(code_invariant_error(
                "Different number of events and patched events in the output.",
            ));
        }
        self.change_set.set_events(patched_events.into_iter());

        Self::convert_to_transaction_output(self)
    }
}
