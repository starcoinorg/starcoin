// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Error, Result};
use scs::SCSCodec;
use starcoin_crypto::{hash::CryptoHash, HashValue};
use starcoin_logger::prelude::*;
use starcoin_state_tree::{StateNodeStore, StateTree};
use starcoin_traits::{ChainState, ChainStateReader, ChainStateWriter};
use starcoin_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{account_struct_tag, AccountResource},
    account_state::AccountState,
    byte_array::ByteArray,
    language_storage::{ModuleId, StructTag},
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::Arc;

use core::num::FpCategory::Nan;
use starcoin_types::state_set::{AccountStateSet, ChainStateSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("the Account for key `{0}` is not exist")]
    AccountNotExist(AccountAddress),
}

/// represent AccountState in runtime memory.
struct AccountStateObject {
    pub resource_tree: StateTree,
    pub code_tree: Option<StateTree>,
}

impl AccountStateObject {
    pub fn get_state(&self) -> AccountState {
        AccountState::new(
            self.code_tree.as_ref().map(|tree| tree.root_hash()),
            self.resource_tree.root_hash(),
        )
    }
}

pub struct ChainStateDB {
    store: Arc<dyn StateNodeStore>,
    ///global state tree.
    state_tree: StateTree,
}

impl ChainStateDB {
    // create empty chain state
    pub fn new(store: Arc<dyn StateNodeStore>, root_hash: Option<HashValue>) -> Self {
        Self {
            store: store.clone(),
            state_tree: StateTree::new(store, root_hash),
        }
    }

    /// Commit
    pub fn commit(&self) -> Result<HashValue> {
        //TODO
        self.state_tree.commit()?;
        Ok(self.state_tree.root_hash())
    }

    /// flush data to db.
    pub fn flush(&self) -> Result<()> {
        //TODO
        Ok(())
    }

    fn new_state_tree(&self, root_hash: HashValue) -> StateTree {
        //TODO cache
        StateTree::new(self.store.clone(), Some(root_hash))
    }

    fn new_empty_state_tree(&self) -> StateTree {
        StateTree::new(self.store.clone(), None)
    }

    fn update_account(
        &self,
        account_address: AccountAddress,
        account_state: AccountState,
    ) -> Result<HashValue> {
        self.state_tree
            .put(account_address.crypto_hash(), account_state.try_into()?)
    }

    fn get_account_state_object(
        &self,
        account_address: &AccountAddress,
    ) -> Result<Option<AccountStateObject>> {
        //TODO cache
        Ok(self
            .get_account_state(account_address)?
            .and_then(|account_state| {
                Some(AccountStateObject {
                    resource_tree: self.new_state_tree(account_state.resource_root()),
                    code_tree: account_state
                        .code_root()
                        .map(|code_root| self.new_state_tree(code_root)),
                })
            }))
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
        let (account_address, hash) = access_path.clone().into();
        self.get_account_state_object(&account_address).and_then(
            |account_state| match account_state {
                Some(account_state) => account_state.resource_tree.get(&hash),
                None => Ok(None),
            },
        )
    }

    fn get_code(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>> {
        self.get_account_state_object(&module_id.address())
            .and_then(|account_state| match account_state {
                Some(account_state) => match account_state.code_tree {
                    Some(code_tree) => code_tree.get(&module_id.name_hash()),
                    None => Ok(None),
                },
                None => Ok(None),
            })
    }

    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>> {
        self.get_account_state_by_hash(&address.crypto_hash())
    }

    fn is_genesis(&self) -> bool {
        //TODO
        return false;
    }

    fn state_root(&self) -> HashValue {
        self.state_tree.root_hash()
    }

    fn dump(&self) -> Result<ChainStateSet> {
        //TODO performance optimize.
        let global_states = self.state_tree.dump()?;
        let mut states = vec![];
        for (address_hash, account_state_bytes) in global_states.iter() {
            let account_state: AccountState = account_state_bytes.as_slice().try_into()?;
            let code_set = match account_state.code_root() {
                Some(root) => Some(self.new_state_tree(root).dump()?),
                None => None,
            };
            let resource_set = self.new_state_tree(account_state.resource_root()).dump()?;
            let account_state_set = AccountStateSet::new(code_set, Some(resource_set));
            states.push((address_hash.clone(), account_state_set));
        }
        Ok(ChainStateSet::new(states))
    }
}

impl ChainStateWriter for ChainStateDB {
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()> {
        let (account_address, hash) = access_path.clone().into();
        let account_state_cache = self
            .get_account_state_object(&account_address)?
            .ok_or(StateError::AccountNotExist(account_address))?;
        account_state_cache.resource_tree.put(hash, value)?;
        //TODO optimize.
        account_state_cache.resource_tree.commit()?;
        let account_state = account_state_cache.get_state();
        self.update_account(account_address, account_state)?;
        Ok(())
    }

    fn delete(&self, access_path: &AccessPath) -> Result<()> {
        unimplemented!()
    }

    fn delete_at(&self, account_state: &AccountState, struct_tag: &StructTag) -> Result<()> {
        unimplemented!()
    }

    fn set_code(&self, module_id: &ModuleId, code: Vec<u8>) -> Result<()> {
        let (account_address, name) = module_id.into_inner();
        let mut account_state_cache = self
            .get_account_state_object(&account_address)?
            .ok_or(StateError::AccountNotExist(account_address))?;

        if account_state_cache.code_tree.is_none() {
            let code_tree = self.new_empty_state_tree();
            account_state_cache.code_tree = Some(code_tree);
        }
        account_state_cache
            .code_tree
            .as_ref()
            .unwrap()
            .put(module_id.crypto_hash(), code);
        //TODO optimize.
        account_state_cache.resource_tree.commit()?;
        let account_state = account_state_cache.get_state();
        self.update_account(account_address, account_state)?;
        Ok(())
    }

    fn create_account(&self, account_address: AccountAddress) -> Result<()> {
        let state_tree = StateTree::new(self.store.clone(), None);
        let account_resource = AccountResource::new(0, 0, ByteArray::new(account_address.to_vec()));
        debug!(
            "create account: {:?} with address: {:?}",
            account_resource, account_address
        );
        let struct_tag = account_struct_tag();
        let resource_root =
            state_tree.put(struct_tag.crypto_hash(), account_resource.try_into()?)?;
        state_tree.commit()?;
        let account_state = AccountState::new(None, resource_root);
        let new_root = self.update_account(account_address, account_state);
        debug!("new state root: {:?} after create account.", new_root);
        Ok(())
    }

    fn apply(&self, state_set: ChainStateSet) -> Result<()> {
        for (address_hash, account_state_set) in state_set.state_sets() {
            let account_state = self.get_account_state_by_hash(address_hash)?;

            let new_resource_root = match (
                account_state
                    .as_ref()
                    .map(|account_state| account_state.resource_root()),
                account_state_set.resource_set(),
            ) {
                (Some(root), Some(state_set)) => {
                    let resource_tree = self.new_state_tree(root);
                    resource_tree.apply(state_set.clone())?;
                    resource_tree.commit()?;
                    resource_tree.root_hash()
                }
                (Some(root), None) => root,
                (None, Some(state_set)) => {
                    let resource_tree = StateTree::new(self.store.clone(), None);
                    resource_tree.apply(state_set.clone())?;
                    resource_tree.commit()?;
                    resource_tree.root_hash()
                }
                (None, None) => bail!(
                    "Invalid GlobalStateSet, can not find account_state by address hash: {:?}",
                    address_hash
                ),
            };

            let new_code_root = match (
                account_state
                    .as_ref()
                    .and_then(|account_state| account_state.code_root()),
                account_state_set.code_set(),
            ) {
                (Some(root), Some(state_set)) => {
                    let code_tree = self.new_state_tree(root);
                    code_tree.apply(state_set.clone())?;
                    code_tree.commit()?;
                    Some(code_tree.root_hash())
                }
                (Some(root), None) => Some(root),
                (None, Some(state_set)) => {
                    let code_tree = StateTree::new(self.store.clone(), None);
                    code_tree.apply(state_set.clone())?;
                    code_tree.commit()?;
                    Some(code_tree.root_hash())
                }
                (None, None) => None,
            };

            let new_account_state = AccountState::new(new_code_root, new_resource_root);
            self.state_tree
                .put(address_hash.clone(), new_account_state.try_into()?);
        }
        self.state_tree.commit()?;
        Ok(())
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
        chain_state_db.create_account(account_address);
        let access_path = AccessPath::new_for_account(account_address);
        let account_resource: AccountResource =
            chain_state_db.get(&access_path)?.unwrap().try_into()?;
        assert_eq!(0, account_resource.balance(), "balance error");
        let new_account_resource =
            AccountResource::new(10, 1, account_resource.authentication_key().clone());
        chain_state_db.set(&access_path, new_account_resource.try_into()?);

        let account_resource2: AccountResource =
            chain_state_db.get(&access_path)?.unwrap().try_into()?;
        assert_eq!(10, account_resource2.balance());
        Ok(())
    }

    #[test]
    fn test_state_db_dump_and_apply() -> Result<()> {
        let storage = MockStateNodeStore::new();
        let chain_state_db = ChainStateDB::new(Arc::new(storage), None);
        let account_address = AccountAddress::random();
        chain_state_db.create_account(account_address);
        let global_state = chain_state_db.dump()?;
        assert_eq!(global_state.state_sets().len(), 1);
        let storage2 = MockStateNodeStore::new();
        let chain_state_db2 = ChainStateDB::new(Arc::new(storage2), None);
        chain_state_db2.apply(global_state.clone())?;
        // let global_state2 = chain_state_db2.dump()?;
        // assert_eq!(global_state2.state_sets().len(), 1);
        // assert_eq!(global_state, global_state2);

        Ok(())
    }
}
