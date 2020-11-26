// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::BlockHeader;
use anyhow::Result;
use scs::SCSCodec;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_uint::U256;
use starcoin_vm_types::genesis_config::ChainId;
use std::convert::{TryFrom, TryInto};
use std::fmt;

/// The info of a chain.
#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct ChainInfo {
    chain_id: ChainId,
    genesis_hash: HashValue,
    status: ChainStatus,
}

impl ChainInfo {
    pub fn new(chain_id: ChainId, genesis_hash: HashValue, status: ChainStatus) -> Self {
        Self {
            chain_id,
            genesis_hash,
            status,
        }
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub fn genesis_hash(&self) -> HashValue {
        self.genesis_hash
    }

    pub fn status(&self) -> &ChainStatus {
        &self.status
    }

    pub fn update_status(&mut self, status: ChainStatus) {
        self.status = status
    }

    pub fn head(&self) -> &BlockHeader {
        self.status.head()
    }

    pub fn total_difficulty(&self) -> U256 {
        self.status.total_difficulty()
    }

    pub fn into_inner(self) -> (ChainId, HashValue, ChainStatus) {
        (self.chain_id, self.genesis_hash, self.status)
    }

    pub fn random() -> Self {
        Self {
            chain_id: ChainId::new(rand::random()),
            genesis_hash: HashValue::random(),
            status: ChainStatus::random(),
        }
    }
}

/// The latest status of a chain.
#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct ChainStatus {
    head: BlockHeader,
    total_difficulty: U256,
}

impl ChainStatus {
    pub fn new(head: BlockHeader, total_difficulty: U256) -> Self {
        Self {
            head,
            total_difficulty,
        }
    }

    pub fn random() -> Self {
        Self {
            head: BlockHeader::random(),
            total_difficulty: U256::from(rand::random::<u64>()),
        }
    }

    pub fn head(&self) -> &BlockHeader {
        &self.head
    }

    pub fn total_difficulty(&self) -> U256 {
        self.total_difficulty
    }

    pub fn into_inner(self) -> (BlockHeader, U256) {
        (self.head, self.total_difficulty)
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct StartupInfo {
    /// main chain head block hash
    pub main: HashValue,
}

impl fmt::Display for StartupInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StartupInfo {{")?;
        write!(f, "main: {:?},", self.main)?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl StartupInfo {
    pub fn new(main: HashValue) -> Self {
        Self { main }
    }

    pub fn update_main(&mut self, new_block_header: &BlockHeader) {
        self.main = new_block_header.id();
    }

    pub fn get_main(&self) -> &HashValue {
        &self.main
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
