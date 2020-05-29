/// message for chain actor
use actix::prelude::*;
use anyhow::Result;
use crypto::HashValue;
use traits::ConnectResult;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate},
    startup_info::{ChainInfo, StartupInfo},
    transaction::{SignedUserTransaction, TransactionInfo},
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
    ConnectBlock(Box<Block>, Option<Box<BlockInfo>>),
    GetStartupInfo(),
    GetHeadChainInfo(),
    GetTransaction(HashValue),
    GetTransactionIdByBlock(HashValue),
    GetBlocksByNumber(Option<BlockNumber>, BlockNumber),
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
    Transaction(TransactionInfo),
    VecBlock(Vec<Block>),
    VecTransactionInfo(Vec<TransactionInfo>),
    None,
    Conn(ConnectResult<()>),
    BlockState(Option<Box<BlockState>>),
}
