/// message for chain actor
use actix::prelude::*;
use anyhow::Result;
use crypto::HashValue;
use types::block::{Block, BlockHeader, BlockTemplate};

#[derive(Clone)]
pub enum ChainRequest {
    // just for test
    CreateBlock(u64),
    CurrentHeader(),
    GetHeaderByHash(HashValue),
    HeadBlock(),
    GetHeaderByNumber(u64),
    GetBlockByNumber(u64),
    CreateBlockTemplate(),
    GetBlockByHash(HashValue),
    ConnectBlock(Block),
}

impl Message for ChainRequest {
    type Result = Result<ChainResponse>;
}

#[derive(Clone)]
pub enum ChainResponse {
    BlockTemplate(BlockTemplate),
    Block(Block),
    OptionBlock(Option<Block>),
    BlockHeader(BlockHeader),
    None,
}
