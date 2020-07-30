use anyhow::{format_err, Result};
use crypto::hash::HashValue;
use network::NetworkAsyncService;
use network_api::NetworkService;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::AccumulatorNode;
use starcoin_network_rpc_api::{
    gen_client::NetworkRpcClient, BlockBody, GetAccumulatorNodeByNodeHash, GetBlockHeaders,
    GetBlockHeadersByNumber, GetTxns, TransactionsData,
};
use starcoin_state_tree::StateNode;
use types::{
    block::{BlockHeader, BlockInfo, BlockNumber},
    peer_info::{PeerId, RpcInfo},
    transaction::TransactionInfo,
    CHAIN_PROTOCOL_NAME,
};

const HEAD_CT: usize = 10;
// TODO: those rpc info is not used in network layout. consider remove them.
const GET_TXNS_STR: &str = "GetTxns";
const GET_TXN_INFOS_STR: &str = "GetTxnInfos";
const GET_BLOCK_HEADERS_BY_NUM_STR: &str = "GetBlockHeadersByNumber";
const GET_BLOCK_HEADERS_STR: &str = "GetBlockHeaders";
const GET_BLOCK_HEADER_BY_HASH_STR: &str = "GetBlockHeaderByHash";
const GET_BLOCK_INFOS_STR: &str = "GetBlockInfos";
const GET_BLOCK_BODIES_STR: &str = "GetBlockBodies";
const GET_STATE_NODE_BY_NODE_HASH_STR: &str = "GetStateNodeByNodeHash";
const GET_ACCUMULATOR_NODE_BY_NODE_HASH_STR: &str = "GetAccumulatorNodeByNodeHash";

pub fn sync_rpc_info() -> (&'static [u8], RpcInfo) {
    let mut paths = Vec::new();
    paths.push(GET_TXNS_STR.to_string());
    paths.push(GET_TXN_INFOS_STR.to_string());
    paths.push(GET_BLOCK_HEADERS_BY_NUM_STR.to_string());
    paths.push(GET_BLOCK_HEADERS_STR.to_string());
    paths.push(GET_BLOCK_HEADER_BY_HASH_STR.to_string());
    paths.push(GET_BLOCK_INFOS_STR.to_string());
    paths.push(GET_BLOCK_BODIES_STR.to_string());
    paths.push(GET_STATE_NODE_BY_NODE_HASH_STR.to_string());
    paths.push(GET_ACCUMULATOR_NODE_BY_NODE_HASH_STR.to_string());
    let rpc_info = RpcInfo::new(paths);
    (CHAIN_PROTOCOL_NAME, rpc_info)
}

pub async fn get_txns(
    client: &NetworkRpcClient<NetworkAsyncService>,
    peer_id: PeerId,
    req: GetTxns,
) -> Result<TransactionsData> {
    client.get_txns(peer_id, req).await
}

pub async fn get_txn_infos(
    client: &NetworkRpcClient<NetworkAsyncService>,
    peer_id: PeerId,
    block_id: HashValue,
) -> Result<Option<Vec<TransactionInfo>>> {
    client.get_txn_infos(peer_id, block_id).await
}

pub async fn get_headers_by_number(
    client: &NetworkRpcClient<NetworkAsyncService>,
    peer_id: PeerId,
    req: GetBlockHeadersByNumber,
) -> Result<Vec<BlockHeader>> {
    client.get_headers_by_number(peer_id, req).await
}

pub async fn get_headers_with_peer(
    client: &NetworkRpcClient<NetworkAsyncService>,
    peer_id: PeerId,
    req: GetBlockHeaders,
) -> Result<Vec<BlockHeader>> {
    client.get_headers_with_peer(peer_id, req).await
}

pub async fn get_headers(
    network: &NetworkAsyncService,
    client: &NetworkRpcClient<NetworkAsyncService>,
    req: GetBlockHeaders,
) -> Result<Vec<BlockHeader>> {
    if let Some(peer_info) = network.best_peer().await? {
        get_headers_with_peer(client, peer_info.get_peer_id(), req).await
    } else {
        Err(format_err!("Can not get peer when sync block header."))
    }
}

pub async fn _get_header_by_hash(
    network: &NetworkAsyncService,
    client: &NetworkRpcClient<NetworkAsyncService>,
    hashes: Vec<HashValue>,
) -> Result<Vec<BlockHeader>> {
    if let Some(peer_info) = network.best_peer().await? {
        client
            .get_header_by_hash(peer_info.get_peer_id(), hashes)
            .await
    } else {
        Err(format_err!("Can not get peer when sync block header."))
    }
}

pub async fn get_body_by_hash(
    client: &NetworkRpcClient<NetworkAsyncService>,
    network: &NetworkAsyncService,
    hashes: Vec<HashValue>,
) -> Result<Vec<BlockBody>> {
    if let Some(peer_info) = network.best_peer().await? {
        client
            .get_body_by_hash(peer_info.get_peer_id(), hashes)
            .await
    } else {
        Err(format_err!("Can not get peer when sync block body."))
    }
}

pub async fn get_info_by_hash(
    client: &NetworkRpcClient<NetworkAsyncService>,
    peer_id: PeerId,
    hashes: Vec<HashValue>,
) -> Result<Vec<BlockInfo>> {
    client.get_info_by_hash(peer_id, hashes).await
}

pub async fn get_state_node_by_node_hash(
    client: &NetworkRpcClient<NetworkAsyncService>,
    peer_id: PeerId,
    node_key: HashValue,
) -> Result<StateNode> {
    client.get_state_node_by_node_hash(peer_id, node_key).await
}

pub async fn get_accumulator_node_by_node_hash(
    client: &NetworkRpcClient<NetworkAsyncService>,
    peer_id: PeerId,
    node_key: HashValue,
    accumulator_type: AccumulatorStoreType,
) -> Result<AccumulatorNode> {
    client
        .get_accumulator_node_by_node_hash(
            peer_id,
            GetAccumulatorNodeByNodeHash {
                node_hash: node_key,
                accumulator_storage_type: accumulator_type,
            },
        )
        .await
}

/// for common
pub fn get_headers_msg_for_common(block_id: HashValue) -> GetBlockHeaders {
    GetBlockHeaders::new(block_id, 1, false, HEAD_CT)
}

pub fn get_headers_msg_for_ancestor(
    block_number: BlockNumber,
    step: usize,
) -> GetBlockHeadersByNumber {
    //todo：binary search
    GetBlockHeadersByNumber::new(block_number, step, HEAD_CT)
}
