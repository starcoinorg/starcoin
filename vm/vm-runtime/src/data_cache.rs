// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Scratchpad for on chain values during the execution.

use move_vm_runtime::data_cache::RemoteCache;
use starcoin_logger::prelude::*;
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::data_store::DataStore;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::loaded_data::types::FatStructType;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::values::{GlobalValue, Struct, Value};
use starcoin_vm_types::write_set::WriteSetMut;
use starcoin_vm_types::{
    access_path::AccessPath,
    vm_error::{StatusCode, VMStatus},
    write_set::{WriteOp, WriteSet},
};
use std::collections::btree_map::BTreeMap;
use std::mem::replace;
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

    pub fn is_genesis(&self) -> bool {
        self.data_view.is_genesis()
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

/// Transaction data cache. Keep updates within a transaction so they can all be published at
/// once when the transaction succeeeds.
///
/// It also provides an implementation for the opcodes that refer to storage and gives the
/// proper guarantees of reference lifetime.
///
/// Dirty objects are serialized and returned in make_write_set.
///
/// It is a responsibility of the client to publish changes once the transaction is executed.
///
/// The Move VM takes a `DataStore` in input and this is the default and correct implementation
/// for a data store related to a transaction. Clients should create an instance of this type
/// and pass it to the Move VM.
pub struct TransactionDataCache<'txn> {
    data_map: BTreeMap<AccessPath, Option<(FatStructType, GlobalValue, usize)>>,
    module_map: BTreeMap<ModuleId, Vec<u8>>,
    event_data: Vec<ContractEvent>,
    data_cache: &'txn dyn RemoteCache,
    size_map: BTreeMap<AccountAddress, i64>,
}

impl<'txn> TransactionDataCache<'txn> {
    /// Create a `TransactionDataCache` with a `RemoteCache` that provides access to data
    /// not updated in the transaction.
    pub fn new(data_cache: &'txn dyn RemoteCache) -> Self {
        TransactionDataCache {
            data_cache,
            data_map: BTreeMap::new(),
            module_map: BTreeMap::new(),
            event_data: vec![],
            size_map: BTreeMap::new(),
        }
    }

    /// Make a write set from the updated (dirty, deleted) global resources along with
    /// published modules.
    ///
    /// Gives all proper guarantees on lifetime of global data as well.
    pub fn make_write_set(&mut self) -> VMResult<WriteSet> {
        if self.data_map.len() + self.module_map.len() > usize::max_value() {
            return Err(vm_error(Location::new(), StatusCode::INVALID_DATA));
        }

        let mut sorted_ws: BTreeMap<AccessPath, WriteOp> = BTreeMap::new();

        let data_map = replace(&mut self.data_map, BTreeMap::new());
        for (key, global_val) in data_map {
            match global_val {
                Some((layout, global_val, _)) => {
                    if global_val.is_dirty()? {
                        // into_owned_struct will check if all references are properly released
                        // at the end of a transaction
                        let data = global_val.into_owned_struct()?;
                        let blob = match data.simple_serialize(&layout) {
                            Some(blob) => blob,
                            None => {
                                return Err(vm_error(
                                    Location::new(),
                                    StatusCode::VALUE_SERIALIZATION_ERROR,
                                ))
                            }
                        };
                        sorted_ws.insert(key, WriteOp::Value(blob));
                    }
                }
                None => {
                    sorted_ws.insert(key, WriteOp::Deletion);
                }
            }
        }

        let module_map = replace(&mut self.module_map, BTreeMap::new());
        for (module_id, module) in module_map {
            sorted_ws.insert((&module_id).into(), WriteOp::Value(module));
        }

        let mut write_set = WriteSetMut::new(Vec::new());
        for (key, value) in sorted_ws {
            write_set.push((key, value));
        }
        write_set
            .freeze()
            .map_err(|_| vm_error(Location::new(), StatusCode::DATA_FORMAT_ERROR))
    }

    /// Return the events that were published during the execution of the transaction.
    pub fn event_data(&self) -> &[ContractEvent] {
        &self.event_data
    }

    /// Get size by account address
    pub fn get_size(&self, address: AccountAddress) -> i64 {
        match self.size_map.get(&address) {
            Some(size) => *size,
            _ => 0,
        }
    }

    // Retrieve data from the local cache or loads it from the remote cache into the local cache.
    // All operations on the global data are based on this API and they all load the data
    // into the cache.
    fn load_data(
        &mut self,
        ap: &AccessPath,
        ty: &FatStructType,
    ) -> VMResult<&mut Option<(FatStructType, GlobalValue, usize)>> {
        if !self.data_map.contains_key(ap) {
            match self.data_cache.get(ap)? {
                Some(bytes) => {
                    let size = bytes.len();
                    let res = Struct::simple_deserialize(&bytes, ty)?;
                    let global_val = GlobalValue::new(Value::struct_(res))?;
                    self.data_map
                        .insert(ap.clone(), Some((ty.clone(), global_val, size)));
                }
                None => {
                    return Err(
                        VMStatus::new(StatusCode::MISSING_DATA).with_message(format!(
                            "Cannot find {:?}::{}::{} for Access Path: {:?}",
                            &ty.address,
                            &ty.module.as_str(),
                            &ty.name.as_str(),
                            ap
                        )),
                    );
                }
            };
        }
        Ok(self.data_map.get_mut(ap).expect("data must exist"))
    }
}

// `DataStore` implementation for the `TransactionDataCache`
impl<'a> DataStore for TransactionDataCache<'a> {
    fn publish_resource(
        &mut self,
        ap: &AccessPath,
        g: (FatStructType, GlobalValue),
    ) -> VMResult<()> {
        //TODO modify to GlobalValue.size() in future

        let data = g.1.into_owned_struct().unwrap();
        match data.simple_serialize(&g.0) {
            Some(blob) => {
                let len = blob.len();
                let global_val = GlobalValue::new(Value::struct_(data)).unwrap();
                global_val.mark_dirty()?;
                let g_wrap = (g.0.clone(), global_val, len);
                self.data_map.insert(ap.clone(), Some(g_wrap));

                self.size_map
                    .entry(ap.clone().address)
                    .and_modify(|v| *v += len as i64)
                    .or_insert(len as i64);
            }
            None => {
                return Err(vm_error(
                    Location::new(),
                    StatusCode::VALUE_SERIALIZATION_ERROR,
                ));
            }
        };
        Ok(())
    }

    fn borrow_resource(
        &mut self,
        ap: &AccessPath,
        ty: &FatStructType,
    ) -> VMResult<Option<&GlobalValue>> {
        let map_entry = self.load_data(ap, ty)?;
        Ok(map_entry.as_ref().map(|(_, g, _)| g))
    }

    fn move_resource_from(
        &mut self,
        ap: &AccessPath,
        ty: &FatStructType,
    ) -> VMResult<Option<GlobalValue>> {
        let map_entry = self.load_data(ap, ty)?;
        // .take() means that the entry is removed from the data map -- this marks the
        // access path for deletion.
        let (_, global_value, size) = map_entry.take().unwrap();
        self.size_map
            .entry(ap.clone().address)
            .and_modify(|v| *v -= size as i64)
            .or_insert(size as i64);
        Ok(Some(global_value))
    }

    fn load_module(&self, module: &ModuleId) -> VMResult<Vec<u8>> {
        match self.module_map.get(module) {
            Some(bytes) => Ok(bytes.clone()),
            None => {
                let ap = AccessPath::from(module);
                self.data_cache.get(&ap).and_then(|data| {
                    data.ok_or_else(|| {
                        VMStatus::new(StatusCode::LINKER_ERROR)
                            .with_message(format!("Cannot find {:?} in data cache", module))
                    })
                })
            }
        }
    }

    fn publish_module(&mut self, m: ModuleId, bytes: Vec<u8>) -> VMResult<()> {
        self.module_map.insert(m, bytes);
        Ok(())
    }

    fn exists_module(&self, m: &ModuleId) -> bool {
        self.module_map.contains_key(m) || {
            let ap = AccessPath::from(m);
            matches!(self.data_cache.get(&ap), Ok(Some(_)))
        }
    }

    fn emit_event(&mut self, event: ContractEvent) {
        self.event_data.push(event)
    }
}
