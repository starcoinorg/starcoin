// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    adapter_common::{PreprocessedTransaction, VMAdapter},
    data_cache::RemoteStorage,
    parallel_executor::{storage_wrapper::VersionedView, StarcoinTransactionOutput},
    starcoin_vm::StarcoinVM,
};

use starcoin_parallel_executor::{
    executor::MVHashMapView,
    task::{ExecutionStatus, ExecutorTask},
};

use move_core_types::{
    ident_str,
    language_storage::{ModuleId, CORE_CODE_ADDRESS},
    vm_status::VMStatus,
};
use starcoin_logger::prelude::*;
use starcoin_vm_types::{
    state_store::state_key::StateKey, state_view::StateView, write_set::WriteOp,
};

pub(crate) struct StarcoinVMWrapper<'a, S> {
    vm: StarcoinVM,
    base_view: &'a S,
}

impl<'a, S: 'a + StateView> ExecutorTask for StarcoinVMWrapper<'a, S> {
    type T = PreprocessedTransaction;
    type Output = StarcoinTransactionOutput;
    type Error = VMStatus;
    type Argument = &'a S;

    // XXX FIXME YSG
    fn init(argument: &'a S) -> Self {
        // XXX FIXME YSG
        let vm = StarcoinVM::new(None);

        // XXX FIXME YSG
        // Loading `0x1::Account` and its transitive dependency into the code cache.
        //
        // This should give us a warm VM to avoid the overhead of VM cold start.
        // Result of this load could be omitted as this is a best effort approach and won't hurt if that fails.
        //
        // Loading up `0x1::AptosAccount` should be sufficient as this is the most common module
        // used for prologue, epilogue and transfer functionality.

        // XXX FIXME YSG
        let _ = vm.load_module(
            &ModuleId::new(CORE_CODE_ADDRESS, ident_str!("Account").to_owned()),
            &RemoteStorage::new(argument),
        );

        Self {
            vm,
            base_view: argument,
        }
    }

    // XXX FIXME YSG, self should be immut
    fn execute_transaction(
        &self,
        view: &MVHashMapView<StateKey, WriteOp>,
        txn: &PreprocessedTransaction,
    ) -> ExecutionStatus<StarcoinTransactionOutput, VMStatus> {
        let versioned_view = VersionedView::new_view(self.base_view, view);

        match self.vm.execute_single_transaction(txn, &versioned_view) {
            Ok((vm_status, output, sender)) => {
                if output.status().is_discarded() {
                    match sender {
                        Some(s) => trace!(
                            "Transaction discarded, sender: {}, error: {:?}",
                            s,
                            vm_status,
                        ),
                        None => {
                            trace!("Transaction malformed, error: {:?}", vm_status,)
                        }
                    };
                }
                // XXX FIXME YSG
                if StarcoinVM::should_restart_execution(&output) {
                    ExecutionStatus::SkipRest(StarcoinTransactionOutput::new(output))
                } else {
                    ExecutionStatus::Success(StarcoinTransactionOutput::new(output))
                }
            }
            Err(err) => ExecutionStatus::Abort(err),
        }
    }
}
