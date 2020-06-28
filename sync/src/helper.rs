use crate::{do_duration, DELAY_TIME};
use anyhow::{format_err, Result};
use crypto::hash::HashValue;
use futures::channel::mpsc::Sender;
use futures::sink::SinkExt;
use network::NetworkAsyncService;
use network_api::NetworkService;
use starcoin_accumulator::node::AccumulatorStoreType;
use starcoin_accumulator::AccumulatorNode;
use starcoin_canonical_serialization::SCSCodec;
use starcoin_state_tree::StateNode;
use starcoin_sync_api::{
    BlockBody, GetBlockHeaders, GetBlockHeadersByNumber, GetTxns, SyncRpcRequest, SyncRpcResponse,
    TransactionsData,
};
use std::borrow::Cow;
use types::{
    block::{BlockHeader, BlockInfo, BlockNumber},
    peer_info::{PeerId, RpcInfo},
    transaction::TransactionInfo,
    CHAIN_PROTOCOL_NAME,
};

const HEAD_CT: usize = 10;

const GET_TXNS_STR: &str = "GetTxns";
const GET_TXN_INFO_STR: &str = "GetTxnInfo";
const GET_BLOCK_HEADERS_BY_NUM_STR: &str = "GetBlockHeadersByNumber";
const GET_BLOCK_HEADERS_STR: &str = "GetBlockHeaders";
const GET_BLOCK_INFOS_STR: &str = "GetBlockInfos";
const GET_BLOCK_BODIES_STR: &str = "GetBlockBodies";
const GET_STATE_NODE_BY_NODE_HASH_STR: &str = "GetStateNodeByNodeHash";
const GET_ACCUMULATOR_NODE_BY_NODE_HASH_STR: &str = "GetAccumulatorNodeByNodeHash";

pub fn sync_rpc_info() -> (&'static [u8], RpcInfo) {
    let mut paths = Vec::new();
    paths.push(GET_TXNS_STR.to_string());
    paths.push(GET_TXN_INFO_STR.to_string());
    paths.push(GET_BLOCK_HEADERS_BY_NUM_STR.to_string());
    paths.push(GET_BLOCK_HEADERS_STR.to_string());
    paths.push(GET_BLOCK_INFOS_STR.to_string());
    paths.push(GET_BLOCK_BODIES_STR.to_string());
    paths.push(GET_STATE_NODE_BY_NODE_HASH_STR.to_string());
    paths.push(GET_ACCUMULATOR_NODE_BY_NODE_HASH_STR.to_string());
    let rpc_info = RpcInfo::new(paths);
    (CHAIN_PROTOCOL_NAME, rpc_info)
}

async fn do_request(
    network: &NetworkAsyncService,
    peer_id: PeerId,
    path: &str,
    req: SyncRpcRequest,
) -> Result<SyncRpcResponse> {
    let request = req.encode()?;
    let response = network
        .send_request_bytes(
            CHAIN_PROTOCOL_NAME.into(),
            peer_id.into(),
            path.to_string(),
            request,
            do_duration(DELAY_TIME),
        )
        .await?;
    SyncRpcResponse::decode(&response)
}

pub async fn get_txns(
    network: &NetworkAsyncService,
    peer_id: PeerId,
    req: GetTxns,
) -> Result<TransactionsData> {
    let request = SyncRpcRequest::GetTxns(req);
    if let SyncRpcResponse::GetTxns(txn_data) =
        do_request(&network, peer_id, GET_TXNS_STR, request).await?
    {
        Ok(txn_data)
    } else {
        Err(format_err!("{:?}", "error SyncRpcResponse type."))
    }
}

pub async fn get_txn_info(
    network: &NetworkAsyncService,
    peer_id: PeerId,
    txn_info_hash: HashValue,
) -> Result<Option<TransactionInfo>> {
    let get_txn_info_req = SyncRpcRequest::GetTxnInfo(txn_info_hash);
    if let SyncRpcResponse::GetTxnInfo(txn_info) =
        do_request(&network, peer_id, GET_TXN_INFO_STR, get_txn_info_req).await?
    {
        //todo: Verify response
        Ok(txn_info)
    } else {
        Err(format_err!(
            "{:?}",
            "error SyncRpcResponse type when get txn info."
        ))
    }
}

pub async fn get_headers_by_number(
    network: &NetworkAsyncService,
    peer_id: PeerId,
    req: GetBlockHeadersByNumber,
) -> Result<Vec<BlockHeader>> {
    let get_block_headers_by_num_req = SyncRpcRequest::GetBlockHeadersByNumber(req.clone());
    if let SyncRpcResponse::BlockHeaders(headers) = do_request(
        &network,
        peer_id,
        GET_BLOCK_HEADERS_BY_NUM_STR,
        get_block_headers_by_num_req,
    )
    .await?
    {
        //todo: Verify response
        Ok(headers)
    } else {
        Err(format_err!(
            "{:?}",
            "error SyncRpcResponse type when sync block header."
        ))
    }
}

pub async fn get_headers_with_peer(
    network: &NetworkAsyncService,
    peer_id: PeerId,
    req: GetBlockHeaders,
) -> Result<Vec<BlockHeader>> {
    let get_block_headers_req = SyncRpcRequest::GetBlockHeaders(req.clone());
    if let SyncRpcResponse::BlockHeaders(headers) = do_request(
        &network,
        peer_id,
        GET_BLOCK_HEADERS_STR,
        get_block_headers_req,
    )
    .await?
    {
        //todo: Verify response
        Ok(headers)
    } else {
        Err(format_err!(
            "{:?}",
            "error SyncRpcResponse type when sync block header."
        ))
    }
}

pub async fn get_headers(
    network: &NetworkAsyncService,
    req: GetBlockHeaders,
) -> Result<Vec<BlockHeader>> {
    if let Some(peer_info) = network.best_peer().await? {
        get_headers_with_peer(network, peer_info.get_peer_id(), req).await
    } else {
        Err(format_err!(
            "{:?}",
            "Can not get peer when sync block header."
        ))
    }
}

pub async fn get_body_by_hash(
    network: &NetworkAsyncService,
    hashs: Vec<HashValue>,
) -> Result<Vec<BlockBody>> {
    if let Some(peer_info) = network.best_peer().await? {
        let get_body_by_hash_req = SyncRpcRequest::GetBlockBodies(hashs);
        if let SyncRpcResponse::BlockBodies(bodies) = do_request(
            &network,
            peer_info.get_peer_id(),
            GET_BLOCK_BODIES_STR,
            get_body_by_hash_req,
        )
        .await?
        {
            Ok(bodies)
        } else {
            Err(format_err!(
                "{:?}",
                "error SyncRpcResponse type when sync block body."
            ))
        }
    } else {
        Err(format_err!(
            "{:?}",
            "Can not get peer when sync block body."
        ))
    }
}

pub async fn get_info_by_hash(
    network: &NetworkAsyncService,
    hashs: Vec<HashValue>,
) -> Result<Vec<BlockInfo>> {
    if let Some(peer_info) = network.best_peer().await? {
        let get_info_by_hash_req = SyncRpcRequest::GetBlockInfos(hashs);
        if let SyncRpcResponse::BlockInfos(infos) = do_request(
            &network,
            peer_info.get_peer_id(),
            GET_BLOCK_INFOS_STR,
            get_info_by_hash_req,
        )
        .await?
        {
            Ok(infos)
        } else {
            Err(format_err!(
                "{:?}",
                "error SyncRpcResponse type when sync block info."
            ))
        }
    } else {
        Err(format_err!(
            "{:?}",
            "Can not get peer when sync block info."
        ))
    }
}

pub async fn get_state_node_by_node_hash(
    network: &NetworkAsyncService,
    peer_id: PeerId,
    node_key: HashValue,
) -> Result<StateNode> {
    if let SyncRpcResponse::StateNode(state_node) = do_request(
        &network,
        peer_id,
        GET_STATE_NODE_BY_NODE_HASH_STR,
        SyncRpcRequest::GetStateNodeByNodeHash(node_key),
    )
    .await?
    {
        Ok(state_node)
    } else {
        Err(format_err!("{:?}", "error SyncRpcResponse type."))
    }
}

pub async fn get_accumulator_node_by_node_hash(
    network: &NetworkAsyncService,
    peer_id: PeerId,
    node_key: HashValue,
    accumulator_type: AccumulatorStoreType,
) -> Result<AccumulatorNode> {
    if let SyncRpcResponse::AccumulatorNode(accumulator_node) = do_request(
        &network,
        peer_id,
        GET_ACCUMULATOR_NODE_BY_NODE_HASH_STR,
        SyncRpcRequest::GetAccumulatorNodeByNodeHash(node_key, accumulator_type),
    )
    .await?
    {
        Ok(accumulator_node)
    } else {
        Err(format_err!("{:?}", "error SyncRpcResponse type."))
    }
}

/////////////////////////////////////////////////////////////////////////

async fn do_response(
    responder: Sender<(Cow<'static, [u8]>, Vec<u8>)>,
    resp: Vec<u8>,
) -> Result<()> {
    if let Err(e) = responder
        .clone()
        .send((CHAIN_PROTOCOL_NAME.into(), resp))
        .await
    {
        Err(format_err!("{:?}", e))
    } else {
        Ok(())
    }
}

pub async fn do_get_headers(
    responder: Sender<(Cow<'static, [u8]>, Vec<u8>)>,
    headers: Vec<BlockHeader>,
) -> Result<()> {
    let resp = SyncRpcResponse::encode(&SyncRpcResponse::BlockHeaders(headers))?;
    do_response(responder, resp).await
}

pub async fn do_get_info_by_hash(
    responder: Sender<(Cow<'static, [u8]>, Vec<u8>)>,
    infos: Vec<BlockInfo>,
) -> Result<()> {
    let resp = SyncRpcResponse::encode(&SyncRpcResponse::BlockInfos(infos))?;
    do_response(responder, resp).await
}

pub async fn do_get_body_by_hash(
    responder: Sender<(Cow<'static, [u8]>, Vec<u8>)>,
    bodies: Vec<BlockBody>,
) -> Result<()> {
    let resp = SyncRpcResponse::encode(&SyncRpcResponse::BlockBodies(bodies))?;
    do_response(responder, resp).await
}

pub async fn do_state_node(
    responder: Sender<(Cow<'static, [u8]>, Vec<u8>)>,
    state_node: StateNode,
) -> Result<()> {
    let resp = SyncRpcResponse::encode(&SyncRpcResponse::StateNode(state_node))?;
    do_response(responder, resp).await
}

pub async fn do_accumulator_node(
    responder: Sender<(Cow<'static, [u8]>, Vec<u8>)>,
    accumulator_node: AccumulatorNode,
) -> Result<()> {
    let resp = SyncRpcResponse::encode(&SyncRpcResponse::AccumulatorNode(accumulator_node))?;
    do_response(responder, resp).await
}

pub async fn do_response_get_txns(
    responder: Sender<(Cow<'static, [u8]>, Vec<u8>)>,
    txns_data: TransactionsData,
) -> Result<()> {
    let resp = SyncRpcResponse::encode(&SyncRpcResponse::GetTxns(txns_data))?;
    do_response(responder, resp).await
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

pub async fn do_get_txn_info(
    responder: Sender<(Cow<'static, [u8]>, Vec<u8>)>,
    txn_info: Option<TransactionInfo>,
) -> Result<()> {
    let resp = SyncRpcResponse::encode(&SyncRpcResponse::GetTxnInfo(txn_info))?;
    do_response(responder, resp).await
}
