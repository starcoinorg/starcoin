// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_service_registry::ServiceRequest;
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate},
    contract_event::ContractEvent,
    contract_event::ContractEventInfo,
    filter::Filter,
    startup_info::{ChainInfo, StartupInfo},
    transaction::{Transaction, TransactionInfo},
};
use starcoin_vm_types::on_chain_resource::{EpochInfo, GlobalTimeOnChain};

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
pub enum ChainRequest {
    CurrentHeader(),
    GetHeaderByHash(HashValue),
    HeadBlock(),
    GetBlockByNumber(BlockNumber),
    GetBlockHeaderByNumber(BlockNumber),
    GetBlockByHash(HashValue),
    GetBlockByUncle(HashValue),
    GetBlockInfoByHash(HashValue),
    GetStartupInfo(),
    GetHeadChainInfo(),
    GetEpochInfo(),
    GetEpochInfoByNumber(u64),
    GetGlobalTimeByNumber(u64),
    GetTransaction(HashValue),
    GetTransactionInfo(HashValue),
    GetBlockTransactionInfos(HashValue),
    GetTransactionInfoByBlockAndIndex {
        block_id: HashValue,
        txn_idx: u64,
    },
    GetEventsByTxnInfoId {
        txn_info_id: HashValue,
    },
    GetBlocksByNumber(Option<BlockNumber>, u64),
    GetBlockStateByHash(HashValue),
    MasterEvents(Filter),
    GetBlockIds {
        start_number: BlockNumber,
        reverse: bool,
        max_size: u64,
    },
    GetBlocks(Vec<HashValue>),
}

impl ServiceRequest for ChainRequest {
    type Response = Result<ChainResponse>;
}

pub enum ChainResponse {
    BlockTemplate(Box<BlockTemplate>),
    Block(Box<Block>),
    BlockOption(Option<Box<Block>>),
    BlockInfoOption(Box<Option<BlockInfo>>),
    BlockHeader(Box<BlockHeader>),
    BlockHeaderOption(Box<Option<BlockHeader>>),
    HashValue(HashValue),
    StartupInfo(Box<StartupInfo>),
    ChainInfo(Box<ChainInfo>),
    Transaction(Box<Transaction>),
    BlockVec(Vec<Block>),
    BlockOptionVec(Vec<Option<Block>>),
    TransactionInfos(Vec<TransactionInfo>),
    TransactionInfo(Option<TransactionInfo>),
    Events(Option<Vec<ContractEvent>>),
    MasterEvents(Vec<ContractEventInfo>),
    None,
    Conn(Result<()>),
    BlockState(Option<Box<BlockState>>),
    EpochInfo(EpochInfo),
    GlobalTime(GlobalTimeOnChain),
    HashVec(Vec<HashValue>),
}
