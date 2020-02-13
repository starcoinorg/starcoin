// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use chain_state::ChainState;
use crypto::HashValue;
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_state::AccountState,
    language_storage::{ModuleId, StructTag},
};

use crypto::hash::CryptoHash;
use starcoin_canonical_serialization::SCSCodec;
use state_tree::SparseMerkleTree;

pub struct StarcoinChainState {
    state_tree: SparseMerkleTree,
}

impl StarcoinChainState {
    /// Commit and calculate new state root
    pub fn commit(&self) -> Result<HashValue> {
        unimplemented!()
    }

    /// flush data to db.
    pub fn flush(&self) -> Result<()> {
        unimplemented!()
    }
}

impl ChainState for StarcoinChainState {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        unimplemented!()
    }

    fn multi_get(&self, access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn get_at(
        &self,
        account_state: &AccountState,
        struct_tag: &StructTag,
    ) -> Result<Option<Vec<u8>>> {
        unimplemented!()
    }

    fn get_code(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>> {
        unimplemented!()
    }

    fn get_account_state(&self, address: AccountAddress) -> Result<Option<AccountState>> {
        self.state_tree
            .get(address.crypto_hash())?
            .map(|value| AccountState::decode(value.as_slice()))
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
