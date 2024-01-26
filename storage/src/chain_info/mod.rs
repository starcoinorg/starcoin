// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{ColumnFamily, InnerStorage, KVStore};
use crate::{StorageVersion, CHAIN_INFO_PREFIX_NAME};
use anyhow::Result;
use bcs_ext::BCSCodec;
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockNumber;
use starcoin_types::startup_info::{BarnardHardFork, DagState, SnapshotRange, StartupInfo};
use std::convert::{TryFrom, TryInto};

#[derive(Clone)]
pub struct ChainInfoColumnFamily;

impl ColumnFamily for ChainInfoColumnFamily {
    type Key = String;
    type Value = Vec<u8>;

    fn name() -> &'static str {
        CHAIN_INFO_PREFIX_NAME
    }
}

pub type ChainInfoStorage = InnerStorage<ChainInfoColumnFamily>;

impl ChainInfoStorage {
    const STARTUP_INFO_KEY: &'static str = "startup_info";
    const GENESIS_KEY: &'static str = "genesis";
    const STORAGE_VERSION_KEY: &'static str = "storage_version";
    const SNAPSHOT_RANGE_KEY: &'static str = "snapshot_height";
    const BARNARD_HARD_FORK: &'static str = "barnard_hard_fork";
    const DAG_STATE_KEY: &'static str = "dag_state";
    const DAG_FORK_NUMBER: &'static str = "dag_fork_number";

    pub fn save_dag_fork_number(&self, fork_number: BlockNumber) -> Result<()> {
        self.put_sync(
            Self::DAG_FORK_NUMBER.as_bytes().to_vec(),
            fork_number.encode()?,
        )
    }

    pub fn get_dag_fork_number(&self) -> Result<Option<BlockNumber>> {
        self.get(Self::DAG_FORK_NUMBER.as_bytes())
            .and_then(|bytes| match bytes {
                Some(bytes) => Ok(Some(BlockNumber::decode(bytes.as_slice())?)),
                None => Ok(None),
            })
    }

    pub fn save_dag_state(&self, dag_state: DagState) -> Result<()> {
        self.put_sync(
            Self::DAG_STATE_KEY.as_bytes().to_vec(),
            dag_state.try_into()?,
        )
    }

    pub fn get_dag_state(&self) -> Result<Option<DagState>> {
        self.get(Self::DAG_STATE_KEY.as_bytes())
            .and_then(|bytes| match bytes {
                Some(bytes) => Ok(Some(bytes.try_into()?)),
                None => Ok(None),
            })
    }

    pub fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        self.get(Self::STARTUP_INFO_KEY.as_bytes())
            .and_then(|bytes| match bytes {
                Some(bytes) => Ok(Some(bytes.try_into()?)),
                None => Ok(None),
            })
    }

    pub fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()> {
        self.put_sync(
            Self::STARTUP_INFO_KEY.as_bytes().to_vec(),
            startup_info.try_into()?,
        )
    }

    pub fn get_genesis(&self) -> Result<Option<HashValue>> {
        self.get(Self::GENESIS_KEY.as_bytes())
            .and_then(|bytes| match bytes {
                Some(bytes) => Ok(Some(HashValue::from_slice(bytes.as_slice())?)),
                None => Ok(None),
            })
    }

    pub fn save_genesis(&self, genesis_block_hash: HashValue) -> Result<()> {
        self.put_sync(
            Self::GENESIS_KEY.as_bytes().to_vec(),
            genesis_block_hash.to_vec(),
        )
    }

    pub fn get_storage_version(&self) -> Result<StorageVersion> {
        Ok(self
            .get(Self::STORAGE_VERSION_KEY.as_bytes())
            .and_then(|bytes| match bytes {
                Some(mut bytes) => {
                    let b = bytes.pop();
                    match b {
                        None => Ok(None),
                        Some(v) => Ok(Some(StorageVersion::try_from(v)?)),
                    }
                }
                None => Ok(None),
            })?
            .unwrap_or(StorageVersion::V1))
    }

    pub fn set_storage_version(&self, version: StorageVersion) -> Result<()> {
        self.put_sync(
            Self::STORAGE_VERSION_KEY.as_bytes().to_vec(),
            vec![version as u8],
        )
    }

    pub fn get_snapshot_range(&self) -> Result<Option<SnapshotRange>> {
        self.get(Self::SNAPSHOT_RANGE_KEY.as_bytes())
            .and_then(|bytes| match bytes {
                Some(bytes) => Ok(Some(bytes.try_into()?)),
                None => Ok(None),
            })
    }

    pub fn save_snapshot_range(&self, snapshot_range: SnapshotRange) -> Result<()> {
        self.put_sync(
            Self::SNAPSHOT_RANGE_KEY.as_bytes().to_vec(),
            snapshot_range.try_into()?,
        )
    }

    pub fn get_barnard_hard_fork(&self) -> Result<Option<BarnardHardFork>> {
        self.get(Self::BARNARD_HARD_FORK.as_bytes())
            .and_then(|bytes| match bytes {
                Some(bytes) => Ok(Some(bytes.try_into()?)),
                None => Ok(None),
            })
    }

    pub fn save_barnard_hard_fork(&self, barnard_hard_fork: BarnardHardFork) -> Result<()> {
        self.put_sync(
            Self::BARNARD_HARD_FORK.as_bytes().to_vec(),
            barnard_hard_fork.try_into()?,
        )
    }
}
