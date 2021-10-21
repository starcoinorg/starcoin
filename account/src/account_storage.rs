// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use bcs_ext::BCSCodec;
use serde::Deserialize;
use serde::Serialize;
use starcoin_account_api::{AccountPrivateKey, AccountPublicKey, Setting};
use starcoin_config::RocksdbConfig;
use starcoin_crypto::ValidCryptoMaterial;
use starcoin_decrypt::{decrypt, encrypt};
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::{KeyCodec, ValueCodec};
use starcoin_storage::{
    define_storage,
    storage::{CodecKVStore, ColumnFamilyName, StorageInstance},
};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::token_code::TokenCode;
use std::convert::TryFrom;
use std::path::Path;

pub const SETTING_PREFIX_NAME: ColumnFamilyName = "account_settings";
pub const ENCRYPTED_PRIVATE_KEY_PREFIX_NAME: ColumnFamilyName = "encrypted_private_key";
pub const PUBLIC_KEY_PREFIX_NAME: ColumnFamilyName = "public_key";
pub const ACCEPTED_TOKEN_PREFIX_NAME: ColumnFamilyName = "accepted_token";
pub const GLOBAL_PREFIX_NAME: ColumnFamilyName = "global";

define_storage!(
    AccountSettingStore,
    AccountAddressWrapper,
    SettingWrapper,
    SETTING_PREFIX_NAME
);

define_storage!(
    PrivateKeyStore,
    AccountAddressWrapper,
    EncryptedPrivateKey,
    ENCRYPTED_PRIVATE_KEY_PREFIX_NAME
);
define_storage!(
    PublicKeyStore,
    AccountAddressWrapper,
    PublicKeyWrapper,
    PUBLIC_KEY_PREFIX_NAME
);

define_storage!(
    GlobalSettingStore,
    GlobalSettingKey,
    GlobalValue,
    GLOBAL_PREFIX_NAME
);

define_storage!(
    AcceptedTokenStore,
    AccountAddressWrapper,
    AcceptedTokens,
    ACCEPTED_TOKEN_PREFIX_NAME
);

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct AcceptedTokens(pub Vec<TokenCode>);

impl ValueCodec for AcceptedTokens {
    fn encode_value(&self) -> Result<Vec<u8>, Error> {
        self.0.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self, Error> {
        <Vec<TokenCode>>::decode(data).map(AcceptedTokens)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum GlobalSettingKey {
    DefaultAddress,
    /// FIXME: once db support iter, remove this.
    AllAddresses,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalValue {
    addresses: Vec<AccountAddress>,
}

impl KeyCodec for GlobalSettingKey {
    fn encode_key(&self) -> Result<Vec<u8>, Error> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> Result<Self, Error> {
        GlobalSettingKey::decode(data)
    }
}

impl ValueCodec for GlobalValue {
    fn encode_value(&self) -> Result<Vec<u8>, Error> {
        self.addresses.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self, Error> {
        <Vec<AccountAddress>>::decode(data).map(|addresses| GlobalValue { addresses })
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub struct AccountAddressWrapper(AccountAddress);
impl From<AccountAddress> for AccountAddressWrapper {
    fn from(addr: AccountAddress) -> Self {
        Self(addr)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingWrapper(Setting);
impl From<Setting> for SettingWrapper {
    fn from(setting: Setting) -> Self {
        Self(setting)
    }
}

impl KeyCodec for AccountAddressWrapper {
    fn encode_key(&self) -> Result<Vec<u8>, Error> {
        Ok(self.0.to_vec())
    }

    fn decode_key(data: &[u8]) -> Result<Self, Error> {
        AccountAddress::try_from(data)
            .map(AccountAddressWrapper)
            .map_err(anyhow::Error::new)
    }
}
/// Setting use json encode/decode for support more setting field in the future.
impl ValueCodec for SettingWrapper {
    fn encode_value(&self) -> Result<Vec<u8>, Error> {
        Ok(serde_json::to_vec(&self.0)?)
    }

    fn decode_value(data: &[u8]) -> Result<Self, Error> {
        Ok(SettingWrapper(serde_json::from_slice(data)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedPrivateKey(pub Vec<u8>);
impl From<Vec<u8>> for EncryptedPrivateKey {
    fn from(s: Vec<u8>) -> Self {
        Self(s)
    }
}

impl ValueCodec for EncryptedPrivateKey {
    fn encode_value(&self) -> Result<Vec<u8>, Error> {
        Ok(self.0.clone())
    }

    fn decode_value(data: &[u8]) -> Result<Self, Error> {
        Ok(EncryptedPrivateKey(data.to_vec()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicKeyWrapper(AccountPublicKey);
impl From<AccountPublicKey> for PublicKeyWrapper {
    fn from(s: AccountPublicKey) -> Self {
        Self(s)
    }
}

impl ValueCodec for PublicKeyWrapper {
    fn encode_value(&self) -> Result<Vec<u8>, Error> {
        bcs_ext::to_bytes(&self.0)
    }

    fn decode_value(data: &[u8]) -> Result<Self, Error> {
        Ok(Self(bcs_ext::from_bytes::<AccountPublicKey>(data)?))
    }
}

#[derive(Clone)]
pub struct AccountStorage {
    setting_store: AccountSettingStore,
    private_key_store: PrivateKeyStore,
    public_key_store: PublicKeyStore,
    global_value_store: GlobalSettingStore,
    accepted_token_store: AcceptedTokenStore,
}

impl AccountStorage {
    pub fn create_from_path(p: impl AsRef<Path>, rocksdb_config: RocksdbConfig) -> Result<Self> {
        let db = DBStorage::open_with_cfs(
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
        let storage_instance =
            StorageInstance::new_cache_and_db_instance(CacheStorage::default(), db);
        Ok(Self::new(storage_instance))
    }

    pub fn new(store: StorageInstance) -> Self {
        Self {
            setting_store: AccountSettingStore::new(store.clone()),
            private_key_store: PrivateKeyStore::new(store.clone()),
            public_key_store: PublicKeyStore::new(store.clone()),
            accepted_token_store: AcceptedTokenStore::new(store.clone()),
            global_value_store: GlobalSettingStore::new(store),
        }
    }

    pub fn mock() -> Self {
        let storage_instance = StorageInstance::new_cache_instance();
        Self::new(storage_instance)
    }
}

impl AccountStorage {
    pub fn default_address(&self) -> Result<Option<AccountAddress>> {
        let value = self
            .global_value_store
            .get(GlobalSettingKey::DefaultAddress)?;
        Ok(value.and_then(|mut v| v.addresses.pop()))
    }

    /// Update or remove default address settings
    pub fn set_default_address(&self, address: Option<AccountAddress>) -> Result<()> {
        match address {
            Some(addr) => self.global_value_store.put(
                GlobalSettingKey::DefaultAddress,
                GlobalValue {
                    addresses: vec![addr],
                },
            ),
            None => self
                .global_value_store
                .remove(GlobalSettingKey::DefaultAddress),
        }
    }

    pub fn contain_address(&self, address: AccountAddress) -> Result<bool> {
        self.public_key_store
            .get(address.into())
            .map(|w| w.is_some())
    }

    /// FIXME: once storage support iter, we can remove this.
    pub fn add_address(&self, address: AccountAddress) -> Result<()> {
        let value = self
            .global_value_store
            .get(GlobalSettingKey::AllAddresses)?;
        let mut addrs = value.map(|v| v.addresses).unwrap_or_default();
        if !addrs.contains(&address) {
            addrs.push(address);
        }
        self.global_value_store.put(
            GlobalSettingKey::AllAddresses,
            GlobalValue { addresses: addrs },
        )
    }

    pub fn remove_address(&self, address: AccountAddress) -> Result<()> {
        let value = self
            .global_value_store
            .get(GlobalSettingKey::AllAddresses)?;
        let mut addrs = value.map(|v| v.addresses).unwrap_or_default();
        addrs.retain(|a| a != &address);

        self.global_value_store.put(
            GlobalSettingKey::AllAddresses,
            GlobalValue { addresses: addrs },
        )
    }

    pub fn list_addresses(&self) -> Result<Vec<AccountAddress>> {
        let value = self
            .global_value_store
            .get(GlobalSettingKey::AllAddresses)?;
        Ok(value.map(|v| v.addresses).unwrap_or_default())
    }

    pub fn public_key(&self, address: AccountAddress) -> Result<Option<AccountPublicKey>> {
        self.public_key_store
            .get(address.into())
            .map(|w| w.map(|p| p.0))
    }

    pub fn decrypt_private_key(
        &self,
        address: AccountAddress,
        password: impl AsRef<str>,
    ) -> Result<Option<AccountPrivateKey>> {
        match self.private_key_store.get(address.into())? {
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
        self.public_key_store
            .put(address.into(), public_key.into())?;
        Ok(())
    }

    /// Update private and public key
    pub fn update_key(
        &self,
        address: AccountAddress,
        private_key: &AccountPrivateKey,
        password: impl AsRef<str>,
    ) -> Result<()> {
        let encrypted_prikey = encrypt(password.as_ref().as_bytes(), &private_key.to_bytes());
        self.private_key_store
            .put(address.into(), encrypted_prikey.into())?;
        let public_key = private_key.public_key();
        self.update_public_key(address, public_key)?;
        Ok(())
    }

    pub fn update_setting(&self, address: AccountAddress, setting: Setting) -> Result<()> {
        self.setting_store.put(address.into(), setting.into())
    }

    pub fn load_setting(&self, address: AccountAddress) -> Result<Setting> {
        Ok(self
            .setting_store
            .get(address.into())?
            .map(|setting| setting.0)
            .unwrap_or_else(Setting::default))
    }

    pub fn destroy_account(&self, address: AccountAddress) -> Result<()> {
        if self.default_address()?.filter(|a| a == &address).is_some() {
            self.set_default_address(None)?;
        }
        self.remove_address(address)?;
        self.private_key_store.remove(address.into())?;
        self.public_key_store.remove(address.into())?;
        self.setting_store.remove(address.into())?;
        self.accepted_token_store.remove(address.into())?;

        Ok(())
    }

    pub fn get_accepted_tokens(&self, address: AccountAddress) -> Result<Vec<TokenCode>> {
        let ts = self.accepted_token_store.get(address.into())?;
        Ok(ts.map(|t| t.0).unwrap_or_default())
    }

    pub fn add_accepted_token(
        &self,
        address: AccountAddress,
        token_code: TokenCode,
    ) -> Result<(), Error> {
        let mut tokens = self
            .accepted_token_store
            .get(address.into())?
            .map(|l| l.0)
            .unwrap_or_default();
        if !tokens.contains(&token_code) {
            tokens.push(token_code);
            self.accepted_token_store
                .put(address.into(), AcceptedTokens(tokens))?;
        }
        Ok(())
    }
}
