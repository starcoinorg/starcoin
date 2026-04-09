// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    data_cache::effective_max_value_nest_depth,
    parallel_executor::{
        storage_wrapper::{DelayedFieldCache, VersionedView},
        StarcoinTransactionOutput,
    },
    starcoin_vm::StarcoinVM,
    PreprocessedTransaction,
};

use starcoin_parallel_executor::{
    executor::MVHashMapView,
    task::{ExecutionStatus, ExecutorTask},
};

use move_core_types::vm_status::VMStatus;
use starcoin_logger::prelude::*;
use starcoin_vm_types::on_chain_config::OnChainConfig;
use starcoin_vm_types::state_store::StateView;
use std::sync::Arc;

pub(crate) struct StarcoinVMWrapper<'a, S> {
    vm: StarcoinVM,
    base_view: &'a S,
    delayed_field_cache: Arc<DelayedFieldCache>,
    delayed_fields_enabled: bool,
    max_value_nest_depth: Option<u64>,
}

impl<'a, S: 'a + StateView + Sync> ExecutorTask for StarcoinVMWrapper<'a, S> {
    type T = PreprocessedTransaction;
    type Output = StarcoinTransactionOutput;
    type Error = VMStatus;
    type Argument = (&'a S, Arc<DelayedFieldCache>, Option<u64>);

    fn init(argument: Self::Argument) -> Self {
        let (base_view, delayed_field_cache, max_value_nest_depth) = argument;
        let mut vm = StarcoinVM::new(None, base_view);
        vm.load_configs(base_view)
            .expect("load configs should always success");
        let features = starcoin_vm_types::on_chain_config::Features::fetch_config(base_view)
            .unwrap_or_default();
        let delayed_fields_enabled = features.is_aggregator_v2_delayed_fields_enabled();
        let max_value_nest_depth =
            max_value_nest_depth.or_else(|| effective_max_value_nest_depth(base_view));

        Self {
            vm,
            base_view,
            delayed_field_cache,
            delayed_fields_enabled,
            max_value_nest_depth,
        }
    }

    fn execute_transaction(
        &self,
        view: &MVHashMapView<super::ParallelStateKey, super::ParallelStateValue>,
        txn: &PreprocessedTransaction,
    ) -> ExecutionStatus<StarcoinTransactionOutput, VMStatus> {
        let versioned_view = VersionedView::new(
            self.base_view,
            view,
            self.delayed_field_cache.clone(),
            self.delayed_fields_enabled,
            self.max_value_nest_depth,
        );
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
                let group_read_layouts = versioned_view.take_group_read_layouts();
                if StarcoinVM::should_restart_execution(&output) {
                    ExecutionStatus::SkipRest(StarcoinTransactionOutput::new(
                        output,
                        group_read_layouts,
                    ))
                } else {
                    ExecutionStatus::Success(StarcoinTransactionOutput::new(
                        output,
                        group_read_layouts,
                    ))
                }
            }
            Err(err) => ExecutionStatus::Abort(err),
        }
    }
}
