// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{CodecKVStore, CodecWriteBatch, ValueCodec};
use crate::{define_storage, TABLE_INFO_PREFIX_NAME};
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_types::table::{StcTableHandle, StcTableInfo};
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};

define_storage!(
    TableInfoStorage,
    TableHandle,
    TableInfo,
    TABLE_INFO_PREFIX_NAME
);

define_storage!(
    StcTableInfoStorage,
    StcTableHandle,
    StcTableInfo,
    TABLE_INFO_PREFIX_NAME
);

pub trait TableInfoStore {
    fn get_table_info(&self, key: TableHandle) -> Result<Option<TableInfo>>;
    fn save_table_info(&self, key: TableHandle, table_info: TableInfo) -> Result<()>;
    fn get_table_infos(&self, keys: Vec<TableHandle>) -> Result<Vec<Option<TableInfo>>>;
    fn save_table_infos(&self, table_infos: Vec<(TableHandle, TableInfo)>) -> Result<()>;
}

pub trait StcTableInfoStore {
    fn get_table_info(&self, key: StcTableHandle) -> Result<Option<StcTableInfo>>;
    fn save_table_info(&self, key: StcTableHandle, table_info: StcTableInfo) -> Result<()>;
    fn get_table_infos(&self, keys: Vec<StcTableHandle>) -> Result<Vec<Option<StcTableInfo>>>;
    fn save_table_infos(&self, table_infos: Vec<(StcTableHandle, StcTableInfo)>) -> Result<()>;
}

impl ValueCodec for TableHandle {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec for TableInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec for StcTableHandle {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec for StcTableInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl TableInfoStore for TableInfoStorage {
    fn get_table_info(&self, key: TableHandle) -> Result<Option<TableInfo>> {
        self.get(key)
    }

    fn save_table_info(&self, key: TableHandle, table_info: TableInfo) -> Result<()> {
        self.put(key, table_info)
    }

    fn get_table_infos(&self, keys: Vec<TableHandle>) -> Result<Vec<Option<TableInfo>>> {
        self.multiple_get(keys)
    }

    fn save_table_infos(&self, table_infos: Vec<(TableHandle, TableInfo)>) -> Result<()> {
        let batch = CodecWriteBatch::new_puts(table_infos);
        self.write_batch(batch)
    }
}

impl StcTableInfoStore for StcTableInfoStorage {
    fn get_table_info(&self, key: StcTableHandle) -> Result<Option<StcTableInfo>> {
        self.get(key)
    }

    fn save_table_info(&self, key: StcTableHandle, table_info: StcTableInfo) -> Result<()> {
        self.put(key, table_info)
    }

    fn get_table_infos(&self, keys: Vec<StcTableHandle>) -> Result<Vec<Option<StcTableInfo>>> {
        self.multiple_get(keys)
    }

    fn save_table_infos(&self, table_infos: Vec<(StcTableHandle, StcTableInfo)>) -> Result<()> {
        let batch = CodecWriteBatch::new_puts(table_infos);
        self.write_batch(batch)
    }
}
