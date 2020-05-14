// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::BlockHeader;
use anyhow::Result;
use scs::SCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use std::convert::{TryFrom, TryInto};
use std::fmt;

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct ChainInfo {
    head_block: HashValue,
}

impl ChainInfo {
    pub fn new(head_block: HashValue) -> Self {
        Self { head_block }
    }

    pub fn get_head(&self) -> &HashValue {
        &self.head_block
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct StartupInfo {
    /// Master chain info
    pub master: HashValue,
    /// Other branch chain
    pub branches: Vec<HashValue>,
}

impl fmt::Display for StartupInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StartupInfo {{")?;
        write!(f, "master: {:?},", self.master)?;
        write!(f, "branches size: {},", self.branches.len())?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl StartupInfo {
    pub fn new(master: HashValue, branches: Vec<HashValue>) -> Self {
        Self { master, branches }
    }

    pub fn insert_branch(&mut self, new_block_header: &BlockHeader) {
        self.branches
            .retain(|head| head == &new_block_header.parent_hash());
        self.branches.push(new_block_header.id())
    }

    pub fn update_master(&mut self, new_block_header: &BlockHeader) {
        if self.master != new_block_header.parent_hash() {
            self.branches.push(self.master)
        }
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

impl Into<Vec<ChainInfo>> for StartupInfo {
    fn into(self) -> Vec<ChainInfo> {
        let mut branches = Vec::new();
        branches.push(ChainInfo::new(self.master));
        let mut chain_info_vec = self
            .branches
            .iter()
            .map(|branch| ChainInfo::new(branch.clone()))
            .collect();
        branches.append(&mut chain_info_vec);
        branches
    }
}
