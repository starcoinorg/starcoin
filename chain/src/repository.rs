// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use libra_crypto::HashValue;
use types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
    block::BlockNumber, change_set::ChangeSet, language::ModuleId, transaction::Version,
};

///  `Repository` is a trait that defines chain's global state store.
pub trait Repository {
    fn load_resource(&self, ap: &AccessPath) -> Result<Option<Vec<u8>>>;

    fn publish_resource(&mut self, ap: &AccessPath, value: Vec<u8>) -> Result<()>;

    fn load_module(&self, module: &ModuleId) -> Result<Option<Vec<u8>>>;

    fn publish_module(&mut self, module_id: ModuleId, module: Vec<u8>) -> Result<()>;

    fn get_account_state(&self, address: AccountAddress) -> Result<Option<AccountState>>;

    fn is_dirty(&self) -> bool;

    /// Write dirty state to storage.
    fn commit(&self) -> Result<()>;

    /// Convert dirty state to ChangeSet
    fn to_change_set(&self) -> Result<ChangeSet>;

    fn state_root(&self) -> HashValue;
}

pub struct DefaultRepository {}

impl DefaultRepository {
    pub fn new(state_root: HashValue) -> DefaultRepository {
        unimplemented!()
    }
}

impl Repository for DefaultRepository {
    fn load_resource(&self, ap: &AccessPath) -> Result<Option<Vec<u8>>, Error> {
        unimplemented!()
    }

    fn publish_resource(&mut self, ap: &AccessPath, value: Vec<u8>) -> Result<(), Error> {
        unimplemented!()
    }

    fn load_module(&self, module: &ModuleId) -> Result<Option<Vec<u8>>, Error> {
        unimplemented!()
    }

    fn publish_module(&mut self, module_id: ModuleId, module: Vec<u8>) -> Result<(), Error> {
        unimplemented!()
    }

    fn get_account_state(&self, address: AccountAddress) -> Result<Option<AccountState>, Error> {
        unimplemented!()
    }

    fn is_dirty(&self) -> bool {
        unimplemented!()
    }

    fn commit(&self) -> Result<(), Error> {
        unimplemented!()
    }

    fn to_change_set(&self) -> Result<ChangeSet, Error> {
        unimplemented!()
    }

    fn state_root(&self) -> HashValue {
        unimplemented!()
    }
}
