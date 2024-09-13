// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::loader::Loader;
use std::collections::btree_map;

use crate::logging::expect_no_verification_errors;
use bytes::Bytes;
use move_binary_format::errors::*;
use move_binary_format::file_format::CompiledScript;
use move_binary_format::CompiledModule;
use move_core_types::language_storage::StructTag;
use move_core_types::{
    account_address::AccountAddress,
    effects::{AccountChangeSet, ChangeSet, Event, Op},
    gas_algebra::NumBytes,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    resolver::MoveResolver,
    value::MoveTypeLayout,
    vm_status::StatusCode,
};
use move_vm_types::{
    data_store::DataStore,
    loaded_data::runtime_types::Type,
    values::{GlobalValue, Value},
};
use sha3::{Digest, Sha3_256};
use std::collections::btree_map::BTreeMap;
use std::sync::Arc;

pub struct AccountDataCache {
    data_map: BTreeMap<StructTag, (MoveTypeLayout, GlobalValue)>,
    module_map: BTreeMap<Identifier, (Vec<u8>, bool)>,
}

impl AccountDataCache {
    fn new() -> Self {
        Self {
            data_map: BTreeMap::new(),
            module_map: BTreeMap::new(),
        }
    }
}

fn load_module_impl<'r, S: MoveResolver>(
    remote: &'r S,
    account_map: &BTreeMap<AccountAddress, AccountDataCache>,
    module_id: &ModuleId,
) -> PartialVMResult<Bytes> {
    if let Some(account_cache) = account_map.get(module_id.address()) {
        if let Some((blob, _is_republishing)) = account_cache.module_map.get(module_id.name()) {
            return Ok(blob.clone().into());
        }
    }

    match remote.get_module(module_id) {
        Ok(Some(bytes)) => Ok(bytes.into()),
        Ok(None) => Err(PartialVMError::new(StatusCode::LINKER_ERROR)
            .with_message(format!("Cannot find {:?} in data cache", module_id))),
        Err(err) => {
            let msg = format!("Unexpected storage error: {:?}", err);
            Err(
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message(msg),
            )
        }
    }
}

/// Transaction data cache. Keep updates within a transaction so they can all be published at
/// once when the transaction succeeds.
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
pub(crate) struct TransactionDataCache<'r, 'l, S> {
    remote: &'r S,
    loader: &'l Loader,
    account_map: BTreeMap<AccountAddress, AccountDataCache>,
    event_data: Vec<(Vec<u8>, u64, Type, MoveTypeLayout, Value)>,

    // Caches to help avoid duplicate deserialization calls.
    compiled_scripts: BTreeMap<[u8; 32], Arc<CompiledScript>>,
    compiled_modules: BTreeMap<ModuleId, (Arc<CompiledModule>, usize, [u8; 32])>,
}

impl<'r, 'l, S: MoveResolver> TransactionDataCache<'r, 'l, S> {
    /// Create a `TransactionDataCache` with a `RemoteCache` that provides access to data
    /// not updated in the transaction.
    pub(crate) fn new(remote: &'r S, loader: &'l Loader) -> Self {
        TransactionDataCache {
            remote,
            loader,
            account_map: BTreeMap::new(),
            event_data: vec![],

            compiled_scripts: BTreeMap::new(),
            compiled_modules: BTreeMap::new(),
        }
    }

    /// Make a write set from the updated (dirty, deleted) global resources along with
    /// published modules.
    ///
    /// Gives all proper guarantees on lifetime of global data as well.
    pub(crate) fn into_effects(self) -> PartialVMResult<(ChangeSet, Vec<Event>)> {
        let mut change_set = ChangeSet::new();
        for (addr, account_data_cache) in self.account_map.into_iter() {
            let mut modules = BTreeMap::new();
            for (module_name, (module_blob, is_republishing)) in account_data_cache.module_map {
                let op = if is_republishing {
                    Op::Modify(module_blob)
                } else {
                    Op::New(module_blob)
                };
                modules.insert(module_name, op);
            }

            let mut resources = BTreeMap::new();
            for (struct_tag, (layout, gv)) in account_data_cache.data_map {
                let op = match gv.into_effect() {
                    Some(op) => op,
                    None => continue,
                };
                match op {
                    Op::New(val) => {
                        let resource_blob = val
                            .simple_serialize(&layout)
                            .ok_or_else(|| PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR))?;
                        resources.insert(struct_tag, Op::New(resource_blob));
                    }
                    Op::Modify(val) => {
                        let resource_blob = val
                            .simple_serialize(&layout)
                            .ok_or_else(|| PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR))?;
                        resources.insert(struct_tag, Op::Modify(resource_blob));
                    }
                    Op::Delete => {
                        resources.insert(struct_tag, Op::Delete);
                    }
                }
            }
            if !modules.is_empty() || !resources.is_empty() {
                change_set
                    .add_account_changeset(
                        addr,
                        AccountChangeSet::from_modules_resources(modules, resources),
                    )
                    .expect("accounts should be unique");
            }
        }

        let mut events = vec![];
        for (guid, seq_num, ty, ty_layout, val) in self.event_data {
            let ty_tag = self.loader.type_to_type_tag(&ty)?;
            let blob = val
                .simple_serialize(&ty_layout)
                .ok_or_else(|| PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR))?;
            events.push((guid, seq_num, ty_tag, blob))
        }

        Ok((change_set, events))
    }

    pub(crate) fn num_mutated_accounts(&self, sender: &AccountAddress) -> u64 {
        // The sender's account will always be mutated.
        let mut total_mutated_accounts: u64 = 1;
        for (addr, entry) in self.account_map.iter() {
            if addr != sender && entry.data_map.values().any(|(_, v)| v.is_mutated()) {
                total_mutated_accounts += 1;
            }
        }
        total_mutated_accounts
    }

    fn get_mut_or_insert_with<'a, K, V, F>(map: &'a mut BTreeMap<K, V>, k: &K, gen: F) -> &'a mut V
    where
        F: FnOnce() -> (K, V),
        K: Ord,
    {
        if !map.contains_key(k) {
            let (k, v) = gen();
            map.insert(k, v);
        }
        map.get_mut(k).unwrap()
    }

    pub(crate) fn load_compiled_script_to_cache(
        &mut self,
        script_blob: &[u8],
        hash_value: [u8; 32],
    ) -> VMResult<Arc<CompiledScript>> {
        let cache = &mut self.compiled_scripts;
        match cache.entry(hash_value) {
            btree_map::Entry::Occupied(entry) => Ok(entry.get().clone()),
            btree_map::Entry::Vacant(entry) => {
                let script = match CompiledScript::deserialize(script_blob) {
                    Ok(script) => script,
                    Err(err) => {
                        let msg = format!("[VM] deserializer for script returned error: {:?}", err);
                        return Err(PartialVMError::new(StatusCode::CODE_DESERIALIZATION_ERROR)
                            .with_message(msg)
                            .finish(Location::Script));
                    }
                };
                Ok(entry.insert(Arc::new(script)).clone())
            }
        }
    }

    pub(crate) fn load_compiled_module_to_cache(
        &mut self,
        id: ModuleId,
        allow_loading_failure: bool,
    ) -> VMResult<(Arc<CompiledModule>, usize, [u8; 32])> {
        let cache = &mut self.compiled_modules;
        match cache.entry(id) {
            btree_map::Entry::Occupied(entry) => Ok(entry.get().clone()),
            btree_map::Entry::Vacant(entry) => {
                // bytes fetching, allow loading to fail if the flag is set
                let bytes = match load_module_impl(self.remote, &self.account_map, entry.key())
                    .map_err(|err| err.finish(Location::Undefined))
                {
                    Ok(bytes) => bytes,
                    Err(err) if allow_loading_failure => return Err(err),
                    Err(err) => {
                        return Err(expect_no_verification_errors(err));
                    }
                };

                let mut sha3_256 = Sha3_256::new();
                sha3_256.update(&bytes);
                let hash_value: [u8; 32] = sha3_256.finalize().into();

                // for bytes obtained from the data store, they should always deserialize and verify.
                // It is an invariant violation if they don't.
                let module = CompiledModule::deserialize(&bytes)
                    .map_err(|err| {
                        let msg = format!("Deserialization error: {:?}", err);
                        PartialVMError::new(StatusCode::CODE_DESERIALIZATION_ERROR)
                            .with_message(msg)
                            .finish(Location::Module(entry.key().clone()))
                    })
                    .map_err(expect_no_verification_errors)?;

                Ok(entry
                    .insert((Arc::new(module), bytes.len(), hash_value))
                    .clone())
            }
        }
    }
}

// `DataStore` implementation for the `TransactionDataCache`
impl<'r, 'l, S: MoveResolver> DataStore for TransactionDataCache<'r, 'l, S> {
    // Retrieve data from the local cache or loads it from the remote cache into the local cache.
    // All operations on the global data are based on this API and they all load the data
    // into the cache.
    fn load_resource(
        &mut self,
        addr: AccountAddress,
        ty: &Type,
    ) -> PartialVMResult<(&mut GlobalValue, Option<Option<NumBytes>>)> {
        let account_cache = Self::get_mut_or_insert_with(&mut self.account_map, &addr, || {
            (addr, AccountDataCache::new())
        });

        let mut load_res = None;
        let ty_tag = match self.loader.type_to_type_tag(ty)? {
            TypeTag::Struct(s_tag) => s_tag,
            _ =>
            // non-struct top-level value; can't happen
            {
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR))
            }
        };
        if !account_cache.data_map.contains_key(&ty_tag) {
            // TODO(Gas): Shall we charge for this?
            let ty_layout = self.loader.type_to_type_layout(ty)?;

            let gv = match self.remote.get_resource(&addr, &ty_tag) {
                Ok(Some(blob)) => {
                    load_res = Some(Some(NumBytes::new(blob.len() as u64)));
                    let val = match Value::simple_deserialize(&blob, &ty_layout) {
                        Some(val) => val,
                        None => {
                            let msg =
                                format!("Failed to deserialize resource {} at {}!", ty_tag, addr);
                            return Err(PartialVMError::new(
                                StatusCode::FAILED_TO_DESERIALIZE_RESOURCE,
                            )
                            .with_message(msg));
                        }
                    };

                    GlobalValue::cached(val)?
                }
                Ok(None) => {
                    load_res = Some(None);
                    GlobalValue::none()
                }
                Err(err) => {
                    let msg = format!("Unexpected storage error: {:?}", err);
                    return Err(
                        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                            .with_message(msg),
                    );
                }
            };

            account_cache
                .data_map
                .insert(*ty_tag.clone(), (ty_layout, gv));
        }

        Ok((
            account_cache
                .data_map
                .get_mut(&ty_tag)
                .map(|(_ty_layout, gv)| gv)
                .expect("global value must exist"),
            load_res,
        ))
    }

    fn load_module(&self, module_id: &ModuleId) -> VMResult<Vec<u8>> {
        load_module_impl(self.remote, &self.account_map, module_id)
            .map(|bytes| bytes.to_vec())
            .map_err(|pve| pve.finish(Location::Undefined))
    }

    fn publish_module(
        &mut self,
        module_id: &ModuleId,
        blob: Vec<u8>,
        is_republishing: bool,
    ) -> VMResult<()> {
        let account_cache =
            Self::get_mut_or_insert_with(&mut self.account_map, module_id.address(), || {
                (*module_id.address(), AccountDataCache::new())
            });

        account_cache
            .module_map
            .insert(module_id.name().to_owned(), (blob, is_republishing));

        Ok(())
    }

    fn exists_module(&self, module_id: &ModuleId) -> VMResult<bool> {
        if let Some(account_cache) = self.account_map.get(module_id.address()) {
            if account_cache.module_map.contains_key(module_id.name()) {
                return Ok(true);
            }
        }
        Ok(self
            .remote
            .get_module(module_id)
            .map_err(|_| {
                PartialVMError::new(StatusCode::STORAGE_ERROR).finish(Location::Undefined)
            })?
            .is_some())
    }

    fn emit_event(
        &mut self,
        guid: Vec<u8>,
        seq_num: u64,
        ty: Type,
        val: Value,
    ) -> PartialVMResult<()> {
        let ty_layout = self.loader.type_to_type_layout(&ty)?;
        Ok(self.event_data.push((guid, seq_num, ty, ty_layout, val)))
    }

    fn events(&self) -> &Vec<(Vec<u8>, u64, Type, MoveTypeLayout, Value)> {
        &self.event_data
    }
}
