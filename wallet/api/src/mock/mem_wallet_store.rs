// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{WalletAccount, WalletStore};
use anyhow::{format_err, Result};
use starcoin_types::account_address::AccountAddress;
use std::collections::{hash_map::Entry, HashMap};
use std::sync::Mutex;

struct WalletAccountObject {
    account: WalletAccount,
    properties: HashMap<String, Vec<u8>>,
}

impl WalletAccountObject {
    pub fn new(account: WalletAccount) -> Self {
        Self {
            account,
            properties: HashMap::new(),
        }
    }
}

pub struct MemWalletStore {
    store: Mutex<HashMap<AccountAddress, WalletAccountObject>>,
}

impl MemWalletStore {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }
}

impl WalletStore for MemWalletStore {
    fn get_account(&self, address: &AccountAddress) -> Result<Option<WalletAccount>> {
        let store = self.store.lock().unwrap();
        Ok(store.get(address).map(|object| object.account.clone()))
    }

    fn save_account(&self, account: WalletAccount) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        match store.entry(account.address) {
            Entry::Vacant(entry) => {
                entry.insert(WalletAccountObject::new(account));
            }
            Entry::Occupied(mut entry) => {
                let mut object = entry.get_mut();
                object.account = account;
            }
        };
        Ok(())
    }

    fn remove_account(&self, address: &AccountAddress) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        store.remove(address);
        Ok(())
    }

    fn get_accounts(&self) -> Result<Vec<WalletAccount>> {
        let store = self.store.lock().unwrap();
        Ok(store.iter().map(|(_, v)| v.account.clone()).collect())
    }

    fn save_to_account(&self, address: &AccountAddress, key: String, value: Vec<u8>) -> Result<()> {
        let mut store = self.store.lock().unwrap();
        let account_object = store
            .get_mut(address)
            .ok_or(format_err!("Can not find account by address:{:?}", address))?;
        account_object.properties.insert(key, value);
        Ok(())
    }

    fn get_from_account(&self, address: &AccountAddress, key: &str) -> Result<Option<Vec<u8>>> {
        let mut store = self.store.lock().unwrap();
        Ok(store
            .get_mut(address)
            .and_then(|object| object.properties.get(key).cloned()))
    }
}
