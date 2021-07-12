// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account::Account;
use crate::account_storage::AccountStorage;
use anyhow::format_err;
use parking_lot::RwLock;
use rand::prelude::*;
use starcoin_account_api::error::AccountError;
use starcoin_account_api::{AccountInfo, AccountPrivateKey, AccountPublicKey, AccountResult};
use starcoin_crypto::ed25519::Ed25519PrivateKey;
use starcoin_crypto::{Uniform, ValidCryptoMaterial};
use starcoin_logger::prelude::*;
use starcoin_types::genesis_config::ChainId;
use starcoin_types::sign_message::{SignedMessage, SigningMessage};
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
    chain_id: ChainId,
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
    pub fn new(storage: AccountStorage, chain_id: ChainId) -> AccountResult<Self> {
        let manager = Self {
            store: storage,
            key_cache: RwLock::new(PasswordCache::default()),
            chain_id,
        };
        Ok(manager)
    }

    pub fn create_account(&self, password: &str) -> AccountResult<Account> {
        let private_key = gen_private_key();
        let private_key = AccountPrivateKey::Single(private_key);
        let address = private_key.public_key().derived_address();
        self.save_account(
            address,
            private_key.public_key(),
            Some((private_key, password.to_string())),
        )
    }

    pub fn unlock_account(
        &self,
        address: AccountAddress,
        password: &str,
        duration: Duration,
    ) -> AccountResult<AccountInfo> {
        let account = Account::load(address, Some(password.to_string()), self.store.clone())?
            .ok_or(AccountError::AccountNotExist(address))?;
        let ttl = std::time::Instant::now().add(duration);
        self.key_cache
            .write()
            .cache_pass(address, password.to_string(), ttl);
        Ok(account.info())
    }

    pub fn lock_account(&self, address: AccountAddress) -> AccountResult<AccountInfo> {
        let account_info = self
            .account_info(address)?
            .ok_or(AccountError::AccountNotExist(address))?;
        self.key_cache.write().remove_pass(&address);
        Ok(account_info)
    }

    pub fn import_account(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: &str,
    ) -> AccountResult<Account> {
        let private_key = AccountPrivateKey::try_from(private_key.as_slice())
            .map_err(AccountError::InvalidPrivateKey)?;
        self.save_account(
            address,
            private_key.public_key(),
            Some((private_key, password.to_string())),
        )
    }

    pub fn import_readonly_account(
        &self,
        address: AccountAddress,
        public_key: Vec<u8>,
    ) -> AccountResult<Account> {
        let public_key = AccountPublicKey::try_from(public_key.as_slice())
            .map_err(AccountError::InvalidPublicKey)?;
        self.save_account(address, public_key, None)
    }

    fn save_account(
        &self,
        address: AccountAddress,
        public_key: AccountPublicKey,
        private_key_and_password: Option<(AccountPrivateKey, String)>,
    ) -> AccountResult<Account> {
        if self.contains(&address)? {
            return Err(AccountError::AccountAlreadyExist(address));
        }
        let mut account = match private_key_and_password {
            Some((private_key, password)) => {
                Account::create(address, private_key, password, self.store.clone())?
            }
            None => Account::create_readonly(address, public_key, self.store.clone())?,
        };

        self.store.add_address(*account.address())?;

        // if it's the first address, set it default.
        if self.store.list_addresses()?.len() == 1 {
            account.set_default()?;
        }
        Ok(account)
    }

    pub fn export_account(
        &self,
        address: AccountAddress,
        password: &str,
    ) -> AccountResult<Vec<u8>> {
        let account = Account::load(address, Some(password.to_string()), self.store.clone())?
            .ok_or(AccountError::AccountNotExist(address))?;
        Ok(account
            .private_key()
            .map(|private_key| private_key.to_bytes().to_vec())
            .unwrap_or_default())
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
        let mut res = vec![];
        for account in self.store.list_addresses()? {
            if let Some(account_info) = self.account_info(account)? {
                res.push(account_info)
            } else {
                warn!(
                    "Can not find account_info by address:{}, clear it from address list.",
                    account
                );
                self.store.remove_address(account)?;
            }
        }
        Ok(res)
    }

    pub fn account_info(&self, address: AccountAddress) -> AccountResult<Option<AccountInfo>> {
        match self.store.public_key(address)? {
            Some(p) => {
                let setting = self.store.load_setting(address)?;
                Ok(Some(AccountInfo::new(
                    address,
                    p,
                    setting.is_default,
                    setting.is_readonly,
                )))
            }
            None => Ok(None),
        }
    }

    pub fn sign_message(
        &self,
        signer_address: AccountAddress,
        message: SigningMessage,
    ) -> AccountResult<SignedMessage> {
        let pass = self.key_cache.write().get_pass(&signer_address);
        match pass {
            None => Err(AccountError::AccountLocked(signer_address)),
            Some(p) => {
                let account = Account::load(signer_address, Some(p), self.store.clone())?
                    .ok_or(AccountError::AccountNotExist(signer_address))?;
                account
                    .sign_message(message, self.chain_id)
                    .map_err(AccountError::MessageSignError)
            }
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
                let account = Account::load(signer_address, Some(p), self.store.clone())?
                    .ok_or(AccountError::AccountNotExist(signer_address))?;
                account
                    .sign_txn(raw_txn)
                    .map_err(AccountError::TransactionSignError)
            }
        }
    }

    pub fn set_default_account(&self, address: AccountAddress) -> AccountResult<AccountInfo> {
        let mut account_info = self
            .account_info(address)?
            .ok_or_else(|| format_err!("Can not find account by address:{}", address))?;
        let current_default = self.store.default_address()?;
        if let Some(current_default) = current_default {
            if current_default != address {
                let mut setting = self
                    .store
                    .load_setting(current_default)
                    .map_err(AccountError::StoreError)?;
                setting.is_default = false;
                self.store.update_setting(current_default, setting)?;
            } else {
                info!("the account {} is already default.", address);
                // do not return here,for fix setting.is_default in some condition.
            }
        }

        self.store
            .set_default_address(Some(address))
            .map_err(AccountError::StoreError)?;
        let mut setting = self
            .store
            .load_setting(address)
            .map_err(AccountError::StoreError)?;
        setting.is_default = true;
        self.store.update_setting(address, setting)?;
        account_info.is_default = true;
        Ok(account_info)
    }

    pub fn change_password(
        &self,
        address: AccountAddress,
        new_pass: impl AsRef<str>,
    ) -> AccountResult<AccountInfo> {
        let account_info = self
            .account_info(address)?
            .ok_or(AccountError::AccountNotExist(address))?;

        let pass = self.key_cache.write().get_pass(&address);

        match pass {
            None => Err(AccountError::AccountLocked(address)),
            Some(old_pass) => {
                // use old pass to export the private key
                let private_key = self.export_account(address, old_pass.as_str())?;
                // and use new pass to update the encrypted private key.
                self.store
                    .update_key(
                        address,
                        &AccountPrivateKey::try_from(private_key.as_slice())
                            .expect("AccountPrivateKey from bytes should be ok"),
                        new_pass,
                    )
                    .map_err(AccountError::StoreError)?;

                // After changing password success, we should remove the old pass cache.
                // And user need to unlock it again, like we always did in websites.
                self.key_cache.write().remove_pass(&address);
                Ok(account_info)
            }
        }
    }

    /// remove account need user password.
    pub fn remove_account(
        &self,
        address: AccountAddress,
        password: Option<String>,
    ) -> AccountResult<AccountInfo> {
        let default_account = self.default_account_info()?;
        if let Some(default_account) = default_account {
            if address == default_account.address {
                return Err(AccountError::RemoveDefaultAccountError(
                    default_account.address,
                ));
            }
        }
        let account = Account::load(address, password, self.store.clone())?;
        match account {
            Some(account) => {
                self.key_cache.write().remove_pass(&address);
                let info = account.info();
                account.destroy().map_err(AccountError::StoreError)?;
                Ok(info)
            }
            None => Err(AccountError::AccountNotExist(address)),
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
