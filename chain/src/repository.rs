// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use libra_crypto::HashValue;
use state_view::StateView;
use types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
    block::BlockNumber, change_set::ChangeSet, language::ModuleId, transaction::Version,
    write_set::WriteSet,
};

///  `Repository` is a trait that defines chain's global state store.
pub trait Repository: StateView {
    fn get_account_state(&self, address: AccountAddress) -> Result<Option<AccountState>>;

    fn is_dirty(&self) -> bool;

    /// return new state root after commit
    fn commit(&self, write_set: &WriteSet) -> Result<HashValue>;

    fn state_root(&self) -> HashValue;

    /// Write data to storage.
    fn flush(&self) -> Result<()>;
}

pub struct DefaultRepository {}

impl DefaultRepository {
    pub fn new(state_root: HashValue) -> DefaultRepository {
        unimplemented!()
    }
}

impl StateView for DefaultRepository {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>, Error> {
        unimplemented!()
    }

    fn multi_get(&self, access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>, Error> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        unimplemented!()
    }
}

impl Repository for DefaultRepository {
    fn get_account_state(&self, address: AccountAddress) -> Result<Option<AccountState>, Error> {
        unimplemented!()
    }

    fn is_dirty(&self) -> bool {
        unimplemented!()
    }

    fn commit(&self, write_set: &WriteSet) -> Result<HashValue, Error> {
        unimplemented!()
    }

    fn state_root(&self) -> HashValue {
        unimplemented!()
    }

    fn flush(&self) -> Result<(), Error> {
        unimplemented!()
    }
}
