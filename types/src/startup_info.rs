// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::{Block, BlockNumber};
use anyhow::Result;
use scs::SCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::AccumulatorInfo;
use starcoin_crypto::{hash::CryptoHash, HashValue};
use std::convert::{TryFrom, TryInto};

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct ChainInfo {
    pub head_block: HashValue,
    //TODO need keep this fields?
    //pub head_number: BlockNumber,
    //pub state_root: HashValue,
    //pub accumulator_info: AccumulatorInfo,
}

impl ChainInfo {
    pub fn new(head_block: HashValue) -> Self {
        Self { head_block }
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
