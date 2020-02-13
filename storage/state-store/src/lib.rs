// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate defines [`trait StateView`](StateView).

use anyhow::Result;
use crypto::HashValue;
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_state::AccountState,
    language_storage::{ModuleId, StructTag},
};

/// `StateView` is a trait that defines a read-only snapshot of the global state.
pub trait StateView {
    /// Gets the state for a single access path.
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>>;

    /// Gets states for a list of access paths.
    fn multi_get(&self, access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>>;

    /// Gets states at account_state's storage_root.
    fn get_at(
        &self,
        account_state: &AccountState,
        struct_tag: &StructTag,
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

/// `StateStore` s a trait that defines chain's global state store.
pub trait StateStore: StateView {
    fn is_dirty(&self) -> bool;

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

    /// Write state to state tree and calculate new root. return new state root after commit
    fn commit(&self) -> Result<HashValue>;

    fn state_root(&self) -> HashValue;

    /// Write data to storage.
    fn flush(&self) -> Result<()>;
}
