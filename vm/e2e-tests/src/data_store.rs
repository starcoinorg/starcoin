// Copyright (c) Starcoin
// SPDX-License-Identifier: Apache-2.0

//! Support for mocking the Aptos data store.
use crate::account::AccountData;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::write_set::{WriteOp, WriteSet};
use std::collections::HashMap;

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
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FakeDataStore {
    state_data: HashMap<StateKey, Vec<u8>>,
}

impl FakeDataStore {
    /// Creates a new `FakeDataStore` with the provided initial data.
    pub fn new(data: HashMap<StateKey, Vec<u8>>) -> Self {
        FakeDataStore { state_data: data }
    }

    /// Adds a [`WriteSet`] to this data store.
    pub fn add_write_set(&mut self, write_set: &WriteSet) {
        for (state_key, write_op) in write_set {
            match write_op {
                WriteOp::Value(blob) => {
                    self.set(state_key.clone(), blob.clone());
                }
                WriteOp::Deletion => {
                    self.remove(state_key);
                }
            }
        }
    }

    /// Sets a (key, value) pair within this data store.
    ///
    /// Returns the previous data if the key was occupied.
    pub fn set(&mut self, state_key: StateKey, data_blob: Vec<u8>) -> Option<Vec<u8>> {
        self.state_data.insert(state_key, data_blob)
    }

    /// Deletes a key from this data store.
    ///
    /// Returns the previous data if the key was occupied.
    pub fn remove(&mut self, state_key: &StateKey) -> Option<Vec<u8>> {
        self.state_data.remove(state_key)
    }

    /// Adds an [`AccountData`] to this data store.
    pub fn add_account_data(&mut self, account_data: &AccountData) {
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
    pub fn inner(&self) -> &HashMap<StateKey, Vec<u8>> {
        &self.state_data
    }
}

// This is used by the `execute_block` API.
// TODO: only the "sync" get is implemented
impl StateView for FakeDataStore {
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<Vec<u8>>> {
        Ok(self.state_data.get(state_key).cloned())
    }

    fn is_genesis(&self) -> bool {
        self.state_data.is_empty()
    }
}
