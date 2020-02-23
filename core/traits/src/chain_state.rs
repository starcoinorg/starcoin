// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use crypto::{hash::CryptoHash, HashValue};
use std::collections::HashMap;
use std::sync::Arc;
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_state::AccountState,
    language_storage::{ModuleId, StructTag},
    transaction::{Transaction, TransactionInfo},
};

pub trait ChainStateReader {
    /// Gets the state for a single access path.
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        self.get_account_state(access_path.address)
            .and_then(|account_state| match account_state {
                Some(account_state) => self.get_by_hash(
                    account_state.storage_root(),
                    HashValue::from_slice(access_path.path.as_slice())?,
                ),
                None => Ok(None),
            })
    }

    /// Gets states for a list of access paths.
    fn multi_get(&self, access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        access_paths
            .iter()
            .map(|access_path| self.get(access_path))
            .collect()
    }

    /// Gets states at account_state's storage_root.
    fn get_at(
        &self,
        account_state: &AccountState,
        struct_tag: &StructTag,
    ) -> Result<Option<Vec<u8>>> {
        self.get_by_hash(account_state.storage_root(), struct_tag.crypto_hash())
    }

    /// Gets states at with account storage_root and hash.
    fn get_by_hash(
        &self,
        storage_root: HashValue,
        resource_key: HashValue,
    ) -> Result<Option<Vec<u8>>>;

    /// Gets Move module by id.
    fn get_code(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>>;

    /// Gets account state
    fn get_account_state(&self, address: AccountAddress) -> Result<Option<AccountState>>;

    /// VM needs this method to know whether the current state view is for genesis state creation.
    fn is_genesis(&self) -> bool;

    /// Gets current state root.
    fn state_root(&self) -> HashValue;
}

pub trait ChainStateWriter {
    /// Sets state at access_path.
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()>;

    fn set_at(
        &self,
        account_state: &AccountState,
        struct_tag: &StructTag,
        value: Vec<u8>,
    ) -> Result<()>;

    /// Delete state at access_path
    fn delete(&self, access_path: &AccessPath) -> Result<()>;

    fn delete_at(&self, account_state: &AccountState, struct_tag: &StructTag) -> Result<()>;

    fn set_code(&self, module_id: &ModuleId) -> Result<()>;
}

/// `ChainState` is a trait that defines chain's global state.
pub trait ChainState: ChainStateReader + ChainStateWriter {}
