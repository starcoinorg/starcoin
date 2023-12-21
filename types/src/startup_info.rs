// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block::{BlockHeader, BlockInfo, BlockNumber, LegacyBlockHeader};
use crate::dag_block::KTotalDifficulty;
use anyhow::Result;
use bcs_ext::{BCSCodec, Sample};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::accumulator_info::AccumulatorInfo;
use starcoin_crypto::HashValue;
use starcoin_uint::U256;
use starcoin_vm_types::genesis_config::ChainId;
use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;

/// The info of a chain.
#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct ChainInfo {
    chain_id: ChainId,
    genesis_hash: HashValue,
    status: ChainStatus,
    flexi_dag_accumulator_info: Option<AccumulatorInfo>,
    k_total_difficulty: KTotalDifficulty,
}

#[derive(Deserialize, Serialize)]
#[serde(rename = "ChainInfo")]
pub struct OldChainInfo {
    chain_id: ChainId,
    genesis_hash: HashValue,
    status: OldChainStatus,
}

impl From<ChainInfo> for OldChainInfo {
    fn from(value: ChainInfo) -> Self {
        Self {
            chain_id: value.chain_id,
            genesis_hash: value.genesis_hash,
            status: value.status.into(),
        }
    }
}

impl From<OldChainInfo> for ChainInfo {
    fn from(value: OldChainInfo) -> Self {
        Self {
            chain_id: value.chain_id,
            genesis_hash: value.genesis_hash,
            status: value.status.into(),
            flexi_dag_accumulator_info: None,
            k_total_difficulty: KTotalDifficulty {
                total_difficulty: 0.into(),
            },
        }
    }
}

impl ChainInfo {
    pub fn new(
        chain_id: ChainId,
        genesis_hash: HashValue,
        status: ChainStatus,
        flexi_dag_accumulator_info: Option<AccumulatorInfo>,
        k_total_difficulty: KTotalDifficulty,
    ) -> Self {
        Self {
            chain_id,
            genesis_hash,
            status,
            flexi_dag_accumulator_info,
            k_total_difficulty,
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
        self.status = status;
    }

    pub fn update_dag_accumulator_info(
        &mut self,
        flexi_dag_accumulator_info: Option<AccumulatorInfo>,
    ) {
        self.flexi_dag_accumulator_info = flexi_dag_accumulator_info;
    }

    pub fn head(&self) -> &BlockHeader {
        &self.status.head
    }

    pub fn dag_accumulator_info(&self) -> &Option<AccumulatorInfo> {
        &self.flexi_dag_accumulator_info
    }

    pub fn total_difficulty(&self) -> U256 {
        self.status.info.get_total_difficulty()
    }

    pub fn k_total_difficulties(&self) -> &KTotalDifficulty {
        &self.k_total_difficulty
    }

    pub fn into_inner(self) -> (ChainId, HashValue, ChainStatus) {
        (self.chain_id, self.genesis_hash, self.status)
    }

    pub fn random() -> Self {
        Self {
            chain_id: ChainId::new(rand::random()),
            genesis_hash: HashValue::random(),
            status: ChainStatus::random(),
            flexi_dag_accumulator_info: Some(AccumulatorInfo::new(
                HashValue::random(),
                vec![],
                rand::random::<u64>(),
                rand::random::<u64>(),
            )),
            k_total_difficulty: KTotalDifficulty {
                total_difficulty: U256::from_big_endian(
                    &rand::random::<u128>()
                        .to_be_bytes()
                        .iter()
                        .chain(rand::random::<u128>().to_be_bytes().iter())
                        .cloned()
                        .collect::<Vec<u8>>(),
                ),
            },
        }
    }
}

impl Default for ChainInfo {
    fn default() -> Self {
        Self {
            chain_id: ChainId::test(),
            genesis_hash: HashValue::default(),
            status: ChainStatus::sample(),
            flexi_dag_accumulator_info: Some(AccumulatorInfo::default()),
            k_total_difficulty: KTotalDifficulty {
                total_difficulty: U256::default(),
            },
        }
    }
}

impl std::fmt::Display for ChainInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).map_err(|_| std::fmt::Error)?
        )
    }
}

/// The latest status of a chain.
#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct ChainStatus {
    /// Chain head block's header.
    pub head: BlockHeader,
    /// Chain block info
    pub info: BlockInfo,
    /// tips
    tips_hash: Option<Vec<HashValue>>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename = "ChainStatus")]
pub struct OldChainStatus {
    pub head: LegacyBlockHeader,
    pub info: BlockInfo,
}

impl From<ChainStatus> for OldChainStatus {
    fn from(value: ChainStatus) -> Self {
        Self {
            head: value.head.into(),
            info: value.info,
        }
    }
}

impl From<OldChainStatus> for ChainStatus {
    fn from(value: OldChainStatus) -> Self {
        Self {
            head: value.head.into(),
            info: value.info,
            tips_hash: None,
        }
    }
}

impl ChainStatus {
    pub fn new(head: BlockHeader, info: BlockInfo, tips_hash: Option<Vec<HashValue>>) -> Self {
        Self {
            head,
            info,
            tips_hash,
        }
    }

    pub fn random() -> Self {
        let head = BlockHeader::random();
        let block_info = BlockInfo::new(
            head.id(),
            U256::from(rand::random::<u64>()),
            AccumulatorInfo::new(
                head.txn_accumulator_root(),
                vec![],
                rand::random::<u64>(),
                rand::random::<u64>(),
            ),
            AccumulatorInfo::new(
                head.block_accumulator_root(),
                vec![],
                head.number().saturating_sub(1),
                rand::random::<u64>(),
            ),
            AccumulatorInfo::new(
                head.block_accumulator_root(),
                vec![],
                head.number().saturating_sub(1),
                rand::random::<u64>(),
            ),
        );
        Self {
            head: head.clone(),
            info: block_info,
            tips_hash: None,
        }
    }

    pub fn head(&self) -> &BlockHeader {
        &self.head
    }

    pub fn info(&self) -> &BlockInfo {
        &self.info
    }

    pub fn tips_hash(&self) -> &Option<Vec<HashValue>> {
        &self.tips_hash
    }

    pub fn total_difficulty(&self) -> U256 {
        self.info.total_difficulty
    }

    pub fn into_inner(self) -> (BlockHeader, BlockInfo) {
        (self.head, self.info)
    }

    pub fn update_tips(&mut self, tips: Option<Vec<HashValue>>) {
        self.tips_hash = tips;
    }
}

impl Sample for ChainStatus {
    fn sample() -> Self {
        Self {
            head: BlockHeader::sample(),
            info: BlockInfo::sample(),
            tips_hash: None,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct DagChainStatus {
    pub flexi_dag_accumulator_info: AccumulatorInfo,
}

impl DagChainStatus {
    pub fn new(flexi_dag_accumulator_info: AccumulatorInfo) -> Self {
        Self {
            flexi_dag_accumulator_info,
        }
    }

    pub fn random() -> Self {
        let head = BlockHeader::random();
        Self {
            flexi_dag_accumulator_info: AccumulatorInfo::new(
                head.block_accumulator_root(),
                vec![],
                rand::random::<u64>(),
                rand::random::<u64>(),
            ),
        }
    }

    pub fn sample() -> Self {
        Self {
            flexi_dag_accumulator_info: AccumulatorInfo::sample(),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct StartupInfo {
    /// main chain head block hash
    pub main: HashValue,
}

impl Display for StartupInfo {
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

    pub fn update_main(&mut self, new_head: HashValue) {
        self.main = new_head;
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

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct DagStartupInfo {
    /// dag accumulator info hash
    dag_main: HashValue,
}

impl DagStartupInfo {
    pub fn new(value: HashValue) -> Self {
        Self { dag_main: value }
    }

    pub fn get_dag_main(&self) -> &HashValue {
        &self.dag_main
    }
}

impl Display for DagStartupInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DagStartupInfo {{")?;
        write!(f, "dag_main: {:?},", self.dag_main)?;
        write!(f, "}}")?;
        Ok(())
    }
}

impl TryFrom<Vec<u8>> for DagStartupInfo {
    type Error = anyhow::Error;
    fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        Self::decode(value.as_slice())
    }
}

impl TryInto<Vec<u8>> for DagStartupInfo {
    type Error = anyhow::Error;
    fn try_into(self) -> std::result::Result<Vec<u8>, Self::Error> {
        self.encode()
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct SnapshotRange {
    /// snapshot [start, end] block number
    start: BlockNumber,
    end: BlockNumber,
}

impl Display for SnapshotRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SnapshotHeight [{}, {}],", self.start, self.end)?;
        Ok(())
    }
}

impl SnapshotRange {
    pub fn new(start: BlockNumber, end: BlockNumber) -> Self {
        Self { start, end }
    }

    pub fn update_range(&mut self, start: BlockNumber, end: BlockNumber) {
        self.start = start;
        self.end = end;
    }

    pub fn get_start(&self) -> BlockNumber {
        self.start
    }

    pub fn get_end(&self) -> BlockNumber {
        self.end
    }
}

impl TryFrom<Vec<u8>> for SnapshotRange {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        SnapshotRange::decode(value.as_slice())
    }
}

impl TryInto<Vec<u8>> for SnapshotRange {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<u8>> {
        self.encode()
    }
}

#[derive(Eq, PartialEq, Hash, Deserialize, Serialize, Clone, Debug)]
pub struct BarnardHardFork {
    // [number, ...) block will remove
    number: BlockNumber,
    hash: HashValue,
}

impl BarnardHardFork {
    pub fn new(number: BlockNumber, hash: HashValue) -> Self {
        Self { number, hash }
    }

    pub fn get_number(&self) -> BlockNumber {
        self.number
    }

    pub fn get_hash(&self) -> HashValue {
        self.hash
    }
}

impl TryFrom<Vec<u8>> for BarnardHardFork {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self> {
        BarnardHardFork::decode(value.as_slice())
    }
}

impl TryInto<Vec<u8>> for BarnardHardFork {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<u8>> {
        self.encode()
    }
}
