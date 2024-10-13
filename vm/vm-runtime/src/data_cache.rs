// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Scratchpad for on chain values during the execution.

use crate::move_vm_ext::AsExecutorView;
use bytes::Bytes;
use move_binary_format::deserializer::DeserializerConfig;
use move_binary_format::CompiledModule;
use move_core_types::metadata::Metadata;
use move_core_types::resolver::{resource_size, ModuleResolver, ResourceResolver};
use move_core_types::value::MoveTypeLayout;
use move_table_extension::{TableHandle, TableResolver};
use starcoin_logger::prelude::*;
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_runtime_types::resolver::ExecutorView;
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
use std::{
    cell::RefCell,
    collections::btree_map::BTreeMap,
    collections::HashSet,
    ops::{Deref, DerefMut},
};

pub(crate) fn get_resource_group_from_metadata(
    struct_tag: &StructTag,
    metadata: &[Metadata],
) -> Option<StructTag> {
    let metadata = starcoin_framework::get_metadata(metadata)?;
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
    deserializer_config: DeserializerConfig,
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
    data_map: BTreeMap<StateKey, Option<Vec<u8>>>,
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
    pub(crate) fn push_write_set(&mut self, write_set: &WriteSet) {
        for (ref ap, ref write_op) in write_set.iter() {
            // todo: handle WriteOp properly
            match write_op {
                WriteOp::Creation { data, metadata: _ } => {
                    self.data_map.insert((*ap).clone(), Some(data.to_vec()));
                }
                WriteOp::Modification { data, metadata: _ } => {
                    self.data_map.insert((*ap).clone(), Some(data.to_vec()));
                }
                WriteOp::Deletion { metadata: _ } => {
                    self.data_map.remove(ap);
                    self.data_map.insert((*ap).clone(), None);
                }
            }
        }
    }
}

impl<'block, S: StateView> TStateView for StateViewCache<'block, S> {
    type Key = StateKey;

    // Get some data either through the cache or the `StateView` on a cache miss.
    fn get_state_value(&self, state_key: &Self::Key) -> Result<Option<StateValue>, StateviewError> {
        match self.data_map.get(state_key) {
            Some(opt_data) => Ok(opt_data.clone().map(|v| StateValue::from(v))),
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
        RemoteStorage::new(self).get_module_metadata(module_id)
    }

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Bytes>, Self::Error> {
        RemoteStorage::new(self).get_module(module_id)
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
        RemoteStorage::new(self)
            .get_resource_bytes_with_metadata_and_layout(address, struct_tag, metadata, layout)
    }
}

// Adapter to convert a `StateView` into a `RemoteCache`.
pub struct RemoteStorage<'a, S>(&'a S);

impl<'a, S: StateView> RemoteStorage<'a, S> {
    pub fn new(state_store: &'a S) -> Self {
        Self(state_store)
    }

    pub fn get(&self, key: &StateKey) -> Result<Option<StateValue>, PartialVMError> {
        self.0
            .get_state_value(key)
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR))
    }
}

impl<'a, S: StateView> ModuleResolver for RemoteStorage<'a, S> {
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
impl<'a, S: StateView> ResourceResolver for RemoteStorage<'a, S> {
    type Error = PartialVMError;

    // TODO(simon): don't ignore metadata and layout
    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
        _metadata: &[Metadata],
        _layout: Option<&MoveTypeLayout>,
    ) -> Result<(Option<Bytes>, usize), Self::Error> {
        let key = StateKey::resource(address, struct_tag)
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR))?;
        let buf = self.get(&key)?.map(|v| v.bytes().clone());
        let size = resource_size(&buf);
        Ok((buf, size))
    }
}

// TODO Note for Conflicting: conflicting implementation in crate `starcoin_vm_types`: - impl<V> ConfigStorage for V where V: StateView;
// impl<'a, S: StateView> ConfigStorage for RemoteStorage<'a, S> {
//     fn fetch_config(&self, access_path: AccessPath) -> Option<Vec<u8>> {
//         self.get(&access_path).ok()?
//     }
// }

impl<'a, S> Deref for RemoteStorage<'a, S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, S: StateView> TableResolver for RemoteStorage<'a, S> {
    // TODO(simon): don't ignore maybe_layout
    fn resolve_table_entry_bytes_with_layout(
        &self,
        handle: &TableHandle,
        key: &[u8],
        _maybe_layout: Option<&MoveTypeLayout>,
    ) -> Result<Option<Bytes>, PartialVMError> {
        self.0
            .get_state_value(&StateKey::table_item(&(*handle).into(), key))
            .map(|r| r.map(|v| v.bytes().clone()))
            .map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR).with_message(format!("{:?}", e))
            })
    }
}

pub trait AsMoveResolver<S> {
    fn as_move_resolver(&self) -> RemoteStorage<S>;
}

impl<S: StateView> AsMoveResolver<S> for S {
    fn as_move_resolver(&self) -> RemoteStorage<S> {
        RemoteStorage::new(self)
    }
}

impl<S: StateView> AsExecutorView for S {
    fn as_executor_view(&self) -> &dyn ExecutorView {
        todo!()
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

pub trait IntoMoveResolver<S> {
    fn into_move_resolver(self) -> RemoteStorageOwned<S>;
}

impl<S: StateView> IntoMoveResolver<S> for S {
    fn into_move_resolver(self) -> RemoteStorageOwned<S> {
        RemoteStorageOwned { state_view: self }
    }
}
