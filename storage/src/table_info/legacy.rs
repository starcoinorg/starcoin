// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::ValueCodec;
use crate::{define_storage, TABLE_INFO_PREFIX_NAME};
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};

define_storage!(
    TableInfoStorage,
    TableHandle,
    TableInfo,
    TABLE_INFO_PREFIX_NAME
);

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