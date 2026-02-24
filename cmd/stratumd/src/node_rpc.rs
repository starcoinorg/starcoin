use anyhow::Result;
use starcoin_config::Connect;
use starcoin_rpc_client::{ConnSource, RpcClient};
use std::str::FromStr;

pub fn parse_conn_source(node_rpc: &str) -> Result<ConnSource> {
    match Connect::from_str(node_rpc)? {
        Connect::WebSocket(url) => Ok(ConnSource::WebSocket(url)),
        Connect::IPC(Some(path)) => Ok(ConnSource::Ipc(path)),
        Connect::IPC(None) => Err(anyhow::anyhow!(
            "node rpc ipc path is empty, please set --node-rpc <path-to-ipc-file>"
        )),
    }
}

pub fn build_sync_rpc_client(node_rpc: &str) -> Result<RpcClient> {
    match Connect::from_str(node_rpc)? {
        Connect::WebSocket(url) => RpcClient::connect_websocket(url.as_str()),
        Connect::IPC(Some(path)) => RpcClient::connect_ipc(path),
        Connect::IPC(None) => Err(anyhow::anyhow!(
            "node rpc ipc path is empty, please set --node-rpc <path-to-ipc-file>"
        )),
    }
}
