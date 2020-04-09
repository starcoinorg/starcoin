/// message for chain actor
use actix::prelude::*;
use anyhow::Result;
use crypto::HashValue;
use types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockTemplate},
    startup_info::StartupInfo,
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
        types::U256,
    ), // just fot test
    GetBlockByHash(HashValue),
    GetBlockInfoByHash(HashValue),
    ConnectBlock(Block, Option<BlockInfo>),
    GetStartupInfo(),
    GenTx(), // just for test
}

impl Message for ChainRequest {
    type Result = Result<ChainResponse>;
}

#[derive(Clone)]
pub enum ChainResponse {
    BlockTemplate(BlockTemplate),
    Block(Block),
    OptionBlock(Option<Block>),
    OptionBlockInfo(Option<BlockInfo>),
    BlockHeader(BlockHeader),
    HashValue(HashValue),
    StartupInfo(StartupInfo),
    None,
}
