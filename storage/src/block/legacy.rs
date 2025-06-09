// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::define_storage;
use crate::storage::{CodecKVStore, SchemaStorage, ValueCodec};
use crate::{BLOCK_PREFIX_NAME, FAILED_BLOCK_PREFIX_NAME};
use anyhow::{format_err, Result};
use bcs_ext::BCSCodec;
use network_p2p_types::peer_id::PeerId;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_types::block::legacy::Block;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct OldFailedBlock {
    block: Block,
    peer_id: Option<PeerId>,
    failed: String,
}

impl OldFailedBlock {
    pub fn new(block: Block, peer_id: Option<PeerId>, failed: String) -> Self {
        Self {
            block,
            peer_id,
            failed,
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct FailedBlock {
    block: Block,
    peer_id: Option<PeerId>,
    failed: String,
    version: String,
}

impl From<FailedBlock> for super::FailedBlock {
    fn from(failed_block: FailedBlock) -> Self {
        super::FailedBlock {
            block: failed_block.block.into(),
            peer_id: failed_block.peer_id,
            failed: failed_block.failed,
            version: failed_block.version,
        }
    }
}

define_storage!(BlockInnerStorage, HashValue, Block, BLOCK_PREFIX_NAME);
define_storage!(
    FailedBlockStorage,
    HashValue,
    FailedBlock,
    FAILED_BLOCK_PREFIX_NAME
);

impl ValueCodec for Block {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl ValueCodec for OldFailedBlock {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}
impl ValueCodec for FailedBlock {
    fn encode_value(&self) -> Result<Vec<u8>> {
        self.encode()
    }

    fn decode_value(data: &[u8]) -> Result<Self> {
        Self::decode(data)
    }
}

impl FailedBlockStorage {
    // todo: update when iterating?
    pub(crate) fn upgrade_old_failed_block(&self) -> Result<usize> {
        let mut item_count = 0;
        let db = self
            .get_store()
            .storage()
            .db()
            .ok_or_else(|| format_err!("Only support scan on db storage instance"))?;
        let mut iter = db.iter::<HashValue, Vec<u8>>(self.get_store().prefix_name)?;
        loop {
            let Some(item) = iter.next() else { break };
            let (key, value) = item?;
            let result = OldFailedBlock::decode_value(value.as_slice());
            let Ok(old) = result else {
                continue;
            };
            let new = FailedBlock {
                block: old.block,
                peer_id: old.peer_id,
                failed: old.failed,
                version: "".to_string(),
            };
            item_count += 1;
            self.put(key, new)?
        }
        Ok(item_count)
    }
}
