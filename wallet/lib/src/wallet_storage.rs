use anyhow::{Error, Result};
use serde::Deserialize;
use serde::Serialize;
use starcoin_canonical_serialization::SCSCodec;
use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use starcoin_decrypt::{decrypt, encrypt};
use starcoin_storage::cache_storage::CacheStorage;
use starcoin_storage::db_storage::DBStorage;
use starcoin_storage::storage::{KeyCodec, ValueCodec};
use starcoin_storage::{
    batch::WriteBatch,
    define_storage,
    storage::{CodecStorage, ColumnFamilyName, StorageInstance},
};
use starcoin_types::account_address::AccountAddress;
use starcoin_wallet_api::Setting;
use std::convert::TryFrom;
use std::path::Path;
use std::sync::Arc;

pub const SETTING_PREFIX_NAME: ColumnFamilyName = "account_settings";
pub const ENCRYPTED_PRIVATE_KEY_PREFIX_NAME: ColumnFamilyName = "encrypted_private_key";
pub const PUBLIC_KEY_PREFIX_NAME: ColumnFamilyName = "public_key";
pub const GLOBAL_PREFIX_NAME: ColumnFamilyName = "global";

define_storage!(
    WalletSettingStore,
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
        AccountAddress::try_from(data).map(AccountAddressWrapper)
    }
}

impl ValueCodec for SettingWrapper {
    fn encode_value(&self) -> Result<Vec<u8>, Error> {
        self.0.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self, Error> {
        Setting::decode(data).map(SettingWrapper)
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
pub struct PublicKeyWrapper(Ed25519PublicKey);
impl From<Ed25519PublicKey> for PublicKeyWrapper {
    fn from(s: Ed25519PublicKey) -> Self {
        Self(s)
    }
}

impl ValueCodec for PublicKeyWrapper {
    fn encode_value(&self) -> Result<Vec<u8>, Error> {
        Ok(self.0.to_bytes().to_vec())
    }

    fn decode_value(data: &[u8]) -> Result<Self, Error> {
        let key = Ed25519PublicKey::try_from(data)?;
        Ok(Self(key))
    }
}

#[derive(Clone)]
pub struct WalletStorage {
    setting_store: WalletSettingStore,
    private_key_store: PrivateKeyStore,
    public_key_store: PublicKeyStore,
    global_value_store: GlobalSettingStore,
}

impl WalletStorage {
    pub fn create_from_path(p: impl AsRef<Path>) -> Result<Self> {
        let db = DBStorage::open_with_cfs(
            p,
            vec![
                SETTING_PREFIX_NAME,
                ENCRYPTED_PRIVATE_KEY_PREFIX_NAME,
                PUBLIC_KEY_PREFIX_NAME,
                GLOBAL_PREFIX_NAME,
            ],
            false,
        )?;
        let storage_instance = StorageInstance::new_cache_and_db_instance(
            Arc::new(CacheStorage::default()),
            Arc::new(db),
        );
        Ok(Self::new(storage_instance))
    }

    pub fn new(store: StorageInstance) -> Self {
        Self {
            setting_store: WalletSettingStore::new(store.clone()),
            private_key_store: PrivateKeyStore::new(store.clone()),
            public_key_store: PublicKeyStore::new(store.clone()),
            global_value_store: GlobalSettingStore::new(store),
        }
    }
}

impl WalletStorage {
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

    pub fn list_addresses(&self) -> Result<Vec<AccountAddress>> {
        let value = self
            .global_value_store
            .get(GlobalSettingKey::AllAddresses)?;
        Ok(value.map(|v| v.addresses).unwrap_or_default())
    }

    pub fn public_key(&self, address: AccountAddress) -> Result<Option<Ed25519PublicKey>> {
        self.public_key_store
            .get(address.into())
            .map(|w| w.map(|p| p.0))
    }

    pub fn decrypt_private_key(
        &self,
        address: AccountAddress,
        password: impl AsRef<str>,
    ) -> Result<Option<Ed25519PrivateKey>> {
        let encrypted_key = self.private_key_store.get(address.into())?;
        if encrypted_key.is_none() {
            return Ok(None);
        }
        let encrypted_key = encrypted_key.unwrap();

        let plain_key_data = decrypt(password.as_ref().as_bytes(), &encrypted_key.0)?;

        let private_key = Ed25519PrivateKey::try_from(plain_key_data.as_slice())?;
        Ok(Some(private_key))
    }

    pub fn update_key(
        &self,
        address: AccountAddress,
        public_key: Ed25519PublicKey,
        private_key: &Ed25519PrivateKey,
        password: impl AsRef<str>,
    ) -> Result<()> {
        let encrypted_prikey = encrypt(password.as_ref().as_bytes(), &private_key.to_bytes());
        self.private_key_store
            .put(address.into(), encrypted_prikey.into())?;
        self.public_key_store
            .put(address.into(), public_key.into())?;
        Ok(())
    }

    #[allow(unused)]
    pub fn update_default_settings(
        &self,
        address: AccountAddress,
        setting: Setting,
    ) -> Result<(), Error> {
        self.setting_store.put(address.into(), setting.into())
    }

    pub fn destroy_wallet(&self, address: AccountAddress) -> Result<()> {
        if self.default_address()?.filter(|a| a == &address).is_some() {
            self.set_default_address(None)?;
        }
        self.private_key_store.remove(address.into())?;
        self.public_key_store.remove(address.into())?;
        self.setting_store.remove(address.into())?;
        Ok(())
    }
}
