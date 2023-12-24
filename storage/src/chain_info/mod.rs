// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{ColumnFamily, InnerStorage, KVStore};
use crate::{StorageVersion, CHAIN_INFO_PREFIX_NAME};
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_types::startup_info::{BarnardHardFork, SnapshotRange, StartupInfo};
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
    const FLEXI_DAG_STARTUP_INFO_KEY: &'static str = "flexi_dag_startup_info";

    fn get_value<DataT>(&self, key: &[u8]) -> Result<Option<DataT>>
    where
        DataT: TryFrom<Vec<u8>>,
        DataT::Error: Into<anyhow::Error>,
    {
        self.get(key).and_then(|bytes| match bytes {
            Some(bytes) => Ok(Some(DataT::try_from(bytes).map_err(Into::into)?)),
            None => Ok(None),
        })
    }

    fn update_value<DataT>(&self, key: Vec<u8>, value: DataT) -> Result<()>
    where
        DataT: TryInto<Vec<u8>>,
        DataT::Error: Into<anyhow::Error>,
    {
        self.put_sync(key, value.try_into().map_err(Into::into)?)
    }

    pub fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        self.get_value(Self::STARTUP_INFO_KEY.as_bytes())
    }

    pub fn save_startup_info(&self, startup_info: StartupInfo) -> Result<()> {
        self.update_value(Self::STARTUP_INFO_KEY.as_bytes().to_vec(), startup_info)
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
        self.get_value(Self::SNAPSHOT_RANGE_KEY.as_bytes())
    }

    pub fn save_snapshot_range(&self, snapshot_range: SnapshotRange) -> Result<()> {
        self.update_value(Self::SNAPSHOT_RANGE_KEY.as_bytes().to_vec(), snapshot_range)
    }

    pub fn get_barnard_hard_fork(&self) -> Result<Option<BarnardHardFork>> {
        self.get_value(Self::BARNARD_HARD_FORK.as_bytes())
    }

    pub fn save_barnard_hard_fork(&self, barnard_hard_fork: BarnardHardFork) -> Result<()> {
        self.update_value(
            Self::BARNARD_HARD_FORK.as_bytes().to_vec(),
            barnard_hard_fork,
        )
    }
}
