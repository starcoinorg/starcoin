// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::define_storage;
use crate::storage::ValueCodec;
use crate::{BLOCK_PREFIX_NAME, FAILED_BLOCK_PREFIX_NAME};
use anyhow::Result;
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

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct FailedBlock {
    block: Block,
    peer_id: Option<PeerId>,
    failed: String,
    version: String,
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
