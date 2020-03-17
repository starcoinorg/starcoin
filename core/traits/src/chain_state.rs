// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use crypto::HashValue;

use types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
    state_set::ChainStateSet,
};

pub trait ChainStateReader {
    /// Gets the state data for a single access path.
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>>;

    /// Gets state data for a list of access paths.
    fn multi_get(&self, access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        access_paths
            .iter()
            .map(|access_path| self.get(access_path))
            .collect()
    }

    /// Gets account state
    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>>;

    /// VM needs this method to know whether the current state view is for genesis state creation.
    fn is_genesis(&self) -> bool;

    /// Gets current state root.
    fn state_root(&self) -> HashValue;

    fn dump(&self) -> Result<ChainStateSet>;
}

pub trait ChainStateWriter {
    /// Sets state at access_path.
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()>;

    /// Remove state at access_path
    fn remove(&self, access_path: &AccessPath) -> Result<()>;

    fn create_account(&self, account_address: AccountAddress) -> Result<()>;

    /// Apply dump result to ChainState
    fn apply(&self, state_set: ChainStateSet) -> Result<()>;
}

/// `ChainState` is a trait that defines chain's global state.
pub trait ChainState: ChainStateReader + ChainStateWriter {}
