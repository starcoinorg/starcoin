// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::{BlockHeader, BlockNumber};
use anyhow::Result;
use scs::SCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

use std::convert::{TryFrom, TryInto};

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct ChainInfo {
    head_block: HashValue,
    branch_id: HashValue,
    start_block_number: BlockNumber,
    parent_branch_id: Option<HashValue>,
}

impl ChainInfo {
    pub fn new(
        parent_branch_id: Option<HashValue>,
        head_block: HashValue,
        block_header: &BlockHeader,
    ) -> Self {
        assert!((head_block == block_header.id() || head_block == block_header.parent_hash()));
        Self {
            head_block,
            branch_id: block_header.id(),
            start_block_number: block_header.number(),
            parent_branch_id,
        }
    }

    pub fn update_head(&mut self, latest_block: BlockHeader) {
        assert_eq!(latest_block.parent_hash(), self.head_block);
        self.head_block = latest_block.id();
    }

    pub fn get_head(&self) -> HashValue {
        self.head_block
    }

    pub fn branch_id(&self) -> HashValue {
        self.branch_id.clone()
    }

    pub fn start_number(&self) -> BlockNumber {
        self.start_block_number
    }

    pub fn parent_branch(&self) -> Option<HashValue> {
        self.parent_branch_id
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct StartupInfo {
    /// head chain info
    pub head: ChainInfo,
    pub branches: Vec<ChainInfo>,
}

impl StartupInfo {
    pub fn new(head: ChainInfo, branches: Vec<ChainInfo>) -> Self {
        Self { head, branches }
    }
}

impl TryFrom<Vec<u8>> for StartupInfo {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        StartupInfo::decode(value.as_slice())
    }
}

impl TryInto<Vec<u8>> for StartupInfo {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<u8>> {
        self.encode()
    }
}
