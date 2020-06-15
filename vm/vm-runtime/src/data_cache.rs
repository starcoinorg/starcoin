// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Scratchpad for on chain values during the execution.

use move_vm_runtime::data_cache::{RemoteCache, TransactionDataCache};
use starcoin_logger::prelude::*;
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::data_store::DataStore;
use starcoin_vm_types::gas_schedule::GasAlgebra;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::loaded_data::types::FatStructType;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::values::GlobalValue;
use starcoin_vm_types::{
    access_path::AccessPath,
    vm_error::{StatusCode, VMStatus},
    write_set::{WriteOp, WriteSet},
};
use std::collections::btree_map::BTreeMap;
use vm::errors::*;

/// A local cache for a given a `StateView`. The cache is private to the Libra layer
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
pub struct StateViewCache<'a> {
    data_view: &'a dyn StateView,
    data_map: BTreeMap<AccessPath, Option<Vec<u8>>>,
}

impl<'a> StateViewCache<'a> {
    /// Create a `StateViewCache` give a `StateView`. Hold updates to the data store and
    /// forward data request to the `StateView` if not in the local cache.
    pub fn new(data_view: &'a dyn StateView) -> Self {
        StateViewCache {
            data_view,
            data_map: BTreeMap::new(),
        }
    }

    // Get some data either through the cache or the `StateView` on a cache miss.
    pub(crate) fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>> {
        match self.data_map.get(access_path) {
            Some(opt_data) => Ok(opt_data.clone()),
            None => match self.data_view.get(&access_path) {
                Ok(remote_data) => Ok(remote_data),
                // TODO: should we forward some error info?
                Err(_) => {
                    error!("[VM] Error getting data from storage for {:?}", access_path);
                    Err(VMStatus::new(StatusCode::STORAGE_ERROR))
                }
            },
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

impl<'block> RemoteCache for StateViewCache<'block> {
    fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>> {
        StateViewCache::get(self, access_path)
    }
}

// Adapter to convert a `StateView` into a `RemoteCache`.
pub struct RemoteStorage<'a>(&'a dyn StateView);

impl<'a> RemoteStorage<'a> {
    pub fn new(state_store: &'a dyn StateView) -> Self {
        Self(state_store)
    }
}

impl<'a> RemoteCache for RemoteStorage<'a> {
    fn get(&self, access_path: &AccessPath) -> VMResult<Option<Vec<u8>>> {
        self.0
            .get(access_path)
            .map_err(|_| VMStatus::new(StatusCode::STORAGE_ERROR))
    }
}

pub struct StarcoinDataCache<'txn>(TransactionDataCache<'txn>, BTreeMap<AccountAddress, u64>);
impl<'txn> StarcoinDataCache<'txn> {
    pub fn new(data_cache: &'txn dyn RemoteCache) -> Self {
        Self(TransactionDataCache::new(data_cache), BTreeMap::new())
    }
    /// override make_write_set method
    pub fn make_write_set(&mut self) -> VMResult<WriteSet> {
        self.0.make_write_set()
    }

    pub fn event_data(&self) -> &[ContractEvent] {
        self.0.event_data()
    }

    /// Get size by account address
    pub fn get_size(&self, address: AccountAddress) -> u64 {
        match self.1.get(&address) {
            Some(size) => *size,
            _ => 0,
        }
    }
}
// `DataStore` implementation for the `StarcoinDataCache`
impl<'a> DataStore for StarcoinDataCache<'a> {
    fn publish_resource(
        &mut self,
        ap: &AccessPath,
        g: (FatStructType, GlobalValue),
    ) -> VMResult<()> {
        let new_size = g.1.size().get();
        self.0.publish_resource(ap, g)?;

        self.1
            .entry(ap.clone().address)
            .and_modify(|v| *v += new_size as u64);
        Ok(())
    }

    fn borrow_resource(
        &mut self,
        ap: &AccessPath,
        ty: &FatStructType,
    ) -> VMResult<Option<&GlobalValue>> {
        self.0.borrow_resource(ap, ty)
    }

    fn move_resource_from(
        &mut self,
        ap: &AccessPath,
        ty: &FatStructType,
    ) -> VMResult<Option<GlobalValue>> {
        let global_value = self.0.move_resource_from(ap, ty)?;
        match global_value {
            Some(g) => {
                let new_size = g.size().get();
                self.1
                    .entry(ap.clone().address)
                    .and_modify(|v| *v -= new_size as u64);
                Ok(Some(g))
            }
            _ => Ok(None),
        }
    }

    fn load_module(&self, module: &ModuleId) -> VMResult<Vec<u8>> {
        self.0.load_module(module)
    }

    fn publish_module(&mut self, m: ModuleId, bytes: Vec<u8>) -> VMResult<()> {
        self.0.publish_module(m, bytes)
    }

    fn exists_module(&self, m: &ModuleId) -> bool {
        self.0.exists_module(m)
    }

    fn emit_event(&mut self, event: ContractEvent) {
        self.0.emit_event(event)
    }
}
