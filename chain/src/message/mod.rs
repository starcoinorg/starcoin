/// message for chain actor
use actix::prelude::*;
use anyhow::Result;
use crypto::HashValue;
use starcoin_types::peer_info::PeerId;
use starcoin_types::{
    account_address::AccountAddress,
    block::{Block, BlockHeader, BlockInfo, BlockNumber, BlockState, BlockTemplate},
    contract_event::ContractEvent,
    startup_info::{ChainInfo, StartupInfo},
    transaction::{SignedUserTransaction, Transaction, TransactionInfo},
};
use starcoin_vm_types::on_chain_config::EpochInfo;
use traits::ConnectBlockResult;

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
    GetBlockByUncle(HashValue),
    GetBlockInfoByHash(HashValue),
    ConnectBlock(Box<Block>),
    ConnectBlockWithoutExe(Box<Block>, PeerId),
    GetStartupInfo(),
    GetHeadChainInfo(),
    GetEpochInfo(),
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
    TransactionInfos(Vec<TransactionInfo>),
    TransactionInfo(Option<TransactionInfo>),
    Events(Option<Vec<ContractEvent>>),
    None,
    Conn(ConnectBlockResult),
    BlockState(Option<Box<BlockState>>),
    EpochInfo(EpochInfo),
}
