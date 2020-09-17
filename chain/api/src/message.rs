// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_service_registry::ServiceRequest;
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate},
    contract_event::ContractEvent,
    startup_info::{ChainInfo, StartupInfo},
    transaction::{Transaction, TransactionInfo},
};
use starcoin_vm_types::on_chain_config::{EpochInfo, GlobalTimeOnChain};

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
    GetTransactionInfoByBlockAndIndex { block_id: HashValue, txn_idx: u64 },
    GetEventsByTxnInfoId { txn_info_id: HashValue },
    GetBlocksByNumber(Option<BlockNumber>, u64),
    GetBlockStateByHash(HashValue),
}

impl ServiceRequest for ChainRequest {
    type Response = Result<ChainResponse>;
}

pub enum ChainResponse {
    BlockTemplate(Box<BlockTemplate>),
    Block(Box<Block>),
    OptionBlock(Option<Box<Block>>),
    OptionBlockInfo(Box<Option<BlockInfo>>),
    BlockHeader(Box<Option<BlockHeader>>),
    HashValue(HashValue),
    StartupInfo(StartupInfo),
    ChainInfo(ChainInfo),
    Transaction(Box<Transaction>),
    VecBlock(Vec<Block>),
    TransactionInfos(Vec<TransactionInfo>),
    TransactionInfo(Option<TransactionInfo>),
    Events(Option<Vec<ContractEvent>>),
    None,
    Conn(Result<()>),
    BlockState(Option<Box<BlockState>>),
    EpochInfo(EpochInfo),
    GlobalTime(GlobalTimeOnChain),
}
