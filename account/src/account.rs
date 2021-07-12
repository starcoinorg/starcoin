// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_manager::gen_private_key;
use crate::account_storage::AccountStorage;
use anyhow::{format_err, Result};
use starcoin_account_api::error::AccountError;
use starcoin_account_api::{
    AccountInfo, AccountPrivateKey, AccountPublicKey, AccountResult, Setting,
};
use starcoin_crypto::PrivateKey;
use starcoin_logger::prelude::*;
use starcoin_storage::storage::StorageInstance;
use starcoin_types::account_address;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::genesis_config::ChainId;
use starcoin_types::sign_message::{SignedMessage, SigningMessage};
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};

pub struct Account {
    addr: AccountAddress,
    public_key: AccountPublicKey,
    private_key: Option<AccountPrivateKey>,
    setting: Setting,
    store: AccountStorage,
}

impl Account {
    pub fn create(
        address: AccountAddress,
        private_key: AccountPrivateKey,
        password: String,
        storage: AccountStorage,
    ) -> AccountResult<Self> {
        storage.update_key(address, &private_key, password.as_str())?;
        let setting = Setting::default();
        storage.update_setting(address, setting.clone())?;
        Ok(Self {
            addr: address,
            public_key: private_key.public_key(),
            private_key: Some(private_key),
            setting,
            store: storage,
        })
    }

    pub fn create_readonly(
        address: AccountAddress,
        public_key: AccountPublicKey,
        storage: AccountStorage,
    ) -> AccountResult<Self> {
        storage.update_public_key(address, public_key.clone())?;
        let setting = Setting::readonly();
        storage.update_setting(address, setting.clone())?;
        Ok(Self {
            addr: address,
            public_key,
            private_key: None,
            setting,
            store: storage,
        })
    }

    /// load account, if account not readonly account , need password to unlock private key.
    pub fn load(
        addr: AccountAddress,
        password: Option<String>,
        storage: AccountStorage,
    ) -> AccountResult<Option<Self>> {
        let setting = storage.load_setting(addr)?;

        let private_key = if setting.is_readonly {
            None
        } else {
            let decrypted_key = storage
                .decrypt_private_key(addr, password.unwrap_or_else(|| "".to_string()))
                .map_err(|e| {
                    warn!(
                        "Try to unlock {} with a invalid password, err: {:?}",
                        addr, e
                    );
                    AccountError::InvalidPassword(addr)
                })?;
            let private_key = match decrypted_key {
                None => return Ok(None),
                Some(p) => p,
            };
            Some(private_key)
        };

        let saved_public_key = storage.public_key(addr)?;
        let saved_public_key = saved_public_key.ok_or_else(|| {
            AccountError::StoreError(format_err!("public key not found for address {}", addr))
        })?;
        Ok(Some(Self {
            addr,
            public_key: saved_public_key,
            private_key,
            setting,
            store: storage,
        }))
    }

    /// Set current account to default account
    pub fn set_default(&mut self) -> Result<()> {
        self.setting.is_default = true;
        self.store.set_default_address(Some(self.addr))?;
        self.store.update_setting(self.addr, self.setting.clone())?;
        Ok(())
    }

    pub fn info(&self) -> AccountInfo {
        AccountInfo::new(
            self.addr,
            self.public_key.clone(),
            self.setting.is_default,
            self.setting.is_readonly,
        )
    }

    pub fn sign_message(
        &self,
        message: SigningMessage,
        chain_id: ChainId,
    ) -> Result<SignedMessage> {
        let authenticator = self
            .private_key
            .as_ref()
            .map(|private_key| private_key.sign_message(&message))
            .ok_or_else(|| format_err!("Readonly account can not sign message."))?;
        Ok(SignedMessage::new(
            self.addr,
            message,
            authenticator,
            chain_id,
        ))
    }

    pub fn sign_txn(&self, raw_txn: RawUserTransaction) -> Result<SignedUserTransaction> {
        let signature = self
            .private_key
            .as_ref()
            .map(|private_key| private_key.sign(&raw_txn))
            .ok_or_else(|| format_err!("Readonly account can not sign txn"))?;
        Ok(SignedUserTransaction::new(raw_txn, signature))
    }

    pub fn destroy(self) -> Result<()> {
        self.store.destroy_account(self.addr)
    }

    pub fn address(&self) -> &AccountAddress {
        &self.addr
    }

    pub fn private_key(&self) -> Option<&AccountPrivateKey> {
        self.private_key.as_ref()
    }

    pub fn public_key(&self) -> AccountPublicKey {
        self.public_key.clone()
    }

    pub fn auth_key(&self) -> AuthenticationKey {
        self.public_key.authentication_key()
    }

    ///Generate a random account for test.
    pub fn random() -> Result<Self> {
        let private_key = gen_private_key();
        let public_key = private_key.public_key();
        let address = account_address::from_public_key(&public_key);
        let storage = AccountStorage::new(StorageInstance::new_cache_instance());
        Self::create(address, private_key.into(), "".to_string(), storage).map_err(|e| e.into())
    }
}
