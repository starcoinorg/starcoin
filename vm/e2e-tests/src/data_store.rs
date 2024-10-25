// Copyright (c) Starcoin
// SPDX-License-Identifier: Apache-2.0

//! Support for mocking the Aptos data store.
use crate::account::AccountData;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_vm_types::{
    access_path::AccessPath, language_storage::ModuleId, state_store::state_key::StateKey,
    write_set::WriteSet,
};

use starcoin_crypto::HashValue;
use starcoin_statedb::ChainStateWriter;
use starcoin_types::state_set::ChainStateSet;
use starcoin_vm_types::access_path::DataPath;
use starcoin_vm_types::state_store::errors::StateviewError;
use starcoin_vm_types::state_store::state_storage_usage::StateStorageUsage;
use starcoin_vm_types::state_store::state_value::StateValue;
use starcoin_vm_types::state_store::TStateView;
use starcoin_vm_types::write_set::TransactionWrite;
use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard};

/// Dummy genesis ChangeSet for testing
// TODO(BobOng): e2e-test
// pub static GENESIS_CHANGE_SET: Lazy<ChangeSet> =
//     Lazy::new(|| generate_genesis_change_set_for_testing(GenesisOptions::Compiled));
//
// pub static GENESIS_CHANGE_SET_FRESH: Lazy<ChangeSet> =
//     Lazy::new(|| generate_genesis_change_set_for_testing(GenesisOptions::Fresh));

/// An in-memory implementation of [`StateView`] and [`RemoteCache`] for the VM.
///
/// Tests use this to set up state, and pass in a reference to the cache whenever a `StateView` or
/// `RemoteCache` is needed.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FakeDataStore {
    state_data: RwLock<HashMap<StateKey, StateValue>>,
}

impl FakeDataStore {
    /// Creates a new `FakeDataStore` with the provided initial data.
    pub fn new(data: HashMap<StateKey, Vec<u8>>) -> Self {
        let data = data
            .into_iter()
            .map(|(k, v)| (k, StateValue::new_legacy(v.into())))
            .collect();
        FakeDataStore {
            state_data: RwLock::new(data),
        }
    }

    /// Adds a [`WriteSet`] to this data store.
    pub fn add_write_set(&self, write_set: &WriteSet) {
        let mut write_handle = self.state_data.write().expect("Panic for lock");
        for (state_key, write_op) in write_set {
            match write_op.as_state_value() {
                Some(blob) => {
                    write_handle.insert(state_key.clone(), blob);
                }
                None => {
                    write_handle.remove(state_key).expect("Panic for remove");
                }
            }
        }
    }

    /// Sets a (key, value) pair within this data store.
    ///
    /// Returns the previous data if the key was occupied.
    pub fn set(&self, state_key: StateKey, data_blob: Vec<u8>) -> Option<StateValue> {
        let mut write_handle = self.state_data.write().expect("Panic for lock");
        write_handle.insert(state_key, StateValue::new_legacy(data_blob.into()))
    }

    /// Deletes a key from this data store.
    ///
    /// Returns the previous data if the key was occupied.
    pub fn remove(&self, state_key: &StateKey) -> Option<StateValue> {
        let mut write_handle = self.state_data.write().expect("Panic for lock");
        write_handle.remove(state_key)
    }

    /// Adds an [`AccountData`] to this data store.
    pub fn add_account_data(&self, account_data: &AccountData) {
        let write_set = account_data.to_writeset();
        self.add_write_set(&write_set)
    }

    /// Adds a [`CompiledModule`] to this data store.
    ///
    /// Does not do any sort of verification on the module.
    pub fn add_module(&mut self, module_id: &ModuleId, blob: Vec<u8>) {
        self.set(StateKey::module_id(module_id), blob);
    }

    /// Yields a reference to the internal data structure of the global state
    pub fn inner(&self) -> RwLockReadGuard<HashMap<StateKey, StateValue>> {
        self.state_data.read().expect("Panic for read state data")
    }
}

// This is used by the `execute_block` API.
// TODO: only the "sync" get is implemented
impl TStateView for FakeDataStore {
    type Key = StateKey;
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<StateValue>, StateviewError> {
        Ok(self.inner().get(state_key).cloned())
    }

    fn get_usage(&self) -> starcoin_vm_types::state_store::Result<StateStorageUsage> {
        unimplemented!("get_usage not implemented for FakeDataStore")
    }

    fn is_genesis(&self) -> bool {
        self.inner().is_empty()
    }
}

impl ChainStateWriter for FakeDataStore {
    /// Sets state at access_path.
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()> {
        let state_key = {
            match &access_path.path {
                DataPath::Code(name) => StateKey::module(&access_path.address, name),
                DataPath::Resource(struct_tag) => {
                    StateKey::resource(&access_path.address, struct_tag)?
                }
                DataPath::ResourceGroup(_) => unimplemented!(),
            }
        };
        self.set(state_key, value);
        Ok(())
    }

    /// Remove state at access_path
    fn remove(&self, access_path: &AccessPath) -> Result<()> {
        let state_key = {
            match &access_path.path {
                DataPath::Code(name) => StateKey::module(&access_path.address, name),
                DataPath::Resource(struct_tag) => {
                    StateKey::resource(&access_path.address, struct_tag)?
                }
                DataPath::ResourceGroup(_) => unimplemented!(),
            }
        };
        self.remove(&state_key);
        Ok(())
    }

    /// Apply dump result to ChainState
    fn apply(&self, _state_set: ChainStateSet) -> Result<()> {
        Ok(())
    }

    fn apply_write_set(&self, write_set: WriteSet) -> Result<()> {
        self.add_write_set(&write_set);
        Ok(())
    }

    fn commit(&self) -> Result<HashValue> {
        Ok(HashValue::zero())
    }

    fn flush(&self) -> Result<()> {
        Ok(())
    }
}
