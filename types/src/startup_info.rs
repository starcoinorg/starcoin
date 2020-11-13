// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::BlockHeader;
use anyhow::Result;
use scs::SCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_uint::U256;
use std::convert::{TryFrom, TryInto};
use std::fmt;

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct ChainInfo {
    head: BlockHeader,
    total_difficulty: U256,
}

impl ChainInfo {
    pub fn new(head: BlockHeader, total_difficulty: U256) -> Self {
        Self {
            head,
            total_difficulty,
        }
    }

    pub fn head(&self) -> &BlockHeader {
        &self.head
    }

    pub fn total_difficulty(&self) -> U256 {
        self.total_difficulty
    }
}
//TODO save more info to StartupInfo and simple chain init.
#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct StartupInfo {
    /// Master chain info
    pub master: HashValue,
}

impl fmt::Display for StartupInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StartupInfo {{")?;
        write!(f, "master: {:?},", self.master)?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl StartupInfo {
    pub fn new(master: HashValue) -> Self {
        Self { master }
    }

    pub fn update_master(&mut self, new_block_header: &BlockHeader) {
        self.master = new_block_header.id();
    }

    pub fn get_master(&self) -> &HashValue {
        &self.master
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
