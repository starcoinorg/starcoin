// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_service_registry::ServiceRequest;
use starcoin_types::stress_test::TPS;
use starcoin_types::transaction::BlockTransactionInfo;
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber},
    contract_event::ContractEventInfo,
    filter::Filter,
    startup_info::{ChainStatus, StartupInfo},
    transaction::Transaction,
};

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
    GetBlocksByNumber(Option<BlockNumber>, u64),
    MainEvents(Filter),
    GetBlockIds {
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    },
    GetBlocks(Vec<HashValue>),
    GetHeaders(Vec<HashValue>),
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
    BlockHeaderVec(Vec<BlockHeader>),
    TransactionInfos(Vec<BlockTransactionInfo>),
    TransactionInfo(Option<BlockTransactionInfo>),
    Events(Vec<ContractEventInfo>),
    MainEvents(Vec<ContractEventInfo>),
    None,
    HashVec(Vec<HashValue>),
    TPS(TPS),
}
