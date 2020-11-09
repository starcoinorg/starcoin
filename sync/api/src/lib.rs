use actix::prelude::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::AccumulatorNode;
use starcoin_crypto::HashValue;
use starcoin_network_rpc_api::{
    BlockBody, GetBlockHeaders, GetBlockHeadersByNumber, GetTxns, TransactionsData,
};
use starcoin_state_tree::StateNode;
use starcoin_types::peer_info::PeerId;
use starcoin_types::{
    block::{Block, BlockHeader, BlockInfo},
    transaction::TransactionInfo,
};

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct StartSyncTxnEvent;

#[derive(Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct PeerNewBlock {
    peer_id: PeerId,
    new_block: Block,
}

impl PeerNewBlock {
    pub fn new(peer_id: PeerId, new_block: Block) -> Self {
        PeerNewBlock { peer_id, new_block }
    }

    pub fn get_peer_id(&self) -> PeerId {
        self.peer_id.clone()
    }

    pub fn get_block(&self) -> &Block {
        &self.new_block
    }
}

#[derive(Message, Clone, Serialize, Deserialize, Debug)]
#[rtype(result = "Result<()>")]
pub enum SyncRpcRequest {
    GetBlockHeadersByNumber(GetBlockHeadersByNumber),
    GetBlockHeaders(GetBlockHeaders),
    GetBlockInfos(Vec<HashValue>),
    GetBlockBodies(Vec<HashValue>),
    GetStateNodeByNodeHash(HashValue),
    GetAccumulatorNodeByNodeHash(HashValue, AccumulatorStoreType),
    GetTxns(GetTxns),
    GetTxnInfos(HashValue),
}

#[derive(Message, Clone, Serialize, Deserialize)]
#[rtype(result = "Result<()>")]
pub enum SyncRpcResponse {
    BlockHeaders(Vec<BlockHeader>),
    BlockBodies(Vec<BlockBody>),
    BlockInfos(Vec<BlockInfo>),
    StateNode(StateNode),
    AccumulatorNode(AccumulatorNode),
    GetTxns(TransactionsData),
    GetTxnInfos(Option<Vec<TransactionInfo>>),
}

#[derive(Debug, Message, Clone, Serialize, Deserialize)]
#[rtype(result = "()")]
pub enum SyncNotify {
    ClosePeerMsg(PeerId),
    NewHeadBlock(PeerId, Box<Block>),
    NewPeerMsg(PeerId),
}
