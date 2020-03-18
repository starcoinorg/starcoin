// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use scs::SCSCodec;
use starcoin_crypto::{hash::CryptoHash, HashValue};
use starcoin_logger::prelude::*;
use starcoin_state_tree::{StateNodeStore, StateTree};
use starcoin_traits::{ChainState, ChainStateReader, ChainStateWriter};
use starcoin_types::{
    access_path::{AccessPath, DataType},
    account_address::AccountAddress,
    account_config::{account_struct_tag, AccountResource},
    account_state::AccountState,
    byte_array::ByteArray,
    state_set::{AccountStateSet, ChainStateSet},
};
use std::cell::RefCell;
use std::collections::{hash_map::Entry, HashMap};
use std::convert::TryInto;
use std::sync::Arc;

use crate::StateError::AccountNotExist;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("the Account for key `{0}` is not exist")]
    AccountNotExist(AccountAddress),
}

/// represent AccountState in runtime memory.
struct AccountStateObject {
    address: AccountAddress,
    trees: RefCell<Vec<Option<StateTree>>>,
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
                Some(root) => Some(StateTree::new(store.clone(), Some(root.clone()))),
                None => None,
            })
            .collect();
        Self {
            address,
            trees: RefCell::new(trees),
            store,
        }
    }

    pub fn new_account(address: AccountAddress, store: Arc<dyn StateNodeStore>) -> Self {
        let mut trees = vec![None; DataType::LENGTH];
        trees[0] = Some(StateTree::new(store.clone(), None));
        Self {
            address,
            trees: RefCell::new(trees),
            store,
        }
    }

    pub fn get(&self, data_type: DataType, key_hash: &HashValue) -> Result<Option<Vec<u8>>> {
        match self.trees.borrow()[data_type.storage_index()].as_ref() {
            Some(tree) => tree.get(key_hash),
            None => Ok(None),
        }
    }

    pub fn set(&self, data_type: DataType, key_hash: HashValue, value: Vec<u8>) {
        let mut trees = self.trees.borrow_mut();
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
        let trees = self.trees.borrow();
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
        for tree in self.trees.borrow().iter() {
            if let Some(tree) = tree {
                if tree.is_dirty() {
                    return true;
                }
            }
        }
        false
    }

    pub fn commit(&self) -> Result<AccountState> {
        for tree in self.trees.borrow().iter() {
            if let Some(tree) = tree {
                if tree.is_dirty() {
                    tree.commit()?;
                }
            }
        }

        Ok(self.to_state())
    }

    pub fn flush(&self) -> Result<()> {
        for tree in self.trees.borrow().iter() {
            if let Some(tree) = tree {
                tree.flush()?;
            }
        }
        Ok(())
    }

    fn to_state(&self) -> AccountState {
        let storage_roots = self
            .trees
            .borrow()
            .iter()
            .map(|tree| match tree {
                Some(tree) => Some(tree.root_hash()),
                None => None,
            })
            .collect();

        AccountState::new(storage_roots)
    }
}

pub struct ChainStateDB {
    store: Arc<dyn StateNodeStore>,
    ///global state tree.
    state_tree: StateTree,
    cache: RefCell<HashMap<HashValue, Option<Arc<AccountStateObject>>>>,
}

impl ChainStateDB {
    // create empty chain state
    pub fn new(store: Arc<dyn StateNodeStore>, root_hash: Option<HashValue>) -> Self {
        Self {
            store: store.clone(),
            state_tree: StateTree::new(store, root_hash),
            cache: RefCell::new(HashMap::new()),
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
                    self.cache
                        .borrow_mut()
                        .insert(address_hash, Some(account_state_object.clone()));
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
        let mut cache = self.cache.borrow_mut();
        let entry = cache.entry(address_hash);
        let object = match entry {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let account_state_object =
                    self.get_account_state_by_hash(&address_hash)?
                        .and_then(|account_state| {
                            Some(Arc::new(AccountStateObject::new(
                                *account_address,
                                account_state,
                                self.store.clone(),
                            )))
                        });
                entry.insert(account_state_object.clone());
                account_state_object
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

impl ChainStateReader for ChainStateDB {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        let (account_address, data_type, hash) = access_path.clone().into();
        self.get_account_state_object_option(&account_address)
            .and_then(|account_state| match account_state {
                Some(account_state) => account_state.get(data_type, &hash),
                None => Ok(None),
            })
    }

    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>> {
        Ok(self
            .get_account_state_object_option(address)?
            .and_then(|state_object| Some(state_object.to_state())))
    }

    fn is_genesis(&self) -> bool {
        //TODO
        return false;
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

            account_states.push((address_hash.clone(), account_state_set));
        }
        Ok(ChainStateSet::new(account_states))
    }
}

impl ChainStateWriter for ChainStateDB {
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()> {
        let (account_address, data_type, key_hash) = access_path.clone().into();
        let account_state_object = self.get_account_state_object(&account_address, true)?;
        account_state_object.set(data_type, key_hash, value);
        Ok(())
    }

    fn remove(&self, access_path: &AccessPath) -> Result<()> {
        let (account_address, data_type, hash) = access_path.clone().into();
        let account_state_object = self.get_account_state_object(&account_address, false)?;
        account_state_object.remove(data_type, &hash)?;
        Ok(())
    }

    fn create_account(&self, account_address: AccountAddress) -> Result<()> {
        let account_state_object =
            AccountStateObject::new_account(account_address, self.store.clone());

        let account_resource = AccountResource::new(0, 0, ByteArray::new(account_address.to_vec()));
        debug!(
            "create account: {:?} with address: {:?}",
            account_resource, account_address
        );
        let struct_tag = account_struct_tag();
        account_state_object.set(
            DataType::RESOURCE,
            struct_tag.crypto_hash(),
            account_resource.try_into()?,
        );
        self.cache.borrow_mut().insert(
            account_address.crypto_hash(),
            Some(Arc::new(account_state_object)),
        );
        Ok(())
    }

    fn apply(&self, chain_state_set: ChainStateSet) -> Result<()> {
        for (address_hash, account_state_set) in chain_state_set.state_sets() {
            let account_state = self
                .get_account_state_by_hash(address_hash)?
                .unwrap_or(AccountState::default());
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
        for (address_hash, state_object) in self.cache.borrow().iter() {
            match state_object {
                Some(state_object) => {
                    if state_object.is_dirty() {
                        let account_state = state_object.commit()?;
                        self.state_tree
                            .put(*address_hash, account_state.try_into()?);
                    }
                }
                None => {}
            }
        }
        self.state_tree.commit()
    }

    /// flush data to db.
    fn flush(&self) -> Result<()> {
        //TODO optimize
        for (_address_hash, state_object) in self.cache.borrow().iter() {
            match state_object {
                Some(state_object) => {
                    state_object.flush()?;
                }
                None => {}
            }
        }
        self.state_tree.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_state_tree::mock::MockStateNodeStore;

    #[test]
    fn test_state_db() -> Result<()> {
        let storage = MockStateNodeStore::new();
        let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
        let account_address = AccountAddress::random();
        chain_state_db.create_account(account_address)?;
        let state_root = chain_state_db.commit()?;

        let access_path = AccessPath::new_for_account(account_address);
        let account_resource: AccountResource = chain_state_db
            .get(&access_path)?
            .expect("before create account must exist.")
            .try_into()?;
        assert_eq!(0, account_resource.balance(), "new account balance error");

        let new_account_resource =
            AccountResource::new(10, 1, account_resource.authentication_key().clone());
        chain_state_db.set(&access_path, new_account_resource.try_into()?)?;

        let account_resource2: AccountResource =
            chain_state_db.get(&access_path)?.unwrap().try_into()?;
        assert_eq!(10, account_resource2.balance());

        let new_state_root = chain_state_db.commit()?;
        assert_ne!(state_root, new_state_root);
        Ok(())
    }

    #[test]
    fn test_write_no_exist_account() -> Result<()> {
        let storage = MockStateNodeStore::new();
        let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
        let access_path = AccessPath::new(
            AccountAddress::random(),
            DataType::RESOURCE,
            HashValue::random(),
        );
        let data = vec![1u8, 2u8];
        chain_state_db.set(&access_path, data.clone())?;
        let data1 = chain_state_db.get(&access_path)?;
        assert_eq!(data1, Some(data));
        Ok(())
    }

    #[test]
    fn test_state_db_dump_and_apply() -> Result<()> {
        let storage = MockStateNodeStore::new();
        let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
        let account_address = AccountAddress::random();
        chain_state_db.create_account(account_address)?;

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
}
