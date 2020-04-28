/// message for chain actor
use actix::prelude::*;
use anyhow::Result;
use crypto::HashValue;
use traits::ConnectResult;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockTemplate},
    startup_info::{ChainInfo, StartupInfo},
    transaction::{SignedUserTransaction, TransactionInfo},
};

#[derive(Clone)]
pub enum ChainRequest {
    CurrentHeader(),
    GetHeaderByHash(HashValue),
    HeadBlock(),
    GetBlockByNumber(u64),
    CreateBlockTemplate(
        AccountAddress,
        Option<Vec<u8>>,
        Option<HashValue>,
        Vec<SignedUserTransaction>,
    ),
    // just fot test
    GetBlockByHash(HashValue),
    GetBlockInfoByHash(HashValue),
    ConnectBlock(Block, Option<BlockInfo>),
    GetStartupInfo(),
    GetHeadChainInfo(),
    GetTransaction(HashValue),
    GetTransactionIdByBlock(HashValue),
    GetBlocksByNumber(Option<BlockNumber>, u64),
    GenTx(), // just for test
}

impl Message for ChainRequest {
    type Result = Result<ChainResponse>;
}

pub enum ChainResponse {
    BlockTemplate(BlockTemplate),
    Block(Block),
    OptionBlock(Option<Block>),
    OptionBlockInfo(Option<BlockInfo>),
    BlockHeader(BlockHeader),
    HashValue(HashValue),
    StartupInfo(StartupInfo),
    ChainInfo(ChainInfo),
    Transaction(TransactionInfo),
    VecBlock(Vec<Block>),
    VecTransactionInfo(Vec<TransactionInfo>),
    None,
    Conn(ConnectResult<()>),
}
