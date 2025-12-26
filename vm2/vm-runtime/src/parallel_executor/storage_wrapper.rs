// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_vm_ext::{AsExecutorView, ResourceGroupResolver};
use bytes::Bytes;
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::{ModuleId, StructTag};
use move_core_types::metadata::Metadata;
use move_core_types::resolver::{ModuleResolver, ResourceResolver};
use move_core_types::value::MoveTypeLayout;
use move_table_extension::{TableHandle, TableResolver};
use starcoin_parallel_executor::executor::MVHashMapView;
use starcoin_vm_runtime_types::resolver::{ExecutorView, ResourceGroupSize, TResourceGroupView};
use starcoin_vm_types::state_store::{
    errors::StateviewError, state_key::StateKey, state_storage_usage::StateStorageUsage,
    state_value::StateValue, StateView, TStateView,
};
use starcoin_vm_types::write_set::WriteOp;
use std::collections::{BTreeMap, HashMap};

pub(crate) struct VersionedView<'a, S: StateView> {
    base_view: &'a S,
    hashmap_view: &'a MVHashMapView<'a, StateKey, WriteOp>,
}

impl<'a, S: StateView> VersionedView<'a, S> {
    pub fn new_view(
        base_view: &'a S,
        hashmap_view: &'a MVHashMapView<'a, StateKey, WriteOp>,
    ) -> VersionedView<'a, S> {
        VersionedView {
            base_view,
            hashmap_view,
        }
    }
}

impl<S: StateView> TStateView for VersionedView<'_, S> {
    type Key = StateKey;

    // Get some data either through the cache or the `StateView` on a cache miss.
    fn get_state_value(&self, state_key: &Self::Key) -> Result<Option<StateValue>, StateviewError> {
        match self.hashmap_view.read(state_key) {
            Some(v) => Ok(v
                .bytes()
                .map(|bytes| StateValue::new_with_metadata(bytes.clone(), v.metadata().clone()))),
            None => self.base_view.get_state_value(state_key),
        }
    }

    fn get_usage(&self) -> Result<StateStorageUsage, StateviewError> {
        unimplemented!("get_usage not implemented for VersionedView")
    }

    fn is_genesis(&self) -> bool {
        self.base_view.is_genesis()
    }
}

impl<S: StateView> TResourceGroupView for VersionedView<'_, S> {
    type GroupKey = StateKey;
    type ResourceTag = StructTag;
    type Layout = MoveTypeLayout;

    fn resource_group_size(&self, _group_key: &StateKey) -> PartialVMResult<ResourceGroupSize> {
        unimplemented!("Currently resolved by ResourceGroupAdapter");
    }

    fn get_resource_from_group(
        &self,
        group_key: &Self::GroupKey,
        resource_tag: &Self::ResourceTag,
        maybe_layout: Option<&Self::Layout>,
    ) -> PartialVMResult<Option<Bytes>> {
        todo!()
    }

    fn resource_size_in_group(
        &self,
        _group_key: &StateKey,
        _resource_tag: &StructTag,
    ) -> PartialVMResult<usize> {
        unimplemented!("Currently resolved by ResourceGroupAdapter");
    }

    fn resource_exists_in_group(
        &self,
        _group_key: &StateKey,
        _resource_tag: &StructTag,
    ) -> PartialVMResult<bool> {
        unimplemented!("Currently resolved by ResourceGroupAdapter");
    }

    fn release_group_cache(
        &self,
    ) -> Option<HashMap<Self::GroupKey, BTreeMap<Self::ResourceTag, Bytes>>> {
        unimplemented!("Currently resolved by ResourceGroupAdapter");
    }
}

impl<S: StateView> AsExecutorView for VersionedView<'_, S> {
    fn as_executor_view(&self) -> &dyn ExecutorView {
        todo!()
    }
}

impl<S: StateView> TableResolver for VersionedView<'_, S> {
    fn resolve_table_entry_bytes_with_layout(
        &self,
        handle: &TableHandle,
        key: &[u8],
        maybe_layout: Option<&MoveTypeLayout>,
    ) -> Result<Option<Bytes>, PartialVMError> {
        todo!()
    }
}

impl<S: StateView> ModuleResolver for VersionedView<'_, S> {
    type Error = PartialVMError;

    fn get_module_metadata(&self, module_id: &ModuleId) -> Vec<Metadata> {
        todo!()
    }

    fn get_module(&self, id: &ModuleId) -> Result<Option<Bytes>, Self::Error> {
        todo!()
    }
}

impl<S: StateView> ResourceResolver for VersionedView<'_, S> {
    type Error = PartialVMError;

    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
        metadata: &[Metadata],
        layout: Option<&MoveTypeLayout>,
    ) -> Result<(Option<Bytes>, usize), Self::Error> {
        todo!()
    }
}

// todo(simon): Is it necessary?
impl<S: StateView> ResourceGroupResolver for VersionedView<'_, S> {
    fn release_resource_group_cache(
        &self,
    ) -> Option<HashMap<StateKey, BTreeMap<StructTag, Bytes>>> {
        todo!()
    }

    fn resource_group_size(&self, _group_key: &StateKey) -> PartialVMResult<ResourceGroupSize> {
        todo!()
    }

    fn resource_size_in_group(
        &self,
        _group_key: &StateKey,
        _resource_tag: &StructTag,
    ) -> PartialVMResult<usize> {
        todo!()
    }

    fn resource_exists_in_group(
        &self,
        _group_key: &StateKey,
        _resource_tag: &StructTag,
    ) -> PartialVMResult<bool> {
        todo!()
    }
}
