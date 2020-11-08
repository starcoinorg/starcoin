// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account::Account;
use crate::account_storage::AccountStorage;

use parking_lot::RwLock;
use rand::prelude::*;
use starcoin_account_api::error::AccountError;
use starcoin_account_api::{AccountInfo, AccountPrivateKey, AccountResult};
use starcoin_crypto::ed25519::Ed25519PrivateKey;
use starcoin_crypto::{Uniform, ValidCryptoMaterial};
use starcoin_types::{
    account_address::AccountAddress,
    account_config::token_code::TokenCode,
    transaction::{RawUserTransaction, SignedUserTransaction},
};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Add;
use std::time::Duration;
use std::time::Instant;

/// Account manager
pub struct AccountManager {
    store: AccountStorage,
    key_cache: RwLock<PasswordCache>,
}

#[derive(Default, Debug, PartialEq, Eq)]
struct PasswordCache {
    cache: HashMap<AccountAddress, (Instant, String)>,
}
impl PasswordCache {
    pub fn cache_pass(&mut self, account: AccountAddress, pass: String, ttl: Instant) {
        self.cache.insert(account, (ttl, pass));
    }
    pub fn remove_pass(&mut self, account: &AccountAddress) {
        self.cache.remove(account);
    }
    pub fn get_pass(&mut self, account: &AccountAddress) -> Option<String> {
        match self.cache.remove(account) {
            None => None,
            Some((ttl, kp)) => {
                if Instant::now() < ttl {
                    self.cache.insert(*account, (ttl, kp));
                    self.cache.get(account).map(|t| t.1.to_string())
                } else {
                    None
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn clean_expired(&mut self) {
        let cur_instant = Instant::now();
        self.cache.retain(|_account, (ttl, _)| &cur_instant < ttl);
    }
}

impl AccountManager {
    pub fn new(storage: AccountStorage) -> AccountResult<Self> {
        let manager = Self {
            store: storage,
            key_cache: RwLock::new(PasswordCache::default()),
        };
        Ok(manager)
    }

    pub fn create_account(&self, password: &str) -> AccountResult<Account> {
        let private_key = gen_private_key();
        let private_key = AccountPrivateKey::Single(private_key);
        let address = private_key.public_key().derived_address();
        self.save_account(address, private_key, password.to_string())
    }

    pub fn unlock_account(
        &self,
        address: AccountAddress,
        password: &str,
        duration: Duration,
    ) -> AccountResult<()> {
        let _ = Account::load(address, password, self.store.clone())?
            .ok_or_else(|| AccountError::AccountNotExist(address))?;
        let ttl = std::time::Instant::now().add(duration);
        self.key_cache
            .write()
            .cache_pass(address, password.to_string(), ttl);
        Ok(())
    }

    pub fn lock_account(&self, address: AccountAddress) -> AccountResult<()> {
        self.key_cache.write().remove_pass(&address);
        Ok(())
    }

    pub fn import_account(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: &str,
    ) -> AccountResult<Account> {
        let private_key = AccountPrivateKey::try_from(private_key.as_slice())
            .map_err(|_| AccountError::InvalidPrivateKey)?;
        self.save_account(address, private_key, password.to_string())
    }

    fn save_account(
        &self,
        address: AccountAddress,
        private_key: AccountPrivateKey,
        password: String,
    ) -> AccountResult<Account> {
        if self.contains(&address)? {
            return Err(AccountError::AccountAlreadyExist(address));
        }
        let account = Account::create(private_key, Some(address), password, self.store.clone())?;
        self.store.add_address(*account.address())?;

        // if it's the first address, set it default.
        if self.store.list_addresses()?.len() == 1 {
            self.set_default_account(address)?;
        }
        Ok(account)
    }

    pub fn export_account(
        &self,
        address: AccountAddress,
        password: &str,
    ) -> AccountResult<Vec<u8>> {
        let account = Account::load(address, password, self.store.clone())?
            .ok_or_else(|| AccountError::AccountNotExist(address))?;
        Ok(account.private_key().to_bytes().to_vec())
    }

    pub fn contains(&self, address: &AccountAddress) -> AccountResult<bool> {
        self.store
            .contain_address(*address)
            .map_err(AccountError::StoreError)
    }

    pub fn default_account_info(&self) -> AccountResult<Option<AccountInfo>> {
        let default_address = self.store.default_address()?;
        let account_info = default_address
            .map(|a| self.account_info(a))
            .transpose()?
            .and_then(|a| a);
        Ok(account_info)
    }
    pub fn list_account_infos(&self) -> AccountResult<Vec<AccountInfo>> {
        let default_account = self.store.default_address()?;
        let mut res = vec![];
        for account in self.store.list_addresses()? {
            let pubkey = self.store.public_key(account)?;
            match pubkey {
                Some(p) => {
                    res.push(AccountInfo {
                        address: account,
                        is_default: default_account.filter(|a| a == &account).is_some(),
                        public_key: p,
                    });
                }
                None => {
                    continue;
                }
            }
        }
        Ok(res)
    }

    pub fn account_info(&self, address: AccountAddress) -> AccountResult<Option<AccountInfo>> {
        match self.store.public_key(address)? {
            Some(p) => {
                let default_account = self.store.default_address()?;
                Ok(Some(AccountInfo {
                    address,
                    is_default: default_account.filter(|a| a == &address).is_some(),
                    public_key: p,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn sign_txn(
        &self,
        signer_address: AccountAddress,
        raw_txn: RawUserTransaction,
    ) -> AccountResult<SignedUserTransaction> {
        let pass = self.key_cache.write().get_pass(&signer_address);
        match pass {
            None => Err(AccountError::AccountLocked(signer_address)),
            Some(p) => {
                let account = Account::load(signer_address, p.as_str(), self.store.clone())?
                    .ok_or_else(|| AccountError::AccountNotExist(signer_address))?;
                account
                    .sign_txn(raw_txn)
                    .map_err(AccountError::TransactionSignError)
            }
        }
    }

    pub fn set_default_account(&self, address: AccountAddress) -> AccountResult<()> {
        self.store
            .set_default_address(Some(address))
            .map_err(AccountError::StoreError)
    }

    /// remove wallet need user password.
    #[allow(unused)]
    pub fn delete_account(&self, address: AccountAddress, password: &str) -> AccountResult<()> {
        let account = Account::load(address, password, self.store.clone())?;
        match account {
            Some(account) => {
                self.key_cache.write().remove_pass(&address);
                account.destroy().map_err(AccountError::StoreError)
            }
            None => Ok(()),
        }
    }

    pub fn accepted_tokens(&self, address: AccountAddress) -> AccountResult<Vec<TokenCode>> {
        self.store
            .get_accepted_tokens(address)
            .map_err(AccountError::StoreError)
    }
}

pub(crate) fn gen_private_key() -> Ed25519PrivateKey {
    let mut seed_rng = rand::rngs::OsRng;
    let seed_buf: [u8; 32] = seed_rng.gen();
    let mut rng: StdRng = SeedableRng::from_seed(seed_buf);
    Ed25519PrivateKey::generate(&mut rng)
}
