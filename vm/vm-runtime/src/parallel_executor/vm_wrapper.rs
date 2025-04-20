// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    adapter_common::{PreprocessedTransaction, VMAdapter},
    parallel_executor::{storage_wrapper::VersionedView, StarcoinTransactionOutput},
    starcoin_vm::StarcoinVM,
};

use starcoin_parallel_executor::{
    executor::MVHashMapView,
    task::{ExecutionStatus, ExecutorTask},
};

use move_core_types::vm_status::VMStatus;
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

    fn init(argument: &'a S) -> Self {
        let mut vm = StarcoinVM::new(None);
        vm.load_configs(argument)
            .expect("load configs should always success");

        Self {
            vm,
            base_view: argument,
        }
    }

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
