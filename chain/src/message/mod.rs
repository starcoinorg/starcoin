/// message for chain actor
use actix::prelude::*;
use anyhow::Result;
use crypto::HashValue;
use traits::ConnectBlockResult;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate},
    startup_info::{ChainInfo, StartupInfo},
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
};

#[derive(Clone)]
pub enum ChainRequest {
    CurrentHeader(),
    GetHeaderByHash(HashValue),
    HeadBlock(),
    GetBlockByNumber(BlockNumber),
    GetBlockHeaderByNumber(BlockNumber),
    CreateBlockTemplate(
        AccountAddress,
        Option<Vec<u8>>,
        Option<HashValue>,
        Vec<SignedUserTransaction>,
    ),
    // just fot test
    GetBlockByHash(HashValue),
    GetBlockInfoByHash(HashValue),
    ConnectBlock(Box<Block>),
    ConnectBlockWithoutExe(Box<Block>),
    GetStartupInfo(),
    GetHeadChainInfo(),
    GetTransaction(HashValue),
    GetBlockTransactionInfos(HashValue),
    GetTransactionInfoByBlockAndIndex {
        block_id: HashValue,
        txn_idx: u64,
    },
    GetBlocksByNumber(Option<BlockNumber>, u64),
    GetBlockStateByHash(HashValue),
}

impl Message for ChainRequest {
    type Result = Result<ChainResponse>;
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
    BlockTransactionInfos(Vec<TransactionInfo>),
    TransactionInfo(Option<TransactionInfo>),
    None,
    Conn(ConnectBlockResult),
    BlockState(Option<Box<BlockState>>),
}
