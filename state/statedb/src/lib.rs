// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::StateError::AccountNotExist;
use anyhow::{bail, ensure, Result};
use lru::LruCache;
use merkle_tree::proof::SparseMerkleProof;
use parking_lot::{Mutex, MutexGuard};
use scs::SCSCodec;
use starcoin_crypto::{hash::PlainCryptoHash, HashValue};
use starcoin_logger::prelude::*;
use starcoin_state_api::{
    ChainState, ChainStateReader, ChainStateWriter, StateProof, StateWithProof,
};
use starcoin_state_tree::{StateNodeStore, StateTree};
use starcoin_types::{
    access_path::{self, AccessPath, DataType},
    account_address::AccountAddress,
    account_state::AccountState,
    state_set::{AccountStateSet, ChainStateSet},
};
use starcoin_vm_types::state_view::StateView;
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
    fn flush(&self) -> Result<()> {
        match self {
            CacheItem::AccountObject(obj) => obj.flush(),
            CacheItem::AccountNotExist() => Ok(()),
        }
    }
    fn is_dirty(&self) -> bool {
        match self {
            CacheItem::AccountObject(obj) => obj.is_dirty(),
            CacheItem::AccountNotExist() => false,
        }
    }
    fn commit(&self) -> Result<AccountState> {
        match self {
            CacheItem::AccountObject(obj) => obj.commit(),
            CacheItem::AccountNotExist() => unreachable!(),
        }
    }
}

/// represent AccountState in runtime memory.
struct AccountStateObject {
    address: AccountAddress,
    //TODO if use RefCell at here, compile error for ActorRef async interface
    // the trait `std::marker::Sync` is not implemented for AccountStateObject
    // refactor AccountStateObject to a readonly object.
    trees: Mutex<Vec<Option<StateTree>>>,
    store: Arc<dyn StateNodeStore>,
}

impl AccountStateObject {
    pub fn new(
        address: AccountAddress,
        account_state: AccountState,
        store: Arc<dyn StateNodeStore>,
    ) -> Self {
        let trees = account_state
            .storage_roots()
            .iter()
            .map(|root| match root {
                Some(root) => Some(StateTree::new(store.clone(), Some(*root))),
                None => None,
            })
            .collect();
        Self {
            address,
            trees: Mutex::new(trees),
            store,
        }
    }

    pub fn new_account(address: AccountAddress, store: Arc<dyn StateNodeStore>) -> Self {
        let mut trees = vec![None; DataType::LENGTH];
        trees[0] = Some(StateTree::new(store.clone(), None));
        Self {
            address,
            trees: Mutex::new(trees),
            store,
        }
    }

    pub fn get(&self, data_type: DataType, key_hash: &HashValue) -> Result<Option<Vec<u8>>> {
        let trees = self.trees.lock();
        match trees[data_type.storage_index()].as_ref() {
            Some(tree) => tree.get(key_hash),
            None => Ok(None),
        }
    }

    /// return value with it proof.
    /// NOTICE: Any un-committed modification will not visible to the method.
    pub fn get_with_proof(
        &self,
        data_type: DataType,
        key_hash: &HashValue,
    ) -> Result<(Option<Vec<u8>>, SparseMerkleProof)> {
        let trees = self.trees.lock();
        match trees[data_type.storage_index()].as_ref() {
            Some(tree) => tree.get_with_proof(key_hash),
            None => Ok((None, SparseMerkleProof::new(None, vec![]))),
        }
    }

    pub fn set(&self, data_type: DataType, key_hash: HashValue, value: Vec<u8>) {
        let mut trees = self.trees.lock();
        if trees[data_type.storage_index()].as_ref().is_none() {
            trees[data_type.storage_index()] = Some(StateTree::new(self.store.clone(), None));
        }
        let tree = trees[data_type.storage_index()]
            .as_ref()
            .expect("state tree must exist after set.");
        tree.put(key_hash, value);
    }

    pub fn remove(&self, data_type: DataType, key_hash: &HashValue) -> Result<()> {
        if data_type.is_code() {
            bail!("Not supported remove code currently.");
        }
        let trees = self.trees.lock();
        let tree = trees[data_type.storage_index()].as_ref();
        match tree {
            Some(tree) => tree.remove(key_hash),
            None => bail!(
                "Can not find storage root fro data_type {:?} at: {:?}",
                data_type,
                self.address
            ),
        }
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        let trees = self.trees.lock();
        for tree in trees.iter() {
            if let Some(tree) = tree {
                if tree.is_dirty() {
                    return true;
                }
            }
        }
        false
    }

    pub fn commit(&self) -> Result<AccountState> {
        let trees = self.trees.lock();
        for tree in trees.iter() {
            if let Some(tree) = tree {
                if tree.is_dirty() {
                    tree.commit()?;
                }
            }
        }

        Ok(Self::build_state(trees))
    }

    pub fn flush(&self) -> Result<()> {
        let trees = self.trees.lock();
        for tree in trees.iter() {
            if let Some(tree) = tree {
                tree.flush()?;
            }
        }
        Ok(())
    }

    fn build_state(trees: MutexGuard<Vec<Option<StateTree>>>) -> AccountState {
        let storage_roots = trees
            .iter()
            .map(|tree| match tree {
                Some(tree) => Some(tree.root_hash()),
                None => None,
            })
            .collect();

        AccountState::new(storage_roots)
    }

    fn to_state(&self) -> AccountState {
        let trees = self.trees.lock();
        Self::build_state(trees)
    }
}

pub struct ChainStateDB {
    store: Arc<dyn StateNodeStore>,
    ///global state tree.
    state_tree: StateTree,
    cache: Mutex<LruCache<HashValue, CacheItem>>,
}

static DEFAULT_CACHE_SIZE: usize = 10240;

impl ChainStateDB {
    pub fn new(store: Arc<dyn StateNodeStore>, root_hash: Option<HashValue>) -> Self {
        Self {
            store: store.clone(),
            state_tree: StateTree::new(store, root_hash),
            cache: Mutex::new(LruCache::new(DEFAULT_CACHE_SIZE)),
        }
    }

    //TODO implements a change_root ChainStateReader
    pub fn change_root(&self, root_hash: HashValue) -> Self {
        Self {
            store: self.store.clone(),
            state_tree: StateTree::new(self.store.clone(), Some(root_hash)),
            cache: Mutex::new(LruCache::new(DEFAULT_CACHE_SIZE)),
        }
    }

    fn new_state_tree(&self, root_hash: HashValue) -> StateTree {
        StateTree::new(self.store.clone(), Some(root_hash))
    }

    fn get_account_state_object(
        &self,
        account_address: &AccountAddress,
        create: bool,
    ) -> Result<Arc<AccountStateObject>> {
        let account_state_object = self.get_account_state_object_option(&account_address)?;
        match account_state_object {
            Some(account_state_object) => Ok(account_state_object),
            None => {
                if create {
                    let account_state_object = Arc::new(AccountStateObject::new_account(
                        *account_address,
                        self.store.clone(),
                    ));
                    let address_hash = account_address.crypto_hash();
                    let mut cache = self.cache.lock();
                    cache.put(address_hash, CacheItem::new(account_state_object.clone()));
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
        let address_hash = account_address.crypto_hash();
        let mut cache = self.cache.lock();
        let item = cache.get(&address_hash);
        let object = match item {
            Some(item) => item.as_object(),
            None => {
                let object = self
                    .get_account_state_by_hash(&address_hash)?
                    .map(|account_state| {
                        Arc::new(AccountStateObject::new(
                            *account_address,
                            account_state,
                            self.store.clone(),
                        ))
                    });
                let cache_item = match &object {
                    Some(object) => CacheItem::new(object.clone()),
                    None => CacheItem::AccountNotExist(),
                };
                cache.put(address_hash, cache_item);
                object
            }
        };
        Ok(object)
    }

    fn get_account_state_by_hash(&self, address_hash: &HashValue) -> Result<Option<AccountState>> {
        self.state_tree
            .get(address_hash)
            .and_then(|value| match value {
                Some(v) => Ok(Some(AccountState::decode(v.as_slice())?)),
                None => Ok(None),
            })
    }
}

impl ChainState for ChainStateDB {}

impl StateView for ChainStateDB {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        let (account_address, data_type, hash) = access_path::into_inner(access_path.clone())?;
        self.get_account_state_object_option(&account_address)
            .and_then(|account_state| match account_state {
                Some(account_state) => account_state.get(data_type, &hash),
                None => Ok(None),
            })
    }

    /// Gets state data for a list of access paths.
    fn multi_get(&self, access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        access_paths
            .iter()
            .map(|access_path| self.get(access_path))
            .collect()
    }

    fn is_genesis(&self) -> bool {
        //TODO
        false
    }
}

impl ChainStateReader for ChainStateDB {
    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof> {
        let (account_address, data_type, hash) = access_path::into_inner(access_path.clone())?;
        let address_hash = account_address.crypto_hash();
        let (account_state, account_proof) = self.state_tree.get_with_proof(&address_hash)?;
        let account_state = account_state
            .map(|v| AccountState::decode(v.as_slice()))
            .transpose()?;
        let state_with_proof = match account_state {
            None => StateWithProof::new(
                None,
                StateProof::new(None, account_proof, SparseMerkleProof::default()),
            ),
            Some(account_state) => {
                let account_state_object =
                    self.get_account_state_object(&account_address, false)?;
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
                    account_state_object.get_with_proof(data_type, &hash)?;
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

    fn state_root(&self) -> HashValue {
        self.state_tree.root_hash()
    }

    fn dump(&self) -> Result<ChainStateSet> {
        //TODO check cache dirty object.
        //TODO performance optimize.
        let global_states = self.state_tree.dump()?;
        let mut account_states = vec![];
        for (address_hash, account_state_bytes) in global_states.iter() {
            let account_state: AccountState = account_state_bytes.as_slice().try_into()?;

            let mut state_sets = vec![];
            for storage_root in account_state.storage_roots().iter() {
                let state_set = match storage_root {
                    Some(storage_root) => Some(self.new_state_tree(storage_root.clone()).dump()?),
                    None => None,
                };

                state_sets.push(state_set);
            }
            let account_state_set = AccountStateSet::new(state_sets);

            account_states.push((*address_hash, account_state_set));
        }
        Ok(ChainStateSet::new(account_states))
    }
}

impl ChainStateWriter for ChainStateDB {
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()> {
        let (account_address, data_type, key_hash) = access_path::into_inner(access_path.clone())?;
        let account_state_object = self.get_account_state_object(&account_address, true)?;
        account_state_object.set(data_type, key_hash, value);
        Ok(())
    }

    fn remove(&self, access_path: &AccessPath) -> Result<()> {
        let (account_address, data_type, hash) = access_path::into_inner(access_path.clone())?;
        let account_state_object = self.get_account_state_object(&account_address, false)?;
        account_state_object.remove(data_type, &hash)?;
        Ok(())
    }

    fn apply(&self, chain_state_set: ChainStateSet) -> Result<()> {
        for (address_hash, account_state_set) in chain_state_set.state_sets() {
            let account_state = self
                .get_account_state_by_hash(address_hash)?
                .unwrap_or_default();
            let mut new_storage_roots = vec![];
            for (storage_root, state_set) in account_state
                .storage_roots()
                .iter()
                .zip(account_state_set.into_iter())
            {
                let new_storage_root = match (storage_root, state_set) {
                    (Some(storage_root), Some(state_set)) => {
                        let state_tree = self.new_state_tree(*storage_root);
                        state_tree.apply(state_set.clone())?;
                        state_tree.flush()?;
                        Some(state_tree.root_hash())
                    }
                    (Some(storage_root), None) => Some(*storage_root),
                    (None, Some(state_set)) => {
                        let state_tree = StateTree::new(self.store.clone(), None);
                        state_tree.apply(state_set.clone())?;
                        state_tree.flush()?;
                        Some(state_tree.root_hash())
                    }
                    (None, None) => None,
                };
                new_storage_roots.push(new_storage_root);
            }

            let new_account_state = AccountState::new(new_storage_roots);
            self.state_tree
                .put(address_hash.clone(), new_account_state.try_into()?);
        }
        self.state_tree.commit()?;
        self.state_tree.flush()?;
        Ok(())
    }
    /// Commit
    fn commit(&self) -> Result<HashValue> {
        //TODO optimize
        for (address_hash, state_object) in self.cache.lock().iter() {
            if state_object.is_dirty() {
                let account_state = state_object.commit()?;
                self.state_tree
                    .put(*address_hash, account_state.try_into()?);
            }
        }
        self.state_tree.commit()
    }

    /// flush data to db.
    fn flush(&self) -> Result<()> {
        //TODO optimize
        for (_address_hash, state_object) in self.cache.lock().iter() {
            state_object.flush()?;
        }
        self.state_tree.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_state_tree::mock::MockStateNodeStore;

    fn random_bytes() -> Vec<u8> {
        HashValue::random().to_vec()
    }

    #[test]
    fn test_state_proof() -> Result<()> {
        let storage = MockStateNodeStore::new();
        let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
        let access_path = access_path::random_resource();
        let state0 = random_bytes();
        chain_state_db.set(&access_path, state0.clone())?;

        let state_root = chain_state_db.commit()?;
        let state1 = chain_state_db.get(&access_path)?;
        assert!(state1.is_some());
        assert_eq!(state0, state1.unwrap());
        let state_with_proof = chain_state_db.get_with_proof(&access_path)?;
        state_with_proof.proof.verify(
            state_root,
            access_path,
            state_with_proof.state.as_deref(),
        )?;
        Ok(())
    }

    #[test]
    fn test_state_db() -> Result<()> {
        let storage = MockStateNodeStore::new();
        let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
        let access_path = access_path::random_resource();

        let state0 = random_bytes();
        chain_state_db.set(&access_path, state0)?;
        let state_root = chain_state_db.commit()?;

        let state1 = random_bytes();
        chain_state_db.set(&access_path, state1)?;

        let new_state_root = chain_state_db.commit()?;
        assert_ne!(state_root, new_state_root);
        Ok(())
    }

    #[test]
    fn test_state_db_dump_and_apply() -> Result<()> {
        let storage = MockStateNodeStore::new();
        let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
        let access_path = access_path::random_resource();
        let state0 = random_bytes();
        chain_state_db.set(&access_path, state0)?;
        chain_state_db.commit()?;
        chain_state_db.flush()?;

        let global_state = chain_state_db.dump()?;
        assert_eq!(
            global_state.state_sets().len(),
            1,
            "unexpect state_set length."
        );

        let storage2 = MockStateNodeStore::new();
        let chain_state_db2 = ChainStateDB::new(Arc::new(storage2), None);
        chain_state_db2.apply(global_state.clone())?;
        let global_state2 = chain_state_db2.dump()?;
        assert_eq!(global_state2.state_sets().len(), 1);
        assert_eq!(global_state, global_state2);

        Ok(())
    }

    #[test]
    fn test_state_version() -> Result<()> {
        let storage = Arc::new(MockStateNodeStore::new());
        let chain_state_db = ChainStateDB::new(storage.clone(), None);
        let account_address = AccountAddress::random();
        let access_path = AccessPath::new_for_account(account_address);
        let old_state = random_bytes();
        chain_state_db.set(&access_path, old_state.clone())?;

        chain_state_db.commit()?;
        chain_state_db.flush()?;
        let old_root = chain_state_db.state_root();

        let new_state = random_bytes();
        chain_state_db.set(&access_path, new_state)?;

        let chain_state_db_ori = ChainStateDB::new(storage, Some(old_root));
        let old_state2 = chain_state_db_ori.get(&access_path)?.unwrap();
        assert_eq!(old_state, old_state2);

        Ok(())
    }
}
