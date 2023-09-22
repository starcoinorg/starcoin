// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{cache_storage::GCacheStorage, upgrade::DBUpgrade};
use anyhow::Result;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::info;
use std::{hash::Hash, sync::Arc};
pub use {
    crate::batch::WriteBatch,
    starcoin_schemadb::{db::DBStorage, ColumnFamilyName, GWriteOp as WriteOp},
};

#[allow(clippy::upper_case_acronyms)]

pub type StorageInstance = GStorageInstance<Vec<u8>, Vec<u8>>;

///Generic Storage instance type define
#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
pub enum GStorageInstance<K, V>
where
    K: Hash + Eq + Default,
    V: Default,
{
    CACHE {
        cache: Arc<GCacheStorage<K, V>>,
    },
    DB {
        db: Arc<DBStorage>,
    },
    CacheAndDb {
        cache: Arc<GCacheStorage<K, V>>,
        db: Arc<DBStorage>,
    },
}

impl<K, V> GStorageInstance<K, V>
where
    K: Hash + Eq + Default,
    V: Default,
{
    pub fn new_cache_instance() -> Self {
        GStorageInstance::CACHE {
            cache: Arc::new(GCacheStorage::default()),
        }
    }
    pub fn new_db_instance(db: DBStorage) -> Self {
        Self::DB { db: Arc::new(db) }
    }

    pub fn new_cache_and_db_instance(cache: GCacheStorage<K, V>, db: DBStorage) -> Self {
        Self::CacheAndDb {
            cache: Arc::new(cache),
            db: Arc::new(db),
        }
    }

    pub fn cache(&self) -> Option<Arc<GCacheStorage<K, V>>> {
        match self {
            Self::CACHE { cache } | Self::CacheAndDb { cache, db: _ } => Some(cache.clone()),
            _ => None,
        }
    }

    pub fn db(&self) -> Option<&Arc<DBStorage>> {
        match self {
            Self::DB { db } | Self::CacheAndDb { cache: _, db } => Some(db),
            _ => None,
        }
    }

    // make sure Arc::strong_count(&db) == 1 unless will get None
    pub fn db_mut(&mut self) -> Option<&mut DBStorage> {
        match self {
            Self::DB { db } | Self::CacheAndDb { cache: _, db } => Arc::get_mut(db),
            _ => None,
        }
    }
}

impl StorageInstance {
    pub fn check_upgrade(&mut self) -> Result<()> {
        DBUpgrade::check_upgrade(self)
    }

    pub fn barnard_hard_fork(&mut self, config: Arc<NodeConfig>) -> Result<()> {
        if config.net().id().chain_id().is_barnard() {
            info!("barnard_hard_fork in");
            return DBUpgrade::barnard_hard_fork(self);
        }
        Ok(())
    }
}
