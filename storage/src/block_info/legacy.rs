// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::define_storage;
use crate::storage::ValueCodec;
use crate::BLOCK_INFO_PREFIX_NAME;
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_types::block::legacy::BlockInfo;

define_storage!(
    BlockInfoStorage,
    HashValue,
    BlockInfo,
    BLOCK_INFO_PREFIX_NAME
);

impl ValueCodec for BlockInfo {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}
