/// message for chain actor
use actix::prelude::*;
use anyhow::Result;
use crypto::HashValue;
use traits::ConnectResult;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockTemplate},
    startup_info::{ChainInfo, StartupInfo},
    transaction::SignedUserTransaction,
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
    ), // just fot test
    GetBlockByHash(HashValue),
    GetBlockInfoByHash(HashValue),
    ConnectBlock(Block, Option<BlockInfo>),
    GetStartupInfo(),
    GetHeadChainInfo(),
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
    None,
    Conn(ConnectResult<()>),
}
