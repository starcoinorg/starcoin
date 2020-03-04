// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Error, Result};
use crypto::hash::CryptoHash;
use crypto::HashValue;
use logger::prelude::*;
use scs::SCSCodec;
use state_tree::{StateNodeStore, StateTree};
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::Arc;
use traits::{ChainState, ChainStateReader, ChainStateWriter};
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{account_struct_tag, AccountResource},
    account_state::AccountState,
    byte_array::ByteArray,
    language_storage::{ModuleId, StructTag},
};

use core::num::FpCategory::Nan;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("the Account for key `{0}` is not exist")]
    AccountNotExist(AccountAddress),
}

struct AccountStateCacheItem {
    pub resource_tree: StateTree,
    pub code_tree: Option<StateTree>,
}

impl AccountStateCacheItem {
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

    fn get_account_state_cache(
        &self,
        account_address: &AccountAddress,
    ) -> Result<Option<AccountStateCacheItem>> {
        //TODO cache
        Ok(self
            .get_account_state(account_address)?
            .and_then(|account_state| {
                Some(AccountStateCacheItem {
                    resource_tree: self.new_state_tree(account_state.resource_root()),
                    code_tree: account_state
                        .code_root()
                        .map(|code_root| self.new_state_tree(code_root)),
                })
            }))
    }
}

impl ChainState for ChainStateDB {}

impl ChainStateReader for ChainStateDB {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        let (account_address, hash) = access_path.clone().into();
        self.get_account_state_cache(&account_address)
            .and_then(|account_state| match account_state {
                Some(account_state) => account_state.resource_tree.get(&hash),
                None => Ok(None),
            })
    }

    fn get_code(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>> {
        self.get_account_state_cache(&module_id.address())
            .and_then(|account_state| match account_state {
                Some(account_state) => match account_state.code_tree {
                    Some(code_tree) => code_tree.get(&module_id.name_hash()),
                    None => Ok(None),
                },
                None => Ok(None),
            })
    }

    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>> {
        self.state_tree
            .get(&address.crypto_hash())
            .and_then(|value| match value {
                Some(v) => Ok(Some(AccountState::decode(v.as_slice())?)),
                None => Ok(None),
            })
    }

    fn is_genesis(&self) -> bool {
        //TODO
        return false;
    }

    fn state_root(&self) -> HashValue {
        self.state_tree.root_hash()
    }
}

impl ChainStateWriter for ChainStateDB {
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()> {
        let (account_address, hash) = access_path.clone().into();
        let account_state_cache = self
            .get_account_state_cache(&account_address)?
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
            .get_account_state_cache(&account_address)?
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use state_tree::mock::MockStateNodeStore;

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
}
