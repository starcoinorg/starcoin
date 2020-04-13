use crate::{do_duration, DELAY_TIME};
use anyhow::{format_err, Result};
use crypto::hash::HashValue;
use network::NetworkAsyncService;
use network_p2p_api::sync_messages::{
    BatchBlockInfo, BatchBodyMsg, BatchHashByNumberMsg, BatchHeaderMsg, DataType, GetDataByHashMsg,
    GetHashByNumberMsg, ProcessMessage, SyncRpcRequest, SyncRpcResponse,
};
use starcoin_canonical_serialization::SCSCodec;
use types::peer_info::PeerId;

pub async fn send_sync_request(
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
    let request = SyncRpcRequest::GetHashByNumberMsg(ProcessMessage::GetHashByNumberMsg(req));

    if let SyncRpcResponse::BatchHashByNumberMsg(batch_hash_by_number_msg) =
        send_sync_request(&network, peer_id, request).await?
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
    let get_data_by_hash_req =
        SyncRpcRequest::GetDataByHashMsg(ProcessMessage::GetDataByHashMsg(GetDataByHashMsg {
            hashs,
            data_type: DataType::HEADER,
        }));
    if let SyncRpcResponse::BatchHeaderAndBodyMsg(headers, bodies, infos) =
        send_sync_request(&network, peer_id, get_data_by_hash_req).await?
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
    let get_data_by_hash_req =
        SyncRpcRequest::GetDataByHashMsg(ProcessMessage::GetDataByHashMsg(GetDataByHashMsg {
            hashs,
            data_type: DataType::HEADER,
        }));
    if let SyncRpcResponse::BatchHeaderAndBodyMsg(headers, _bodies, _infos) =
        send_sync_request(&network, peer_id, get_data_by_hash_req).await?
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
