// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Scratchpad for on chain values during the execution.

use crate::move_vm_ext::{resource_state_key, AsExecutorView, ResourceGroupResolver};
use bytes::Bytes;
use move_binary_format::deserializer::DeserializerConfig;
use move_binary_format::CompiledModule;
use move_bytecode_utils::compiled_module_viewer::CompiledModuleView;
use move_core_types::metadata::Metadata;
use move_core_types::resolver::{resource_size, ModuleResolver, ResourceResolver};
use move_core_types::value::MoveTypeLayout;
use move_table_extension::{TableHandle, TableResolver};
use starcoin_gas_schedule::LATEST_GAS_FEATURE_VERSION;
use starcoin_logger::prelude::*;
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_runtime_types::resolver::{
    ExecutorView, ResourceGroupSize, TResourceGroupView, TResourceView,
};
use starcoin_vm_runtime_types::resource_group_adapter::ResourceGroupAdapter;
use starcoin_vm_types::state_store::{
    errors::StateviewError, state_key::StateKey, state_storage_usage::StateStorageUsage,
    state_value::StateValue, StateView, TStateView,
};
use starcoin_vm_types::{
    errors::*,
    language_storage::{ModuleId, StructTag},
    vm_status::StatusCode,
    write_set::{WriteOp, WriteSet},
};
use std::collections::HashMap;
use std::{
    cell::RefCell,
    collections::btree_map::BTreeMap,
    collections::HashSet,
    ops::{Deref, DerefMut},
};

pub fn get_resource_group_member_from_metadata(
    struct_tag: &StructTag,
    metadata: &[Metadata],
) -> Option<StructTag> {
    eprintln!(
        "get_resource_group_member_from_metadata {} origin metadata count {}",
        struct_tag,
        metadata.len()
    );
    if metadata.is_empty() && struct_tag.name.as_ident_str().as_str() == "ObjectCore" {
        panic!("let's see the backtrace");
    }
    let metadata = starcoin_framework::get_metadata(metadata)?;
    eprintln!(
        "get_resource_group_member_from_metadata {} metadata struct_attributes {:?}",
        struct_tag, metadata.struct_attributes
    );
    metadata
        .struct_attributes
        .get(struct_tag.name.as_ident_str().as_str())?
        .iter()
        .find_map(|attr| attr.get_resource_group_member())
}

/// Adapter to convert a `ExecutorView` into a `AptosMoveResolver`.
///
/// Resources in groups are handled either through dedicated interfaces of executor_view
/// (that tie to specialized handling in block executor), or via 'standard' interfaces
/// for (non-group) resources and subsequent handling in the StorageAdapter itself.
pub struct StorageAdapter<'e, E> {
    executor_view: &'e E,
    _deserializer_config: DeserializerConfig,
    resource_group_view: ResourceGroupAdapter<'e>,
    accessed_groups: RefCell<HashSet<StateKey>>,
}

/// A local cache for a given a `StateView`. The cache is private to the Diem layer
/// but can be used as a one shot cache for systems that need a simple `RemoteCache`
/// implementation (e.g. tests or benchmarks).
///
/// The cache is responsible to track all changes to the `StateView` that are the result
/// of transaction execution. Those side effects are published at the end of a transaction
/// execution via `StateViewCache::push_write_set`.
///
/// `StateViewCache` is responsible to give an up to date view over the data store,
/// so that changes executed but not yet committed are visible to subsequent transactions.
///
/// If a system wishes to execute a block of transaction on a given view, a cache that keeps
/// track of incremental changes is vital to the consistency of the data store and the system.
pub struct StateViewCache<'a, S> {
    data_view: &'a S,
    data_map: BTreeMap<StateKey, WriteOp>,
}

impl<'a, S: StateView> StateViewCache<'a, S> {
    /// Create a `StateViewCache` give a `StateView`. Hold updates to the data store and
    /// forward data request to the `StateView` if not in the local cache.
    pub fn new(data_view: &'a S) -> Self {
        StateViewCache {
            data_view,
            data_map: BTreeMap::new(),
        }
    }

    // Publishes a `WriteSet` computed at the end of a transaction.
    // The effect is to build a layer in front of the `StateView` which keeps
    // track of the data as if the changes were applied immediately.
    pub(crate) fn push_write_set(&mut self, write_set: &WriteSet) -> Result<(), StateviewError> {
        for (key, write_op) in write_set.iter() {
            use std::collections::btree_map::Entry::*;
            match self.data_map.entry(key.clone()) {
                Vacant(entry) => {
                    entry.insert(write_op.clone());
                }
                Occupied(mut entry) => {
                    if !WriteOp::squash(entry.get_mut(), write_op.clone())? {
                        entry.remove();
                    }
                }
            }
        }
        Ok(())
    }
}

impl<'block, S: StateView> TStateView for StateViewCache<'block, S> {
    type Key = StateKey;

    // Get some data either through the cache or the `StateView` on a cache miss.
    fn get_state_value(&self, state_key: &Self::Key) -> Result<Option<StateValue>, StateviewError> {
        match self.data_map.get(state_key) {
            Some(opt_data) => Ok(opt_data.bytes().map(|bytes| {
                StateValue::new_with_metadata(bytes.clone(), opt_data.metadata().clone())
            })),
            None => match self.data_view.get_state_value(state_key) {
                Ok(remote_data) => Ok(remote_data),
                // TODO: should we forward some error info?
                Err(e) => {
                    error!("[VM] Error getting data from storage for {:?}", state_key);
                    Err(e)
                }
            },
        }
    }

    fn get_usage(&self) -> Result<StateStorageUsage, StateviewError> {
        todo!()
    }

    fn is_genesis(&self) -> bool {
        self.data_view.is_genesis()
    }
}

impl<'block, S: StateView> ModuleResolver for StateViewCache<'block, S> {
    type Error = PartialVMError;

    fn get_module_metadata(&self, module_id: &ModuleId) -> Vec<Metadata> {
        StorageAdapter::new(self).get_module_metadata(module_id)
    }

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Bytes>, Self::Error> {
        StorageAdapter::new(self).get_module(module_id)
    }
}
impl<'block, S: StateView> ResourceResolver for StateViewCache<'block, S> {
    type Error = PartialVMError;

    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
        metadata: &[Metadata],
        layout: Option<&MoveTypeLayout>,
    ) -> Result<(Option<Bytes>, usize), Self::Error> {
        StorageAdapter::new(self)
            .get_resource_bytes_with_metadata_and_layout(address, struct_tag, metadata, layout)
    }
}

impl<'a, S: StateView> StorageAdapter<'a, S> {
    pub fn new(state_store: &'a S) -> Self {
        Self {
            executor_view: state_store,
            _deserializer_config: DeserializerConfig::new(0, 0),
            resource_group_view: ResourceGroupAdapter::new(
                None,
                state_store,
                LATEST_GAS_FEATURE_VERSION,
                false,
            ),
            accessed_groups: RefCell::new(HashSet::new()),
        }
    }

    pub fn get(&self, key: &StateKey) -> Result<Option<StateValue>, PartialVMError> {
        self.executor_view
            .get_state_value(key)
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR))
    }
}

impl<'a, S: StateView> ModuleResolver for StorageAdapter<'a, S> {
    type Error = PartialVMError;

    fn get_module_metadata(&self, module_id: &ModuleId) -> Vec<Metadata> {
        let module = match self.get_module(module_id) {
            Ok(Some(module)) => module,
            _ => return vec![],
        };

        let compiled_module = match CompiledModule::deserialize(&module) {
            Ok(module) => module,
            _ => return vec![],
        };

        compiled_module.metadata
    }

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Bytes>, Self::Error> {
        // REVIEW: cache this?
        let key = StateKey::module_id(module_id);
        self.get(&key).map(|r| r.map(|v| v.bytes().clone()))
    }
}
impl<'a, S: StateView> ResourceResolver for StorageAdapter<'a, S> {
    type Error = PartialVMError;

    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
        metadata: &[Metadata],
        maybe_layout: Option<&MoveTypeLayout>,
    ) -> Result<(Option<Bytes>, usize), Self::Error> {
        let resource_group = get_resource_group_member_from_metadata(struct_tag, metadata);
        if let Some(resource_group) = resource_group {
            eprintln!(
                "get_resource_bytes_with_metadata_and_layout {} from group",
                struct_tag
            );
            let key = StateKey::resource_group(address, &resource_group);
            let buf =
                self.resource_group_view
                    .get_resource_from_group(&key, struct_tag, maybe_layout)?;

            let first_access = self.accessed_groups.borrow_mut().insert(key.clone());
            let group_size = if first_access {
                self.resource_group_view.resource_group_size(&key)?.get()
            } else {
                0
            };

            let buf_size = resource_size(&buf);
            Ok((buf, buf_size + group_size as usize))
        } else {
            eprintln!(
                "get_resource_bytes_with_metadata_and_layout {} from executor_view",
                struct_tag
            );
            let state_key = resource_state_key(address, struct_tag)?;
            let buf = self
                .executor_view
                .get_resource_bytes(&state_key, maybe_layout)?;
            let buf_size = resource_size(&buf);
            Ok((buf, buf_size))
        }
    }
}

impl<'a, S: StateView> ResourceGroupResolver for StorageAdapter<'a, S> {
    fn release_resource_group_cache(
        &self,
    ) -> Option<HashMap<StateKey, BTreeMap<StructTag, Bytes>>> {
        self.resource_group_view.release_group_cache()
    }

    fn resource_group_size(&self, group_key: &StateKey) -> PartialVMResult<ResourceGroupSize> {
        self.resource_group_view.resource_group_size(group_key)
    }

    fn resource_size_in_group(
        &self,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> PartialVMResult<usize> {
        self.resource_group_view
            .resource_size_in_group(group_key, resource_tag)
    }

    fn resource_exists_in_group(
        &self,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> PartialVMResult<bool> {
        self.resource_group_view
            .resource_exists_in_group(group_key, resource_tag)
    }
}

impl<'a, S> Deref for StorageAdapter<'a, S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.executor_view
    }
}

impl<'a, S: StateView> TableResolver for StorageAdapter<'a, S> {
    // TODO(simon): don't ignore maybe_layout
    fn resolve_table_entry_bytes_with_layout(
        &self,
        handle: &TableHandle,
        key: &[u8],
        _maybe_layout: Option<&MoveTypeLayout>,
    ) -> Result<Option<Bytes>, PartialVMError> {
        self.executor_view
            .get_state_value(&StateKey::table_item(&(*handle).into(), key))
            .map(|r| r.map(|v| v.bytes().clone()))
            .map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR).with_message(format!("{:?}", e))
            })
    }
}
impl<'a, S: StateView> CompiledModuleView for StorageAdapter<'a, S> {
    type Item = CompiledModule;
    fn view_compiled_module(&self, id: &ModuleId) -> anyhow::Result<Option<Self::Item>> {
        let module = match self.get_module(id) {
            Ok(Some(module)) => module,
            _ => return Ok(None),
        };
        Ok(Some(CompiledModule::deserialize(&module)?))
    }
}

pub trait AsMoveResolver<S> {
    fn as_move_resolver(&self) -> StorageAdapter<S>;
}

impl<S: StateView> AsMoveResolver<S> for S {
    fn as_move_resolver(&self) -> StorageAdapter<S> {
        StorageAdapter::new(self)
    }
}

impl<'a, S: StateView> AsExecutorView for StorageAdapter<'a, S> {
    fn as_executor_view(&self) -> &dyn ExecutorView {
        self.executor_view
    }
}

pub struct RemoteStorageOwned<S> {
    state_view: S,
}

impl<S> Deref for RemoteStorageOwned<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.state_view
    }
}

impl<S> DerefMut for RemoteStorageOwned<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state_view
    }
}

impl<S: StateView> ModuleResolver for RemoteStorageOwned<S> {
    type Error = PartialVMError;

    fn get_module_metadata(&self, module_id: &ModuleId) -> Vec<Metadata> {
        self.as_move_resolver().get_module_metadata(module_id)
    }

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Bytes>, Self::Error> {
        self.as_move_resolver().get_module(module_id)
    }
}

impl<S: StateView> ResourceResolver for RemoteStorageOwned<S> {
    type Error = PartialVMError;

    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
        metadata: &[Metadata],
        layout: Option<&MoveTypeLayout>,
    ) -> Result<(Option<Bytes>, usize), Self::Error> {
        self.as_move_resolver()
            .get_resource_bytes_with_metadata_and_layout(address, struct_tag, metadata, layout)
    }
}

impl<S: StateView> ResourceGroupResolver for RemoteStorageOwned<S> {
    fn release_resource_group_cache(
        &self,
    ) -> Option<HashMap<StateKey, BTreeMap<StructTag, Bytes>>> {
        self.as_move_resolver().release_resource_group_cache()
    }

    fn resource_group_size(&self, group_key: &StateKey) -> PartialVMResult<ResourceGroupSize> {
        self.as_move_resolver().resource_group_size(group_key)
    }

    fn resource_size_in_group(
        &self,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> PartialVMResult<usize> {
        self.as_move_resolver()
            .resource_size_in_group(group_key, resource_tag)
    }

    fn resource_exists_in_group(
        &self,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> PartialVMResult<bool> {
        self.as_move_resolver()
            .resource_exists_in_group(group_key, resource_tag)
    }
}

impl<S: StateView> TableResolver for RemoteStorageOwned<S> {
    fn resolve_table_entry_bytes_with_layout(
        &self,
        handle: &TableHandle,
        key: &[u8],
        maybe_layout: Option<&MoveTypeLayout>,
    ) -> Result<Option<Bytes>, PartialVMError> {
        self.as_move_resolver()
            .resolve_table_entry_bytes_with_layout(handle, key, maybe_layout)
    }
}

impl<S: StateView> AsExecutorView for RemoteStorageOwned<S> {
    fn as_executor_view(&self) -> &dyn ExecutorView {
        &self.state_view
    }
}

pub trait IntoMoveResolver<S> {
    fn into_move_resolver(self) -> RemoteStorageOwned<S>;
}

impl<S: StateView> IntoMoveResolver<S> for S {
    fn into_move_resolver(self) -> RemoteStorageOwned<S> {
        RemoteStorageOwned { state_view: self }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use starcoin_vm_runtime_types::resource_group_adapter::GroupSizeKind;
    //use starcoin_vm_types::on_chain_config::{Features, OnChainConfig};

    // Expose a method to create a storage adapter with a provided group size kind.
    #[allow(dead_code)]
    pub(crate) fn as_resolver_with_group_size_kind<S: StateView>(
        state_view: &S,
        group_size_kind: GroupSizeKind,
    ) -> StorageAdapter<S> {
        assert_ne!(group_size_kind, GroupSizeKind::AsSum, "not yet supported");

        let (gas_feature_version, resource_groups_split_in_vm_change_set_enabled) =
            match group_size_kind {
                GroupSizeKind::AsSum => (12, true),
                GroupSizeKind::AsBlob => (10, false),
                GroupSizeKind::None => (1, false),
            };

        let _group_adapter = ResourceGroupAdapter::new(
            // TODO[agg_v2](test) add a converter for StateView for tests that implements ResourceGroupView
            None,
            state_view,
            gas_feature_version,
            resource_groups_split_in_vm_change_set_enabled,
        );

        // let features = Features::fetch_config(state_view).unwrap_or_default();
        //  let deserializer_config = aptos_prod_deserializer_config(&features);
        StorageAdapter::new(state_view)
    }
}
