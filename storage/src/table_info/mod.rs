// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod legacy;

use crate::storage::{CodecKVStore, CodecWriteBatch, KeyCodec, ValueCodec};
use crate::{define_storage, TABLE_INFO_PREFIX_NAME_V2};
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_types::table::{StcTableHandle, StcTableInfo};

define_storage!(
    StcTableInfoStorage,
    StcTableHandle,
    StcTableInfo,
    TABLE_INFO_PREFIX_NAME_V2
);

pub trait TableInfoStore {
    fn get_table_info(&self, key: StcTableHandle) -> Result<Option<StcTableInfo>>;
    fn save_table_info(&self, key: StcTableHandle, table_info: StcTableInfo) -> Result<()>;
    fn get_table_infos(&self, keys: Vec<StcTableHandle>) -> Result<Vec<Option<StcTableInfo>>>;
    fn save_table_infos(&self, table_infos: Vec<(StcTableHandle, StcTableInfo)>) -> Result<()>;
}

impl KeyCodec for StcTableHandle {
    fn encode_key(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_key(data: &[u8]) -> Result<Self> {
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

impl TableInfoStore for StcTableInfoStorage {
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
