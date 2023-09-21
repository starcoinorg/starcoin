// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::schema::block_info::BlockInfo as BlockInfoSchema;
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_schemadb::db::DBStorage;
use starcoin_types::block::BlockInfo;
use std::sync::Arc;

pub trait BlockInfoStore {
    fn save_block_info(&self, block_info: BlockInfo) -> Result<()>;
    fn get_block_info(&self, hash_value: HashValue) -> Result<Option<BlockInfo>>;
    fn delete_block_info(&self, block_hash: HashValue) -> Result<()>;
    fn get_block_infos(&self, ids: Vec<HashValue>) -> Result<Vec<Option<BlockInfo>>>;
}

#[derive(Clone)]
pub(crate) struct BlockInfoStorage {
    db: Arc<DBStorage>,
}

impl BlockInfoStorage {
    pub(crate) fn new(db: &Arc<DBStorage>) -> Self {
        Self { db: Arc::clone(db) }
    }

    pub(crate) fn remove(&self, key: &HashValue) -> Result<()> {
        self.db.remove::<BlockInfoSchema>(key)
    }

    pub(crate) fn put(&self, key: &HashValue, value: &BlockInfo) -> Result<()> {
        self.db.put::<BlockInfoSchema>(key, value)
    }

    pub(crate) fn get(&self, key: &HashValue) -> Result<Option<BlockInfo>> {
        self.db.get::<BlockInfoSchema>(key)
    }

    pub(crate) fn multi_get(&self, keys: Vec<HashValue>) -> Result<Vec<Option<BlockInfo>>> {
        self.db.multi_get::<BlockInfoSchema>(&keys)
    }
}
