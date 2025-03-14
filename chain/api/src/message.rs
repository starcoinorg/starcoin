// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::range_locate::RangeInLocation;
use crate::{ChainType, TransactionInfoWithProof};
use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_dag::consensusdb::consenses_state::{DagStateView, ReachabilityView};
use starcoin_dag::types::ghostdata::GhostdagData;
use starcoin_service_registry::ServiceRequest;
use starcoin_types::transaction::RichTransactionInfo;
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber},
    contract_event::ContractEventInfo,
    filter::Filter,
    startup_info::{ChainStatus, StartupInfo},
    transaction::Transaction,
};
use starcoin_vm_types::access_path::AccessPath;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum ChainRequest {
    CurrentHeader(),
    GetHeaderByHash(HashValue),
    HeadBlock(),
    GetBlockByNumber(BlockNumber),
    GetBlockHeaderByNumber(BlockNumber),
    GetBlockByHash(HashValue),
    GetBlockInfoByHash(HashValue),
    GetBlockInfoByNumber(u64),
    GetStartupInfo(),
    GetHeadChainStatus(),
    GetTransactionBlock(HashValue),
    GetTransaction(HashValue),
    GetTransactionInfo(HashValue),
    GetBlockTransactionInfos(HashValue),
    GetTransactionInfoByBlockAndIndex {
        block_hash: HashValue,
        txn_idx: u64,
    },
    GetEventsByTxnHash {
        txn_hash: HashValue,
    },
    GetBlocksByNumber(Option<BlockNumber>, bool, u64),
    MainEvents(Filter),
    GetBlockIds {
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    },
    GetBlocks(Vec<HashValue>),
    GetHeaders(Vec<HashValue>),
    GetTransactionInfos {
        start_index: u64,
        reverse: bool,
        max_size: u64,
    },
    GetTransactionProof {
        block_id: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<AccessPath>,
    },
    GetBlockInfos(Vec<HashValue>),
    GetDagBlockChildren {
        block_ids: Vec<HashValue>,
    },
    GetDagStateView,
    CheckChainType,
    GetGhostdagData(HashValue),
    IsAncestorOfCommand {
        ancestor: HashValue,
        descendants: Vec<HashValue>,
    },
    GetRangeInLocation {
        start_id: HashValue,
        end_id: Option<HashValue>,
    },
}

impl ServiceRequest for ChainRequest {
    type Response = Result<ChainResponse>;
}

#[allow(clippy::upper_case_acronyms)]
pub enum ChainResponse {
    Block(Box<Block>),
    BlockOption(Option<Box<Block>>),
    BlockInfoOption(Box<Option<BlockInfo>>),
    BlockHeader(Box<BlockHeader>),
    BlockHeaderOption(Box<Option<BlockHeader>>),
    HashValue(HashValue),
    StartupInfo(Box<StartupInfo>),
    ChainStatus(Box<ChainStatus>),
    Transaction(Box<Transaction>),
    TransactionOption(Option<Box<Transaction>>),
    BlockVec(Vec<Block>),
    BlockOptionVec(Vec<Option<Block>>),
    BlockHeaderVec(Vec<Option<BlockHeader>>),
    TransactionInfos(Vec<RichTransactionInfo>),
    TransactionInfo(Option<RichTransactionInfo>),
    Events(Vec<ContractEventInfo>),
    MainEvents(Vec<ContractEventInfo>),
    HashVec(Vec<HashValue>),
    TransactionProof(Box<Option<TransactionInfoWithProof>>),
    BlockInfoVec(Box<Vec<Option<BlockInfo>>>),
    DagStateView(Box<DagStateView>),
    CheckChainType(ChainType),
    GhostdagDataOption(Box<Option<GhostdagData>>),
    IsAncestorOfCommand { reachability_view: ReachabilityView },
    GetRangeInLocation { range: RangeInLocation },
}
