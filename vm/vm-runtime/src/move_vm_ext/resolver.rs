// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, HashMap};

use bytes::Bytes;
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{language_storage::StructTag, resolver::MoveResolver};
use move_table_extension::TableResolver;

use starcoin_aggregator::resolver::{AggregatorV1Resolver, DelayedFieldResolver};
use starcoin_vm_runtime_types::resolver::{ExecutorView, ResourceGroupSize, ResourceGroupView};
use starcoin_vm_types::{
    on_chain_config::ConfigStorage,
    state_store::state_key::StateKey
};

/// A general resolver used by StarcoinVM. Allows to implement custom hooks on
/// top of storage, e.g. get resources from resource groups, etc.
/// MoveResolver implements ResourceResolver and ModuleResolver
pub trait StarcoinMoveResolver:
    AggregatorV1Resolver
    + DelayedFieldResolver
    + ConfigStorage
    + MoveResolver<PartialVMError>
    + ResourceGroupResolver
    + TableResolver
    + AsExecutorView
    + AsResourceGroupView
{
}

impl<
        S: AggregatorV1Resolver
            + ConfigStorage
            + DelayedFieldResolver
            + MoveResolver<PartialVMError>
            + ResourceGroupResolver
            + TableResolver
            + AsExecutorView
            + AsResourceGroupView
    > StarcoinMoveResolver for S
{
}

pub trait ResourceGroupResolver {
    fn release_resource_group_cache(&self) -> Option<HashMap<StateKey, BTreeMap<StructTag, Bytes>>>;

    fn resource_group_size(&self, group_key: &StateKey) -> PartialVMResult<ResourceGroupSize>;

    fn resource_size_in_group(
        &self,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> PartialVMResult<usize>;

    fn resource_exists_in_group(
        &self,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> PartialVMResult<bool>;
}


pub trait AsExecutorView {
    fn as_executor_view(&self) -> &dyn ExecutorView;
}

pub trait AsResourceGroupView {
    fn as_resource_group_view(&self) -> &dyn ResourceGroupView;
}
