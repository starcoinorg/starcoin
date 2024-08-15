// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Scratchpad for on chain values during the execution.

use crate::create_access_path;
use bytes::Bytes;
use move_binary_format::CompiledModule;
use move_core_types::metadata::Metadata;
use move_core_types::value::MoveTypeLayout;
use move_table_extension::{TableHandle, TableResolver};
use move_vm_types::resolver::{resource_size, ModuleResolver, ResourceResolver};
use starcoin_logger::prelude::*;
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::{
    access_path::AccessPath,
    errors::*,
    language_storage::{ModuleId, StructTag},
    state_view::StateView,
    vm_status::StatusCode,
    write_set::{WriteOp, WriteSet},
};
use std::collections::btree_map::BTreeMap;
use std::ops::{Deref, DerefMut};

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
            match write_op {
                WriteOp::Value(blob) => {
                    self.data_map.insert(ap.clone(), Some(blob.clone()));
                }
                WriteOp::Deletion => {
                    self.data_map.remove(ap);
                    self.data_map.insert(ap.clone(), None);
                }
            }
        }
    }
}

impl<'block, S: StateView> StateView for StateViewCache<'block, S> {
    // Get some data either through the cache or the `StateView` on a cache miss.
    fn get_state_value(&self, state_key: &StateKey) -> anyhow::Result<Option<Vec<u8>>> {
        match self.data_map.get(state_key) {
            Some(opt_data) => Ok(opt_data.clone()),
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

    fn is_genesis(&self) -> bool {
        self.data_view.is_genesis()
    }
}

impl<'block, S: StateView> ModuleResolver for StateViewCache<'block, S> {
    fn get_module_metadata(&self, module_id: &ModuleId) -> Vec<Metadata> {
        RemoteStorage::new(self).get_module_metadata(module_id)
    }

    fn get_module(&self, module_id: &ModuleId) -> PartialVMResult<Option<Bytes>> {
        RemoteStorage::new(self).get_module(module_id)
    }
}
impl<'block, S: StateView> ResourceResolver for StateViewCache<'block, S> {
    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
        metadata: &[Metadata],
        layout: Option<&MoveTypeLayout>,
    ) -> PartialVMResult<(Option<Bytes>, usize)> {
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

    pub fn get(&self, access_path: &AccessPath) -> PartialVMResult<Option<Vec<u8>>> {
        self.0
            .get_state_value(&StateKey::AccessPath(access_path.clone()))
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR))
    }
}

impl<'a, S: StateView> ModuleResolver for RemoteStorage<'a, S> {
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

    fn get_module(&self, module_id: &ModuleId) -> PartialVMResult<Option<Bytes>> {
        // REVIEW: cache this?
        let ap = AccessPath::from(module_id);
        self.get(&ap).map(|r| r.map(Bytes::from))
    }
}
impl<'a, S: StateView> ResourceResolver for RemoteStorage<'a, S> {
    // TODO(simon): don't ignore metadata and layout
    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
        _metadata: &[Metadata],
        _layout: Option<&MoveTypeLayout>,
    ) -> PartialVMResult<(Option<Bytes>, usize)> {
        let ap = create_access_path(*address, struct_tag.clone());
        let buf = self.get(&ap)?.map(Bytes::from);
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
            .get_state_value(&StateKey::table_item((*handle).into(), key.to_vec()))
            .map(|r| r.map(Bytes::from))
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
    fn get_module_metadata(&self, module_id: &ModuleId) -> Vec<Metadata> {
        self.as_move_resolver().get_module_metadata(module_id)
    }

    fn get_module(&self, module_id: &ModuleId) -> PartialVMResult<Option<Bytes>> {
        self.as_move_resolver().get_module(module_id)
    }
}

impl<S: StateView> ResourceResolver for RemoteStorageOwned<S> {
    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
        metadata: &[Metadata],
        layout: Option<&MoveTypeLayout>,
    ) -> PartialVMResult<(Option<Bytes>, usize)> {
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

// TODO Note for Conflicting: conflicting implementation in crate `starcoin_vm_types`: - impl<V> ConfigStorage for V where V: StateView;
// impl<S: StateView> ConfigStorage for RemoteStorageOwned<S> {
//     fn fetch_config(&self, access_path: AccessPath) -> Option<Vec<u8>> {
//         self.as_move_resolver().fetch_config(access_path)
//     }
// }

pub trait IntoMoveResolver<S> {
    fn into_move_resolver(self) -> RemoteStorageOwned<S>;
}

impl<S: StateView> IntoMoveResolver<S> for S {
    fn into_move_resolver(self) -> RemoteStorageOwned<S> {
        RemoteStorageOwned { state_view: self }
    }
}
