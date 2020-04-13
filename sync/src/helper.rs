use crate::{do_duration, DELAY_TIME};
use anyhow::{format_err, Result};
use crypto::hash::HashValue;
use futures::channel::mpsc::Sender;
use futures::sink::SinkExt;
use network::NetworkAsyncService;
use starcoin_canonical_serialization::SCSCodec;
use starcoin_state_tree::StateNode;
use starcoin_sync_api::sync_messages::{
    BatchBlockInfo, BatchBodyMsg, BatchHashByNumberMsg, BatchHeaderMsg, DataType, GetDataByHashMsg,
    GetHashByNumberMsg, SyncRpcRequest, SyncRpcResponse,
};
use types::peer_info::PeerId;

async fn do_request(
    network: &NetworkAsyncService,
    peer_id: PeerId,
    req: SyncRpcRequest,
) -> Result<SyncRpcResponse> {
    let request = req.encode()?;
    let response = network
        .send_request_bytes(peer_id.into(), request, do_duration(DELAY_TIME))
        .await?;
    SyncRpcResponse::decode(&response)
}

pub async fn get_hash_by_number(
    network: &NetworkAsyncService,
    peer_id: PeerId,
    req: GetHashByNumberMsg,
) -> Result<BatchHashByNumberMsg> {
    let request = SyncRpcRequest::GetHashByNumberMsg(req);

    if let SyncRpcResponse::BatchHashByNumberMsg(batch_hash_by_number_msg) =
        do_request(&network, peer_id, request).await?
    {
        Ok(batch_hash_by_number_msg)
    } else {
        Err(format_err!("{:?}", "error SyncRpcResponse type."))
    }
}

pub async fn get_block_by_hash(
    network: &NetworkAsyncService,
    peer_id: PeerId,
    hashs: Vec<HashValue>,
) -> Result<(BatchHeaderMsg, BatchBodyMsg, BatchBlockInfo)> {
    let get_data_by_hash_req = SyncRpcRequest::GetDataByHashMsg(GetDataByHashMsg {
        hashs,
        data_type: DataType::HEADER,
    });
    if let SyncRpcResponse::BatchHeaderAndBodyMsg(headers, bodies, infos) =
        do_request(&network, peer_id, get_data_by_hash_req).await?
    {
        Ok((headers, bodies, infos))
    } else {
        Err(format_err!("{:?}", "error SyncRpcResponse type."))
    }
}

pub async fn get_header_by_hash(
    network: &NetworkAsyncService,
    peer_id: PeerId,
    hashs: Vec<HashValue>,
) -> Result<BatchHeaderMsg> {
    let get_data_by_hash_req = SyncRpcRequest::GetDataByHashMsg(GetDataByHashMsg {
        hashs,
        data_type: DataType::HEADER,
    });
    if let SyncRpcResponse::BatchHeaderAndBodyMsg(headers, _bodies, _infos) =
        do_request(&network, peer_id, get_data_by_hash_req).await?
    {
        Ok(headers)
    } else {
        Err(format_err!("{:?}", "error SyncRpcResponse type."))
    }
}

pub async fn get_body_by_hash(
    _network: &NetworkAsyncService,
    _peer_id: PeerId,
    _hashs: Vec<HashValue>,
) -> Result<BatchBodyMsg> {
    unimplemented!()
}

pub async fn get_info_by_hash(
    _network: &NetworkAsyncService,
    _peer_id: PeerId,
    _hashs: Vec<HashValue>,
) -> Result<BatchBlockInfo> {
    unimplemented!()
}

pub async fn get_state_node_by_node_hash(
    network: &NetworkAsyncService,
    peer_id: PeerId,
    node_key: HashValue,
) -> Result<StateNode> {
    if let SyncRpcResponse::GetStateNodeByNodeHash(state_node) = do_request(
        &network,
        peer_id,
        SyncRpcRequest::GetStateNodeByNodeHash(node_key),
    )
    .await?
    {
        Ok(state_node)
    } else {
        Err(format_err!("{:?}", "error SyncRpcResponse type."))
    }
}

/////////////////////////////////////////////////////////////////////////

async fn do_response(responder: Sender<Vec<u8>>, resp: Vec<u8>) -> Result<()> {
    if let Err(e) = responder.clone().send(resp).await {
        Err(format_err!("{:?}", e))
    } else {
        Ok(())
    }
}

pub async fn do_get_hash_by_number(
    responder: Sender<Vec<u8>>,
    batch_hash_by_number_msg: BatchHashByNumberMsg,
) -> Result<()> {
    let resp = SyncRpcResponse::encode(&SyncRpcResponse::BatchHashByNumberMsg(
        batch_hash_by_number_msg,
    ))?;
    do_response(responder, resp).await
}

pub async fn do_get_block_by_hash(
    responder: Sender<Vec<u8>>,
    batch_header_msg: BatchHeaderMsg,
    batch_body_msg: BatchBodyMsg,
    batch_block_info_msg: BatchBlockInfo,
) -> Result<()> {
    let resp = SyncRpcResponse::encode(&SyncRpcResponse::BatchHeaderAndBodyMsg(
        batch_header_msg,
        batch_body_msg,
        batch_block_info_msg,
    ))?;
    do_response(responder, resp).await
}

pub async fn do_state_node(responder: Sender<Vec<u8>>, state_node: StateNode) -> Result<()> {
    let resp = SyncRpcResponse::encode(&SyncRpcResponse::GetStateNodeByNodeHash(state_node))?;
    do_response(responder, resp).await
}
