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
            .retain(|head| head != &new_block_header.parent_hash());
        self.branches.retain(|head| head != &new_block_header.id());
        self.branches.push(new_block_header.id())
    }

    pub fn update_master(&mut self, new_block_header: &BlockHeader) {
        if self.master != new_block_header.parent_hash() {
            let old_master = self.master;
            self.branches.retain(|head| head != &old_master);
            self.branches.push(old_master)
        }
        self.master = new_block_header.id();
    }

    pub fn get_master(&self) -> &HashValue {
        &self.master
    }

    pub fn branch_exist_exclude(&self, branch_id: &HashValue) -> bool {
        self.branches.contains(branch_id)
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
            .map(|branch| ChainInfo::new(*branch))
            .collect();
        branches.append(&mut chain_info_vec);
        branches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startup_head() {
        let head = BlockHeader::random();
        let startup = StartupInfo::new(head.id(), Vec::new());
        assert_eq!(head.id(), *startup.get_master());
    }

    #[test]
    fn test_startup_head_parent() {
        let parent = BlockHeader::random();
        let mut startup = StartupInfo::new(parent.id(), Vec::new());
        assert_eq!(parent.id(), *startup.get_master());
        let mut son = BlockHeader::random();
        son.parent_hash = parent.id();
        startup.update_master(&son);
        assert_eq!(son.id(), *startup.get_master());
        assert!(!startup.branch_exist_exclude(&parent.id()));
    }

    #[test]
    fn test_startup_head_not_parent() {
        let parent = BlockHeader::random();
        let mut startup = StartupInfo::new(parent.id(), Vec::new());
        assert_eq!(parent.id(), *startup.get_master());
        let son = BlockHeader::random();
        startup.update_master(&son);
        assert_eq!(son.id(), *startup.get_master());
        assert!(startup.branch_exist_exclude(&parent.id()));
    }

    #[test]
    fn test_startup_branch_parent() {
        let head = BlockHeader::random();
        let mut startup = StartupInfo::new(head.id(), Vec::new());
        assert_eq!(head.id(), *startup.get_master());

        let parent = BlockHeader::random();
        startup.insert_branch(&parent);
        assert!(startup.branch_exist_exclude(&parent.id()));

        let mut son = BlockHeader::random();
        son.parent_hash = parent.id();
        startup.insert_branch(&son);
        assert!(!startup.branch_exist_exclude(&parent.id()));
        assert!(startup.branch_exist_exclude(&son.id()));
    }

    #[test]
    fn test_startup_branch_not_parent() {
        let head = BlockHeader::random();
        let mut startup = StartupInfo::new(head.id(), Vec::new());
        assert_eq!(head.id(), *startup.get_master());

        let branch1 = BlockHeader::random();
        startup.insert_branch(&branch1);
        assert!(startup.branch_exist_exclude(&branch1.id()));

        let branch2 = BlockHeader::random();
        startup.insert_branch(&branch2);
        assert!(startup.branch_exist_exclude(&branch2.id()));
        assert!(startup.branch_exist_exclude(&branch1.id()));
    }
}
