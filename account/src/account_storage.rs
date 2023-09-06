// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_schemadb::{
    AcceptedToken, AcceptedTokens, AccountAddressWrapper, AccountSetting, AccountStore,
    EncryptedPrivateKey, GlobalSetting, GlobalSettingKey, GlobalValue, PrivateKey, PublicKey,
    PublicKeyWrapper, SettingWrapper, ACCEPTED_TOKEN_PREFIX_NAME,
    ENCRYPTED_PRIVATE_KEY_PREFIX_NAME, GLOBAL_PREFIX_NAME, PUBLIC_KEY_PREFIX_NAME,
    SETTING_PREFIX_NAME,
};
use anyhow::{Error, Result};
use starcoin_account_api::{AccountPrivateKey, AccountPublicKey, Setting};
use starcoin_config::{temp_dir, RocksdbConfig};
use starcoin_crypto::ValidCryptoMaterial;
use starcoin_decrypt::{decrypt, encrypt};
use starcoin_schemadb::{SchemaBatch, DB};
use starcoin_types::{account_address::AccountAddress, account_config::token_code::TokenCode};
use std::{convert::TryFrom, path::Path, sync::Arc};

#[derive(Clone)]
pub struct AccountStorage {
    db: Arc<DB>,
    setting_store: AccountStore<AccountSetting>,
    private_key_store: AccountStore<PrivateKey>,
    public_key_store: AccountStore<PublicKey>,
    global_value_store: AccountStore<GlobalSetting>,
    accepted_token_store: AccountStore<AcceptedToken>,
}

impl AccountStorage {
    pub fn create_from_path(p: impl AsRef<Path>, rocksdb_config: RocksdbConfig) -> Result<Self> {
        let db = DB::open_with_cfs(
            "accountdb",
            p,
            vec![
                SETTING_PREFIX_NAME,
                ENCRYPTED_PRIVATE_KEY_PREFIX_NAME,
                PUBLIC_KEY_PREFIX_NAME,
                ACCEPTED_TOKEN_PREFIX_NAME,
                GLOBAL_PREFIX_NAME,
            ],
            false,
            rocksdb_config,
            None,
        )?;
        Ok(Self::new(Arc::new(db)))
    }

    pub fn new(db: Arc<DB>) -> Self {
        Self {
            db: Arc::clone(&db),
            setting_store: AccountStore::<AccountSetting>::new(),
            private_key_store: AccountStore::<PrivateKey>::new(),
            public_key_store: AccountStore::<PublicKey>::new(),
            accepted_token_store: AccountStore::<AcceptedToken>::new(),
            global_value_store: AccountStore::<GlobalSetting>::new(),
        }
    }

    pub fn mock() -> Self {
        let path = temp_dir();
        let db = DB::open_with_cfs(
            "acccountmock",
            &path,
            vec![
                SETTING_PREFIX_NAME,
                ENCRYPTED_PRIVATE_KEY_PREFIX_NAME,
                PUBLIC_KEY_PREFIX_NAME,
                ACCEPTED_TOKEN_PREFIX_NAME,
                GLOBAL_PREFIX_NAME,
            ],
            false,
            RocksdbConfig::default(),
            None,
        )
        .unwrap();
        Self::new(Arc::new(db))
    }
}

impl AccountStorage {
    pub fn default_address(&self) -> Result<Option<AccountAddress>> {
        let value = self.get_addresses(&GlobalSettingKey::DefaultAddress)?;
        Ok(value.and_then(|mut v| v.addresses.pop()))
    }

    /// Update or remove default address settings
    pub fn set_default_address(&self, address: Option<AccountAddress>) -> Result<()> {
        let key = GlobalSettingKey::DefaultAddress;
        match address {
            Some(addr) => {
                let val = GlobalValue {
                    addresses: vec![addr],
                };
                self.put_addresses(key, val)
            }
            None => self.remove_address(&key),
        }
    }

    pub fn contain_address(&self, address: AccountAddress) -> Result<bool> {
        match self.get_public_key(&address.into())? {
            Some(v) => {
                let _ = Into::<AccountPublicKey>::into(v);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    fn get_addresses(&self, global_setting_key: &GlobalSettingKey) -> Result<Option<GlobalValue>> {
        self.global_value_store
            .get(global_setting_key)?
            .map(|v| Ok(Some(v)))
            .unwrap_or_else(|| {
                self.db
                    .get::<GlobalSetting>(&GlobalSettingKey::AllAddresses)
            })
    }

    fn put_addresses(&self, key: GlobalSettingKey, value: GlobalValue) -> Result<()> {
        self.db
            .put::<GlobalSetting>(&key, &value)
            .and_then(|_| self.global_value_store.put(key, value))
    }

    fn remove_address(&self, key: &GlobalSettingKey) -> Result<()> {
        self.db.remove::<GlobalSetting>(key)?;
        self.global_value_store.remove(key)
    }

    /// FIXME: once storage support iter, we can remove this.
    pub fn add_address(&self, address: AccountAddress) -> Result<()> {
        let value = self.get_addresses(&GlobalSettingKey::AllAddresses)?;
        let mut addrs = value.map(|v| v.addresses).unwrap_or_default();
        if !addrs.contains(&address) {
            addrs.push(address);
        }
        self.put_addresses(
            GlobalSettingKey::AllAddresses,
            GlobalValue { addresses: addrs },
        )
    }

    pub fn remove_address_from_all(&self, address: AccountAddress) -> Result<()> {
        let value = self.get_addresses(&GlobalSettingKey::AllAddresses)?;
        let mut addrs = value.map(|v| v.addresses).unwrap_or_default();
        addrs.retain(|a| a != &address);

        self.put_addresses(
            GlobalSettingKey::AllAddresses,
            GlobalValue { addresses: addrs },
        )
    }

    pub fn list_addresses(&self) -> Result<Vec<AccountAddress>> {
        let value = self.get_addresses(&GlobalSettingKey::AllAddresses)?;
        Ok(value.map(|v| v.addresses).unwrap_or_default())
    }

    fn get_public_key(&self, address: &AccountAddressWrapper) -> Result<Option<AccountPublicKey>> {
        self.public_key_store
            .get(address)?
            .map(|v| Ok(Some(v)))
            .unwrap_or_else(|| self.db.get::<PublicKey>(address))
            .map(|v| v.map(Into::into))
    }

    fn put_public_key(&self, key: AccountAddress, value: AccountPublicKey) -> Result<()> {
        let key: AccountAddressWrapper = key.into();
        let value: PublicKeyWrapper = value.into();
        self.db
            .put::<PublicKey>(&key, &value)
            .and_then(|_| self.public_key_store.put(key, value))
    }

    pub fn public_key(&self, address: AccountAddress) -> Result<Option<AccountPublicKey>> {
        self.get_public_key(&address.into())
    }

    fn get_private_key(
        &self,
        address: &AccountAddressWrapper,
    ) -> Result<Option<EncryptedPrivateKey>> {
        self.private_key_store
            .get(address)?
            .map(|v| Ok(Some(v)))
            .unwrap_or_else(|| self.db.get::<PrivateKey>(address))
    }

    //fn put_private_key(&self, key: AccountAddress, value: EncryptedPrivateKey) -> Result<()> {
    //    let key: AccountAddressWrapper = key.into();
    //    self.db
    //        .put::<PrivateKey>(&key, &value)
    //        .and_then(|_| self.private_key_store.put(key, value))
    //}

    pub fn decrypt_private_key(
        &self,
        address: AccountAddress,
        password: impl AsRef<str>,
    ) -> Result<Option<AccountPrivateKey>> {
        match self.get_private_key(&address.into())? {
            None => Ok(None),
            Some(encrypted_key) => {
                let plain_key_data = decrypt(password.as_ref().as_bytes(), &encrypted_key.0)?;
                let private_key = AccountPrivateKey::try_from(plain_key_data.as_slice())?;
                Ok(Some(private_key))
            }
        }
    }

    pub fn update_public_key(
        &self,
        address: AccountAddress,
        public_key: AccountPublicKey,
    ) -> Result<()> {
        self.put_public_key(address, public_key)
    }

    /// Update private and public key
    pub fn update_key(
        &self,
        address: AccountAddress,
        private_key: &AccountPrivateKey,
        password: impl AsRef<str>,
    ) -> Result<()> {
        let batch = SchemaBatch::default();
        let encrypted_prikey = encrypt(password.as_ref().as_bytes(), &private_key.to_bytes());
        self.private_key_store
            .put_batch(address.into(), encrypted_prikey.into(), &batch)?;
        let public_key = private_key.public_key();
        self.public_key_store
            .put_batch(address.into(), public_key.into(), &batch)?;
        self.write_schemas(batch)?;
        Ok(())
    }

    fn put_setting(&self, address: AccountAddress, setting: Setting) -> Result<()> {
        let key: AccountAddressWrapper = address.into();
        let value: SettingWrapper = setting.into();
        self.db
            .put::<AccountSetting>(&key, &value)
            .and_then(|_| self.setting_store.put(key, value))
    }

    pub fn update_setting(&self, address: AccountAddress, setting: Setting) -> Result<()> {
        self.put_setting(address, setting)
    }

    pub fn load_setting(&self, address: AccountAddress) -> Result<Setting> {
        let key: AccountAddressWrapper = address.into();
        Ok(self
            .setting_store
            .get(&key)?
            .map(|setting| Ok(Some(setting)))
            .unwrap_or_else(|| self.db.get::<AccountSetting>(&key))?
            .unwrap_or_default()
            .0)
    }

    pub fn destroy_account(&self, address: AccountAddress) -> Result<()> {
        let batch = SchemaBatch::default();

        if self.default_address()?.filter(|a| a == &address).is_some() {
            // clean up default address
            // self.set_default_address(None)?;
            self.global_value_store
                .remove_batch(&GlobalSettingKey::DefaultAddress, &batch)?;
        }

        //self.remove_address_from_all(address)?;
        {
            if let Some(GlobalValue {
                addresses: mut addrs,
            }) = self.get_addresses(&GlobalSettingKey::AllAddresses)?
            {
                addrs.retain(|a| a != &address);
                self.global_value_store.put_batch(
                    GlobalSettingKey::AllAddresses,
                    GlobalValue { addresses: addrs },
                    &batch,
                )?;
            }
        }

        let key: AccountAddressWrapper = address.into();
        self.private_key_store.remove_batch(&key, &batch)?;
        self.public_key_store.remove_batch(&key, &batch)?;
        self.setting_store.remove_batch(&key, &batch)?;
        self.accepted_token_store.remove_batch(&key, &batch)?;

        // persist updates to underlying storage
        self.db.write_schemas(batch)?;

        Ok(())
    }

    pub fn get_accepted_tokens(&self, address: AccountAddress) -> Result<Vec<TokenCode>> {
        let key: AccountAddressWrapper = address.into();
        let ts = self
            .accepted_token_store
            .get(&key)?
            .map(|v| Ok(Some(v)))
            .unwrap_or_else(|| self.db.get::<AcceptedToken>(&key))?;
        Ok(ts.map(|t| t.0).unwrap_or_default())
    }

    fn put_accepted_tokens(&self, key: AccountAddressWrapper, value: AcceptedTokens) -> Result<()> {
        self.db
            .put::<AcceptedToken>(&key, &value)
            .and_then(|_| self.accepted_token_store.put(key, value))
    }

    pub fn add_accepted_token(
        &self,
        address: AccountAddress,
        token_code: TokenCode,
    ) -> Result<(), Error> {
        let mut tokens = self.get_accepted_tokens(address)?;
        if !tokens.contains(&token_code) {
            tokens.push(token_code);
            self.put_accepted_tokens(address.into(), AcceptedTokens(tokens))?;
        }
        Ok(())
    }

    pub fn write_schemas(&self, batch: SchemaBatch) -> Result<()> {
        self.db.write_schemas(batch)
    }
}
