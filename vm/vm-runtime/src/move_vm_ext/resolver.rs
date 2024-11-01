// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMError;
use move_core_types::resolver::MoveResolver;
use move_table_extension::TableResolver;
use starcoin_aggregator::resolver::{AggregatorV1Resolver, DelayedFieldResolver};
use starcoin_vm_runtime_types::resolver::ExecutorView;
use starcoin_vm_types::on_chain_config::ConfigStorage;

/// A general resolver used by StarcoinVM. Allows to implement custom hooks on
/// top of storage, e.g. get resources from resource groups, etc.
/// MoveResolver implements ResourceResolver and ModuleResolver
pub trait StarcoinMoveResolver:
    AggregatorV1Resolver
    + DelayedFieldResolver
    + ConfigStorage
    + MoveResolver<PartialVMError>
    + TableResolver
    + AsExecutorView {

}

impl<S: AggregatorV1Resolver + ConfigStorage + DelayedFieldResolver + MoveResolver<PartialVMError> + TableResolver + AsExecutorView>
StarcoinMoveResolver for S {

}

pub trait AsExecutorView {
    fn as_executor_view(&self) -> &dyn ExecutorView;
}
