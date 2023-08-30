use anyhow::Result;
use starcoin_schemadb::schema::Schema;
use starcoin_storage::cache_storage::GCacheStorage;
use starcoin_types::account_address::AccountAddress;
use std::sync::Arc;

mod accepted_token;
mod global_setting;
mod private_key;
mod public_key;
mod setting;

pub(crate) use accepted_token::*;
pub(crate) use global_setting::*;
pub(crate) use private_key::*;
pub(crate) use public_key::*;
pub(crate) use setting::*;
use starcoin_schemadb::SchemaBatch;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Copy)]
pub(crate) struct AccountAddressWrapper(AccountAddress);

impl Default for AccountAddressWrapper {
    fn default() -> Self {
        Self(AccountAddress::ZERO)
    }
}
impl From<AccountAddress> for AccountAddressWrapper {
    fn from(value: AccountAddress) -> Self {
        Self(value)
    }
}
impl TryFrom<&[u8]> for AccountAddressWrapper {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        AccountAddress::try_from(value)
            .map(Self)
            .map_err(Into::into)
    }
}

#[derive(Clone)]
pub(super) struct AccountStore<S: Schema> {
    cache: Arc<GCacheStorage<S::Key, S::Value>>,
}

impl<S: Schema> AccountStore<S> {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(GCacheStorage::<S::Key, S::Value>::new(None)),
        }
    }

    pub fn get(&self, key: &S::Key) -> Result<Option<S::Value>> {
        Ok(self.cache.get_inner(key))
    }

    pub fn put(&self, key: S::Key, value: S::Value) -> Result<()> {
        self.cache.put_inner(key, value);
        Ok(())
    }

    pub fn remove(&self, key: &S::Key) -> Result<()> {
        self.cache.remove_inner(key);
        Ok(())
    }

    pub fn put_batch(&self, key: S::Key, value: S::Value, batch: &SchemaBatch) -> Result<()> {
        batch.put::<S>(&key, &value)?;
        self.put(key, value)
    }

    pub fn remove_batch(&self, key: &S::Key, batch: &SchemaBatch) -> Result<()> {
        batch.delete::<S>(key)?;
        self.remove(key)
    }
}
