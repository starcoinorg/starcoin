// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use chain_state::ChainState;
use crypto::HashValue;
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_state::AccountState,
    language_storage::{ModuleId, StructTag},
};

use crypto::hash::CryptoHash;
use scs::SCSCodec;
use state_tree::SparseMerkleTree;

pub struct StarcoinChainState {
    state_tree: SparseMerkleTree,
}

impl StarcoinChainState {
    /// Commit and calculate new state root
    pub fn commit(&self) -> Result<HashValue> {
        //TODO
        Ok(self.state_tree.root_hash())
    }

    /// flush data to db.
    pub fn flush(&self) -> Result<()> {
        //TODO
        Ok(())
    }

    fn get_account_storage_tree(&self, storage_root: HashValue) -> SparseMerkleTree {
        unimplemented!()
    }

    fn get_code_storage_tree(&self, code_root: HashValue) -> SparseMerkleTree {
        unimplemented!()
    }
}

impl ChainState for StarcoinChainState {
    fn get_by_hash(
        &self,
        storage_root: HashValue,
        resource_key: HashValue,
    ) -> Result<Option<Vec<u8>>> {
        let storage_tree = self.get_account_storage_tree(storage_root);
        storage_tree.get(resource_key)
    }

    fn get_code(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>> {
        self.get_account_state(module_id.address())
            .and_then(|account_state| match account_state {
                Some(account_state) => {
                    let storage_tree = self.get_account_storage_tree(account_state.storage_root());
                    storage_tree.get(module_id.name_hash())
                }
                None => Ok(None),
            })
    }

    fn get_account_state(&self, address: AccountAddress) -> Result<Option<AccountState>> {
        self.state_tree
            .get(address.crypto_hash())
            .and_then(|value| match value {
                Some(v) => Ok(Some(AccountState::decode(v.as_slice())?)),
                None => Ok(None),
            })
    }

    fn is_genesis(&self) -> bool {
        unimplemented!()
    }

    fn state_root(&self) -> HashValue {
        unimplemented!()
    }

    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()> {
        unimplemented!()
    }

    fn set_at(
        &self,
        account_state: &AccountState,
        struct_tag: &StructTag,
        value: Vec<u8>,
    ) -> Result<()> {
        unimplemented!()
    }

    fn delete(&self, access_path: &AccessPath) -> Result<()> {
        unimplemented!()
    }

    fn delete_at(&self, account_state: &AccountState, struct_tag: &StructTag) -> Result<()> {
        unimplemented!()
    }

    fn set_code(&self, module_id: &ModuleId) -> Result<()> {
        unimplemented!()
    }
}
