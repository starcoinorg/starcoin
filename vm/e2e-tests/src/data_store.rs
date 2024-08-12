// Copyright (c) Starcoin
// SPDX-License-Identifier: Apache-2.0

//! Support for mocking the Aptos data store.
use crate::account::AccountData;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_vm_types::{
    access_path::AccessPath,
    language_storage::ModuleId,
    state_store::state_key::StateKey,
    state_view::StateView,
    write_set::{WriteOp, WriteSet},
};

use starcoin_crypto::HashValue;
use starcoin_statedb::ChainStateWriter;
use starcoin_types::state_set::ChainStateSet;
use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard};
use starcoin_vm_types::state_view::TStateView;

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
    state_data: RwLock<HashMap<StateKey, Vec<u8>>>,
}

impl FakeDataStore {
    /// Creates a new `FakeDataStore` with the provided initial data.
    pub fn new(data: HashMap<StateKey, Vec<u8>>) -> Self {
        FakeDataStore {
            state_data: RwLock::new(data),
        }
    }

    /// Adds a [`WriteSet`] to this data store.
    pub fn add_write_set(&self, write_set: &WriteSet) {
        let mut write_handle = self.state_data.write().expect("Panic for lock");
        for (state_key, write_op) in write_set {
            match write_op {
                WriteOp::Value(blob) => {
                    write_handle.insert(state_key.clone(), blob.clone());
                }
                WriteOp::Deletion => {
                    write_handle.remove(state_key).expect("Panic for remove");
                }
            }
        }
    }

    /// Sets a (key, value) pair within this data store.
    ///
    /// Returns the previous data if the key was occupied.
    pub fn set(&self, state_key: StateKey, data_blob: Vec<u8>) -> Option<Vec<u8>> {
        let mut write_handle = self.state_data.write().expect("Panic for lock");
        write_handle.insert(state_key, data_blob)
    }

    /// Deletes a key from this data store.
    ///
    /// Returns the previous data if the key was occupied.
    pub fn remove(&self, state_key: &StateKey) -> Option<Vec<u8>> {
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
        let access_path = AccessPath::from(module_id);
        self.set(StateKey::AccessPath(access_path), blob);
    }

    /// Yields a reference to the internal data structure of the global state
    pub fn inner(&self) -> RwLockReadGuard<HashMap<StateKey, Vec<u8>>> {
        self.state_data.read().expect("Panic for read state data")
    }
}

// This is used by the `execute_block` API.
// TODO: only the "sync" get is implemented
impl TStateView for FakeDataStore {
    type Key = StateKey;
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<Vec<u8>>> {
        Ok(self.inner().get(state_key).cloned())
    }

    fn is_genesis(&self) -> bool {
        self.inner().is_empty()
    }
}

impl ChainStateWriter for FakeDataStore {
    /// Sets state at access_path.
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()> {
        self.set(StateKey::AccessPath(access_path.clone()), value);
        Ok(())
    }

    /// Remove state at access_path
    fn remove(&self, access_path: &AccessPath) -> Result<()> {
        self.remove(&StateKey::AccessPath(access_path.clone()));
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
