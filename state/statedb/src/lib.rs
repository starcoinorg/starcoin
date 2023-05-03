// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::StateError::AccountNotExist;
use anyhow::{bail, ensure, Result};
use bcs_ext::BCSCodec;
use forkable_jellyfish_merkle::proof::SparseMerkleProof;
use forkable_jellyfish_merkle::RawKey;
use lru::LruCache;
use parking_lot::{Mutex, RwLock};
use starcoin_crypto::hash::SPARSE_MERKLE_PLACEHOLDER_HASH;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
pub use starcoin_state_api::{ChainStateReader, ChainStateWriter, StateProof, StateWithProof};
use starcoin_state_api::{StateWithTableItemProof, TABLE_PATH_LIST};
use starcoin_state_tree::mock::MockStateNodeStore;
use starcoin_state_tree::AccountStateSetIterator;
use starcoin_state_tree::{StateNodeStore, StateTree};
use starcoin_types::write_set::{WriteOp, WriteSet, WriteSetMut};
use starcoin_types::{
    access_path::{AccessPath, DataType},
    account_address::AccountAddress,
    account_state::AccountState,
    state_set::{AccountStateSet, ChainStateSet},
};
use starcoin_vm_types::access_path::{DataPath, ModuleName};
use starcoin_vm_types::account_config::TABLE_HANDLE_ADDRESS_LIST;
use starcoin_vm_types::language_storage::StructTag;
use starcoin_vm_types::state_store::{state_key::StateKey, table::TableHandle};
use starcoin_vm_types::state_view::StateView;
use std::collections::HashSet;
use std::convert::TryInto;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("the Account for key `{0}` is not exist")]
    AccountNotExist(AccountAddress),
}

enum CacheItem {
    AccountObject(Arc<AccountStateObject>),
    AccountNotExist(),
}

impl CacheItem {
    fn new(obj: Arc<AccountStateObject>) -> Self {
        CacheItem::AccountObject(obj)
    }

    fn as_object(&self) -> Option<Arc<AccountStateObject>> {
        match self {
            CacheItem::AccountObject(obj) => Some(obj.clone()),
            CacheItem::AccountNotExist() => None,
        }
    }
}

/// represent AccountState in runtime memory.
struct AccountStateObject {
    //TODO if use RefCell at here, compile error for ActorRef async interface
    // the trait `std::marker::Sync` is not implemented for AccountStateObject
    // refactor AccountStateObject to a readonly object.
    code_tree: Mutex<Option<StateTree<ModuleName>>>,
    resource_tree: Mutex<StateTree<StructTag>>,
    store: Arc<dyn StateNodeStore>,
}

impl AccountStateObject {
    pub fn new(account_state: AccountState, store: Arc<dyn StateNodeStore>) -> Self {
        let code_tree = account_state
            .code_root()
            .map(|root| StateTree::<ModuleName>::new(store.clone(), Some(root)));
        let resource_tree =
            StateTree::<StructTag>::new(store.clone(), Some(account_state.resource_root()));

        Self {
            code_tree: Mutex::new(code_tree),
            resource_tree: Mutex::new(resource_tree),
            store,
        }
    }

    pub fn empty_account(store: Arc<dyn StateNodeStore>) -> Self {
        let resource_tree = StateTree::<StructTag>::new(store.clone(), None);
        Self {
            code_tree: Mutex::new(None),
            resource_tree: Mutex::new(resource_tree),
            store,
        }
    }

    pub fn get(&self, data_path: &DataPath) -> Result<Option<Vec<u8>>> {
        match data_path {
            DataPath::Code(module_name) => Ok(self
                .code_tree
                .lock()
                .as_ref()
                .map(|tree| tree.get(module_name))
                .transpose()?
                .flatten()),
            DataPath::Resource(struct_tag) => self.resource_tree.lock().get(struct_tag),
        }
    }

    /// return value with it proof.
    /// NOTICE: Any un-committed modification will not visible to the method.
    pub fn get_with_proof(
        &self,
        data_path: &DataPath,
    ) -> Result<(Option<Vec<u8>>, SparseMerkleProof)> {
        match data_path {
            DataPath::Code(module_name) => Ok(self
                .code_tree
                .lock()
                .as_ref()
                .map(|tree| tree.get_with_proof(module_name))
                .transpose()?
                .unwrap_or((None, SparseMerkleProof::new(None, vec![])))),
            DataPath::Resource(struct_tag) => self.resource_tree.lock().get_with_proof(struct_tag),
        }
    }

    pub fn set(&self, data_path: DataPath, value: Vec<u8>) {
        match data_path {
            DataPath::Code(module_name) => {
                if self.code_tree.lock().is_none() {
                    *self.code_tree.lock() =
                        Some(StateTree::<ModuleName>::new(self.store.clone(), None));
                }
                self.code_tree
                    .lock()
                    .as_ref()
                    .expect("state tree must exist after set.")
                    .put(module_name, value);
            }
            DataPath::Resource(struct_tag) => {
                self.resource_tree.lock().put(struct_tag, value);
            }
        }
    }

    pub fn remove(&self, data_path: &DataPath) -> Result<()> {
        if data_path.is_code() {
            bail!("Not supported remove code currently.");
        }
        let struct_tag = data_path
            .as_struct_tag()
            .expect("DataPath must been struct tag at here.");
        self.resource_tree.lock().remove(struct_tag);
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        if self.resource_tree.lock().is_dirty() {
            return true;
        }
        if let Some(code_tree) = self.code_tree.lock().as_ref() {
            if code_tree.is_dirty() {
                return true;
            }
        }
        false
    }

    pub fn commit(&self) -> Result<AccountState> {
        {
            let code_tree = self.code_tree.lock();
            if let Some(code_tree) = code_tree.as_ref() {
                if code_tree.is_dirty() {
                    code_tree.commit()?;
                }
            }
        }
        {
            let resource_tree = self.resource_tree.lock();
            if resource_tree.is_dirty() {
                resource_tree.commit()?;
            }
        }
        Ok(self.to_state())
    }

    pub fn flush(&self) -> Result<()> {
        self.resource_tree.lock().flush()?;
        if let Some(code_tree) = self.code_tree.lock().as_ref() {
            code_tree.flush()?;
        }

        Ok(())
    }

    fn to_state_set(&self) -> Result<AccountStateSet> {
        let code_root = self
            .code_tree
            .lock()
            .as_ref()
            .map(|tree| tree.dump())
            .transpose()?;
        let resource_root = self.resource_tree.lock().dump()?;
        Ok(AccountStateSet::new(vec![code_root, Some(resource_root)]))
    }
    fn to_state(&self) -> AccountState {
        let code_root = self.code_tree.lock().as_ref().map(|tree| tree.root_hash());
        let resource_root = self.resource_tree.lock().root_hash();
        AccountState::new(code_root, resource_root)
    }
}

#[allow(clippy::upper_case_acronyms)]
pub struct ChainStateDB {
    store: Arc<dyn StateNodeStore>,
    ///global state tree.
    state_tree: StateTree<AccountAddress>,
    cache: Mutex<LruCache<AccountAddress, CacheItem>>,
    updates: RwLock<HashSet<AccountAddress>>,
    updates_table_handle: RwLock<HashSet<TableHandle>>,
    cache_table_handle: Mutex<LruCache<TableHandle, Arc<TableHandleStateObject>>>,
    /// state_tree_table_handles root_hash save in [TABLE_HANDLE_ADDRESS_LIST, TABLE_PATH_LIST]
    /// state_tree_table_handles SMT save VEC [TableHandle -> TableHandleState.root_hash, TABLE_ADDRESS_ARR_LEN]
    state_tree_table_handles: Vec<StateTree<TableHandle>>,
    update_table_handle_idx_list: Mutex<HashSet<usize>>,
}

static G_DEFAULT_CACHE_SIZE: usize = 10240;

impl ChainStateDB {
    pub fn mock() -> Self {
        Self::new(Arc::new(MockStateNodeStore::new()), None)
    }

    pub fn new(store: Arc<dyn StateNodeStore>, root_hash: Option<HashValue>) -> Self {
        let mut chain_statedb = ChainStateDB {
            store: store.clone(),
            state_tree: StateTree::new(store.clone(), root_hash),
            cache: Mutex::new(LruCache::new(G_DEFAULT_CACHE_SIZE)),
            updates: RwLock::new(HashSet::new()),
            updates_table_handle: RwLock::new(HashSet::new()),
            cache_table_handle: Mutex::new(LruCache::new(G_DEFAULT_CACHE_SIZE)),
            state_tree_table_handles: vec![],
            update_table_handle_idx_list: Mutex::new(HashSet::new()),
        };
        for (handle_address, table_path) in
            TABLE_HANDLE_ADDRESS_LIST.iter().zip(TABLE_PATH_LIST.iter())
        {
            let account_state_object = chain_statedb
                .get_account_state_object(handle_address, true)
                .expect("get account state success");
            let state_root = account_state_object
                .get(table_path)
                .expect("get state_root success");
            match state_root {
                Some(state_root) => {
                    let hash =
                        HashValue::from_slice(state_root.as_slice()).expect("hash value success");
                    chain_statedb
                        .state_tree_table_handles
                        .push(StateTree::new(store.clone(), Some(hash)));
                }
                None => chain_statedb
                    .state_tree_table_handles
                    .push(StateTree::new(store.clone(), None)),
            };
        }
        chain_statedb
    }

    /// Fork a new statedb base current statedb
    pub fn fork(&self) -> Self {
        Self::new(self.store.clone(), Some(self.state_root()))
    }

    /// Fork a new statedb at `root_hash`
    pub fn fork_at(&self, state_root: HashValue) -> Self {
        Self::new(self.store.clone(), Some(state_root))
    }

    fn new_state_tree<K: RawKey>(&self, root_hash: HashValue) -> StateTree<K> {
        StateTree::new(self.store.clone(), Some(root_hash))
    }

    fn get_account_state_object(
        &self,
        account_address: &AccountAddress,
        create: bool,
    ) -> Result<Arc<AccountStateObject>> {
        let account_state_object = self.get_account_state_object_option(account_address)?;
        match account_state_object {
            Some(account_state_object) => Ok(account_state_object),
            None => {
                if create {
                    let account_state_object =
                        Arc::new(AccountStateObject::empty_account(self.store.clone()));
                    let mut cache = self.cache.lock();
                    cache.put(
                        *account_address,
                        CacheItem::new(account_state_object.clone()),
                    );
                    Ok(account_state_object)
                } else {
                    Err(AccountNotExist(*account_address).into())
                }
            }
        }
    }

    fn get_account_state_object_option(
        &self,
        account_address: &AccountAddress,
    ) -> Result<Option<Arc<AccountStateObject>>> {
        let mut cache = self.cache.lock();
        let item = cache.get(account_address);
        let object = match item {
            Some(item) => item.as_object(),
            None => {
                let object = self
                    .get_account_state(account_address)?
                    .map(|account_state| {
                        Arc::new(AccountStateObject::new(account_state, self.store.clone()))
                    });
                let cache_item = match &object {
                    Some(object) => CacheItem::new(object.clone()),
                    None => CacheItem::AccountNotExist(),
                };
                cache.put(*account_address, cache_item);
                object
            }
        };
        Ok(object)
    }

    fn get_account_state(&self, account_address: &AccountAddress) -> Result<Option<AccountState>> {
        self.state_tree
            .get(account_address)
            .and_then(|value| match value {
                Some(v) => Ok(Some(AccountState::decode(v.as_slice())?)),
                None => Ok(None),
            })
    }

    fn get_table_handle_state_object(
        &self,
        handle: &TableHandle,
    ) -> Result<Arc<TableHandleStateObject>> {
        let idx = handle.get_idx();
        let mut cache = self.cache_table_handle.lock();
        let item = cache.get(handle);
        let object = match item {
            Some(item) => item.clone(),
            None => {
                // XXX FIXME YSG
                let val = self
                    .state_tree_table_handles
                    .get(idx)
                    .unwrap()
                    .get(handle)?;
                let hash = match val {
                    Some(val) => HashValue::from_slice(val)?,
                    None => *SPARSE_MERKLE_PLACEHOLDER_HASH,
                };
                let obj = Arc::new(TableHandleStateObject::new(
                    *handle,
                    self.store.clone(),
                    hash,
                ));
                cache.put(*handle, obj.clone());
                obj
            }
        };
        Ok(object)
    }

    #[cfg(test)]
    fn table_handles_root_hash(&self, idx: usize) -> HashValue {
        use starcoin_vm_types::account_config::TABLE_ADDRESS_LIST_LEN;
        self.state_tree_table_handles
            .get(idx % TABLE_ADDRESS_LIST_LEN)
            .unwrap()
            .root_hash()
    }

    #[cfg(test)]
    fn table_handle_address_root_hash(&self, idx: usize) -> HashValue {
        // XXX FIXME YSG
        let handle_address = TABLE_HANDLE_ADDRESS_LIST
            .get(0)
            .expect("get TABLE_HANDLE_ADDRESS_LIST should always succeed");
        let table_path = TABLE_PATH_LIST
            .get(0)
            .expect("get TABLE_PATH_LIST should always succeed");
        // XXX FIXME YSG
        let account_state_object = self
            .get_account_state_object(handle_addres, true)
            .expect("get account state success");
        let state_root = account_state_object
            .get(table_path)
            .expect("get state_root success");
        HashValue::from_slice(state_root.unwrap()).unwrap()
    }
}

impl StateView for ChainStateDB {
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<Vec<u8>>> {
        match state_key {
            StateKey::AccessPath(access_path) => {
                let account_address = &access_path.address;
                let data_path = &access_path.path;
                self.get_account_state_object_option(account_address)
                    .and_then(|account_state| match account_state {
                        Some(account_state) => account_state.get(data_path),
                        None => Ok(None),
                    })
            }
            StateKey::TableItem(table_item) => {
                let table_handle_state_object =
                    self.get_table_handle_state_object(&table_item.handle)?;
                table_handle_state_object.get(&table_item.key)
            }
        }
    }

    fn is_genesis(&self) -> bool {
        self.state_tree.is_genesis()
    }
}

impl ChainStateReader for ChainStateDB {
    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof> {
        let account_address = &access_path.address;
        let data_path = &access_path.path;
        let (account_state, account_proof) = self.state_tree.get_with_proof(account_address)?;
        let account_state = account_state
            .map(|v| AccountState::decode(v.as_slice()))
            .transpose()?;
        let state_with_proof = match account_state {
            None => StateWithProof::new(
                None,
                StateProof::new(None, account_proof, SparseMerkleProof::default()),
            ),
            Some(account_state) => {
                let account_state_object = self.get_account_state_object(account_address, false)?;
                ensure!(
                    !account_state_object.is_dirty(),
                    "account {} has uncommitted modification",
                    &account_address
                );

                ensure!(
                    account_state == account_state_object.to_state(),
                    "global state tree is not synced with account {} state",
                    &account_address,
                );

                let (resource_value, resource_proof) =
                    account_state_object.get_with_proof(data_path)?;
                StateWithProof::new(
                    resource_value,
                    StateProof::new(Some(account_state.encode()?), account_proof, resource_proof),
                )
            }
        };
        Ok(state_with_proof)
    }

    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>> {
        Ok(self
            .get_account_state_object_option(address)?
            .map(|state_object| state_object.to_state()))
    }

    fn get_account_state_set(&self, address: &AccountAddress) -> Result<Option<AccountStateSet>> {
        self.get_account_state_object_option(address)?
            .map(|s| s.to_state_set())
            .transpose()
    }

    fn state_root(&self) -> HashValue {
        self.state_tree.root_hash()
    }

    // this interface on large records is slow, use dump_iter
    fn dump(&self) -> Result<ChainStateSet> {
        //TODO check cache dirty object.
        //TODO performance optimize.
        let global_states = self.state_tree.dump()?;
        let mut account_states = vec![];
        for (address_bytes, account_state_bytes) in global_states.iter() {
            let account_state: AccountState = account_state_bytes.as_slice().try_into()?;

            let mut state_sets = vec![];
            for (idx, storage_root) in account_state.storage_roots().iter().enumerate() {
                let state_set = match storage_root {
                    Some(storage_root) => {
                        let data_type = DataType::from_index(idx as u8)?;
                        match data_type {
                            DataType::CODE => {
                                Some(self.new_state_tree::<ModuleName>(*storage_root).dump()?)
                            }
                            DataType::RESOURCE => {
                                Some(self.new_state_tree::<StructTag>(*storage_root).dump()?)
                            }
                        }
                    }
                    None => None,
                };

                state_sets.push(state_set);
            }
            let account_state_set = AccountStateSet::new(state_sets);

            account_states.push((
                AccountAddress::decode_key(address_bytes.as_slice())?,
                account_state_set,
            ));
        }
        Ok(ChainStateSet::new(account_states))
    }

    // we have test 17514004 records cost 79598 seconds
    // the env is Intel(R) Xeon(R) Platinum 8275CL CPU @ 3.00GHz, 8 CPU and 16 GB memory
    // os is ubuntu 18.04.6 LTS
    // TODO future have 20 million records use increment export optimize
    fn dump_iter(&self) -> Result<AccountStateSetIterator> {
        let jmt_into_iter = self.state_tree.dump_iter()?;
        let iter = AccountStateSetIterator::new(self.store.clone(), jmt_into_iter);
        Ok(iter)
    }

    fn get_with_table_item_proof(
        &self,
        handle: &TableHandle,
        key: &[u8],
    ) -> Result<StateWithTableItemProof> {
        let idx = handle.get_idx();
        let handle_address = TABLE_HANDLE_ADDRESS_LIST
            .get(idx)
            .expect("get TABLE_HANDLE_ADDRESS_LIST should always succeed");
        let table_path = TABLE_PATH_LIST
            .get(idx)
            .expect("get TABLE_PATH_LIST should always succeed");
        let table_path_proof =
            self.get_with_proof(&AccessPath::new(handle_address.clone(), table_path.clone()))?;
        // XXX FIXME YSG
        let state_tree_table_handle = self.state_tree_table_handles.get(idx).unwrap();
        let table_handle_proof = state_tree_table_handle.get_with_proof(handle)?;
        let table_handle_state_object = self.get_table_handle_state_object(handle)?;
        let key_proof = table_handle_state_object.get_with_proof(&key.to_vec())?;
        Ok(StateWithTableItemProof::new(
            (table_path_proof, self.state_root()),
            (
                table_handle_proof.0,
                table_handle_proof.1,
                state_tree_table_handle.root_hash(),
            ),
            (
                key_proof.0,
                key_proof.1,
                table_handle_state_object.root_hash(),
            ),
        ))
    }
}

impl ChainStateWriter for ChainStateDB {
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()> {
        self.apply_write_set(
            WriteSetMut::new(vec![(
                StateKey::AccessPath(access_path.clone()),
                WriteOp::Value(value),
            )])
            .freeze()
            .expect("freeze write_set must success."),
        )
    }

    fn remove(&self, access_path: &AccessPath) -> Result<()> {
        self.apply_write_set(
            WriteSetMut::new(vec![(
                StateKey::AccessPath(access_path.clone()),
                WriteOp::Deletion,
            )])
            .freeze()
            .expect("freeze write_set must success."),
        )
    }

    fn apply(&self, chain_state_set: ChainStateSet) -> Result<()> {
        for (address, account_state_set) in chain_state_set.state_sets() {
            let code_root = if let Some(state_set) = account_state_set.code_set() {
                let state_tree = StateTree::<ModuleName>::new(self.store.clone(), None);
                state_tree.apply(state_set.clone())?;
                state_tree.flush()?;
                Some(state_tree.root_hash())
            } else {
                None
            };
            let resource_root = if let Some(state_set) = account_state_set.resource_set() {
                let state_tree = StateTree::<StructTag>::new(self.store.clone(), None);
                state_tree.apply(state_set.clone())?;
                state_tree.flush()?;
                state_tree.root_hash()
            } else {
                unreachable!("this should never happened")
            };
            let new_account_state = AccountState::new(code_root, resource_root);
            self.state_tree.put(*address, new_account_state.try_into()?);
        }
        self.state_tree.commit()?;
        self.state_tree.flush()?;
        Ok(())
    }

    fn apply_write_set(&self, write_set: WriteSet) -> Result<()> {
        let mut lock_table_handle = self.updates_table_handle.write();
        let mut locks = self.updates.write();
        for (state_key, write_op) in write_set {
            //update self updates record
            match state_key {
                StateKey::AccessPath(access_path) => {
                    locks.insert(access_path.address);
                    let (account_address, data_path) = access_path.into_inner();
                    match write_op {
                        WriteOp::Value(value) => {
                            let account_state_object =
                                self.get_account_state_object(&account_address, true)?;
                            account_state_object.set(data_path, value);
                        }
                        WriteOp::Deletion => {
                            let account_state_object =
                                self.get_account_state_object(&account_address, false)?;
                            account_state_object.remove(&data_path)?;
                        }
                    }
                }
                StateKey::TableItem(table_item) => {
                    debug!("{:?}", table_item);
                    lock_table_handle.insert(table_item.handle);
                    let table_handle_state_object =
                        self.get_table_handle_state_object(&table_item.handle)?;
                    match write_op {
                        WriteOp::Value(value) => {
                            table_handle_state_object.set(table_item.key, value);
                        }
                        WriteOp::Deletion => {
                            table_handle_state_object.remove(&table_item.key);
                        }
                    }
                }
            }
        }
        Ok(())
    }
    /// Commit
    fn commit(&self) -> Result<HashValue> {
        // cache commit
        for handle in self.updates_table_handle.read().iter() {
            let table_handle_state_object = self.get_table_handle_state_object(handle)?;
            table_handle_state_object.commit()?;
            let idx = handle.get_idx();
            self.update_table_handle_idx_list.lock().insert(idx);
            // put table_handle_state_object commit
            // XXX FIXME YSG
            self.state_tree_table_handles
                .get(idx)
                .expect("get state_tree_table_handles index should success")
                .put(*handle, table_handle_state_object.root_hash().to_vec());
        }
        for idx in self.update_table_handle_idx_list.lock().iter() {
            let state_tree_table_handle = self
                .state_tree_table_handles
                .get(*idx)
                .expect("get state_tree_table_handles index should success");
            state_tree_table_handle.commit()?;

            // update table_handle_address state
            // XXX FIXME YSG
            let handle_address = TABLE_HANDLE_ADDRESS_LIST
                .get(*idx)
                .expect("get TABLE_HANDLE_ADDRESS_LIST should always succeed");
            let table_path: &DataPath = TABLE_PATH_LIST
                .get(*idx)
                .expect("get TABLE_PATH_LIST should always succeed");
            // XXX FIXME YSG

            let mut locks = self.updates.write();
            locks.insert(handle_address.clone());
            let table_handle_account_state_object =
                self.get_account_state_object(handle_address, true)?;
            table_handle_account_state_object.set(
                table_path.clone(),
                state_tree_table_handle.root_hash().to_vec(),
            );
        }

        for address in self.updates.read().iter() {
            let account_state_object = self.get_account_state_object(address, false)?;
            let state = account_state_object.commit()?;
            self.state_tree.put(*address, state.try_into()?);
        }
        self.state_tree.commit()
    }

    /// flush data to db.
    fn flush(&self) -> Result<()> {
        //cache flush
        let mut locks_table_handle = self.updates_table_handle.write();
        for h in locks_table_handle.iter() {
            let table_handle_state_object = self.get_table_handle_state_object(h)?;
            table_handle_state_object.flush()?;
        }
        locks_table_handle.clear();

        // XXX FIXME YSG
        for idx in self.update_table_handle_idx_list.lock().iter() {
            let state_tree_table_handle = self
                .state_tree_table_handles
                .get(*idx)
                .expect("get state_tree_table_handles index should success");
            state_tree_table_handle.flush()?;
        }
        self.update_table_handle_idx_list.lock().clear();
        // XXX FIXME YSG

        let mut locks = self.updates.write();
        for address in locks.iter() {
            let account_state_object = self.get_account_state_object(address, false)?;
            account_state_object.flush()?;
        }
        locks.clear();

        // self tree flush
        self.state_tree.flush()
    }
}

/// represent TableHandleState in runtime memory.
/// TableHandle's SMT Save (table.key, table.value)
struct TableHandleStateObject {
    _handle: TableHandle,
    state_tree: Mutex<StateTree<Vec<u8>>>,
}

impl TableHandleStateObject {
    pub fn new(_handle: TableHandle, store: Arc<dyn StateNodeStore>, root: HashValue) -> Self {
        let state_tree = StateTree::<Vec<u8>>::new(store.clone(), Some(root));
        Self {
            _handle,
            state_tree: Mutex::new(state_tree),
        }
    }

    pub fn set(&self, key: Vec<u8>, value: Vec<u8>) {
        self.state_tree.lock().put(key, value)
    }

    pub fn remove(&self, key: &Vec<u8>) {
        self.state_tree.lock().remove(key)
    }

    pub fn commit(&self) -> Result<()> {
        let state_tree = self.state_tree.lock();
        if state_tree.is_dirty() {
            state_tree.commit()?;
        }
        Ok(())
    }

    pub fn flush(&self) -> Result<()> {
        self.state_tree.lock().flush()?;
        Ok(())
    }

    pub fn root_hash(&self) -> HashValue {
        self.state_tree.lock().root_hash()
    }

    pub fn get(&self, key: &Vec<u8>) -> Result<Option<Vec<u8>>> {
        self.state_tree.lock().get(key)
    }

    pub fn get_with_proof(&self, key: &Vec<u8>) -> Result<(Option<Vec<u8>>, SparseMerkleProof)> {
        self.state_tree.lock().get_with_proof(key)
    }
}

#[cfg(test)]
mod tests;
