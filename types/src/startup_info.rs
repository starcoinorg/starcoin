// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::{BlockHeader, BlockNumber};
use anyhow::Result;
use scs::SCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_crypto::{HashValue};

use std::convert::{TryFrom, TryInto};


#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct ChainInfo {
    head_block: HashValue,
    //TODO need keep this fields?
    //pub head_number: BlockNumber,
    //pub state_root: HashValue,
    //pub accumulator_info: AccumulatorInfo,
    hash_number: Vec<HashValue>,
    fork_hash: HashValue,
    parent_hash: HashValue,
}

impl ChainInfo {
    pub fn new(
        fork_block_header: &BlockHeader,
        head_block_header: &BlockHeader,
        hash_number: Vec<HashValue>,
    ) -> Self {
        let mut begin_hash = Vec::new();
        begin_hash.push(fork_block_header.id());
        assert!(hash_number.starts_with(&begin_hash));
        let mut end_hash = Vec::new();
        end_hash.push(head_block_header.id());
        assert!(hash_number.ends_with(&end_hash));
        assert!(fork_block_header.number() <= head_block_header.number());
        let size = head_block_header.number() - fork_block_header.number() + 1;
        assert_eq!(size, hash_number.len() as u64);
        Self {
            head_block: head_block_header.id(),
            hash_number,
            fork_hash: fork_block_header.id(),
            parent_hash: fork_block_header.parent_hash(),
        }
    }

    pub fn get_head(&self) -> HashValue {
        assert_eq!(
            &self.head_block,
            self.hash_number.last().expect("hash_number is none.")
        );
        self.head_block
    }

    pub fn get_begin(&self) -> HashValue {
        self.hash_number.first().unwrap().clone()
    }

    pub fn contains(&self, block_id: &HashValue) -> bool {
        self.hash_number.contains(block_id)
    }

    pub fn fork(&self, block_id: &HashValue) -> Option<ChainInfo> {
        if self.contains(block_id) {
            if block_id == &self.head_block {
                Some(self.clone())
            } else {
                let mut index = 0;
                for key in &self.hash_number {
                    index = index + 1;
                    if key == block_id {
                        break;
                    }
                }

                assert!(self.hash_number.len() > index);
                let mut hash_number = self.hash_number.clone();
                hash_number.split_off(index);
                Some(ChainInfo {
                    head_block: hash_number.last().unwrap().clone(),
                    hash_number,
                    fork_hash: self.fork_hash,
                    parent_hash: self.parent_hash,
                })
            }
        } else {
            None
        }
    }

    pub fn append(&mut self, new_block: &BlockHeader) {
        assert_eq!(new_block.parent_hash(), self.head_block);
        assert_eq!(new_block.number(), self.hash_number.len() as u64);
        self.head_block = new_block.id();
        self.hash_number.push(new_block.id());
    }

    pub fn size(&self) -> usize {
        self.hash_number.len()
    }

    pub fn get_hash_by_number(&self, number: u64) -> HashValue {
        assert!(number < (self.hash_number.len() as u64));
        self.hash_number
            .get::<usize>(number as usize)
            .unwrap()
            .clone()
    }

    pub fn latest_blocks(&self) -> Vec<(BlockNumber, HashValue)> {
        let mut latest = Vec::new();
        let mut len = self.hash_number.len();
        loop {
            if len == 0 || latest.len() >= 10 {
                break;
            }
            let index = len - 1;
            let block_id: HashValue = self.hash_number.get(index).unwrap().clone();
            latest.push((index as BlockNumber, block_id));
            len = len - 1;
        }
        latest.reverse();
        latest
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
