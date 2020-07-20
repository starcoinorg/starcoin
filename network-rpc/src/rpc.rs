use crate::{
    BlockBody, GetAccumulatorNodeByNodeHash, GetBlockHeaders, GetBlockHeadersByNumber, GetTxns,
    TransactionsData, DELAY_TIME,
};
use accumulator::AccumulatorNode;
use actix::prelude::*;
use anyhow::Result;
use crypto::HashValue;
use futures::future::BoxFuture;
use logger::prelude::*;
use netowrk_rpc_derive::*;
use starcoin_state_tree::StateNode;
use types::{
    block::{BlockHeader, BlockInfo},
    peer_info::PeerId,
    transaction::TransactionInfo,
};

#[net_rpc]
pub trait NetworkRpc: Sized + Send + Sync + 'static {
    fn get_txns(&self, peer_id: PeerId, req: GetTxns) -> BoxFuture<Result<TransactionsData>>;

    fn get_txn_infos(
        &self,
        peer_id: PeerId,
        block_id: HashValue,
    ) -> BoxFuture<Result<Option<Vec<TransactionInfo>>>>;

    fn get_headers_by_number(
        &self,
        peer_id: PeerId,
        request: GetBlockHeadersByNumber,
    ) -> BoxFuture<Result<Vec<BlockHeader>>>;

    fn get_headers_with_peer(
        &self,
        peer_id: PeerId,
        request: GetBlockHeaders,
    ) -> BoxFuture<Result<Vec<BlockHeader>>>;

    fn get_info_by_hash(
        &self,
        peer_id: PeerId,
        hashes: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockInfo>>>;

    fn get_body_by_hash(
        &self,
        peer_id: PeerId,
        hashs: Vec<HashValue>,
    ) -> BoxFuture<Result<Vec<BlockBody>>>;

    fn get_state_node_by_node_hash(
        &self,
        peer_id: PeerId,
        node_key: HashValue,
    ) -> BoxFuture<Result<StateNode>>;

    fn get_accumulator_node_by_node_hash(
        &self,
        peer_id: PeerId,
        request: GetAccumulatorNodeByNodeHash,
    ) -> BoxFuture<Result<AccumulatorNode>>;
}
