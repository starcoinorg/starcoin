// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{CodecKVStore, CodecWriteBatch, ValueCodec};
use crate::{define_storage, TABLE_INFO_PREFIX_NAME};
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_types::table::{TableHandleKey, TableInfoValue};

define_storage!(
    TableInfoStorage,
    TableHandleKey,
    TableInfoValue,
    TABLE_INFO_PREFIX_NAME
);

pub trait TableInfoStore {
    fn get_table_info(&self, key: TableHandleKey) -> Result<Option<TableInfoValue>>;
    fn save_table_info(&self, key: TableHandleKey, table_info: TableInfoValue) -> Result<()>;
    fn get_table_infos(&self, keys: Vec<TableHandleKey>) -> Result<Vec<Option<TableInfoValue>>>;
    fn save_table_infos(
        &self,
        keys: Vec<TableHandleKey>,
        table_infos: Vec<TableInfoValue>,
    ) -> Result<()>;
}

impl ValueCodec for TableHandleKey {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec for TableInfoValue {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl TableInfoStore for TableInfoStorage {
    fn get_table_info(&self, key: TableHandleKey) -> Result<Option<TableInfoValue>> {
        self.get(key)
    }

    fn save_table_info(&self, key: TableHandleKey, table_info: TableInfoValue) -> Result<()> {
        self.put(key, table_info)
    }

    fn get_table_infos(&self, keys: Vec<TableHandleKey>) -> Result<Vec<Option<TableInfoValue>>> {
        self.multiple_get(keys)
    }

    fn save_table_infos(
        &self,
        keys: Vec<TableHandleKey>,
        table_infos: Vec<TableInfoValue>,
    ) -> Result<()> {
        let batch = CodecWriteBatch::new_puts(keys.into_iter().zip(table_infos).collect());
        self.write_batch(batch)
    }
}
