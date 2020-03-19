// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::BlockHeader;
use anyhow::Result;
use scs::SCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;

use std::convert::{TryFrom, TryInto};

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct ChainInfo {
    head_block: HashValue,
    //TODO need keep this fields?
    //pub head_number: BlockNumber,
    //pub state_root: HashValue,
    //pub accumulator_info: AccumulatorInfo,
    branch_id: HashValue,
}

impl ChainInfo {
    pub fn new(parent_hash: HashValue, branch_id: HashValue) -> Self {
        Self {
            head_block: parent_hash,
            branch_id,
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
