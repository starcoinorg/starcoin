use anyhow::Result;
use async_trait::async_trait;
use futures::{Stream, TryStreamExt};
use starcoin_config::Connect;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::{
    chain::{GetBlockOption, GetEventOption},
    types::{
        BlockView, ChainInfoView, MintedBlockView, TransactionEventResponse, TransactionInfoView,
    },
};
use starcoin_rpc_client::{AsyncRpcClient, ConnSource, RpcClient};
use starcoin_types::system_events::MintBlockEvent;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;

pub type MintBlockEventStream = Pin<Box<dyn Stream<Item = anyhow::Result<MintBlockEvent>> + Send>>;

#[async_trait]
pub trait NodeRpc: Send + Sync {
    async fn miner_get_job(&self) -> Result<Option<MintBlockEvent>>;
    async fn subscribe_new_mint_blocks(&self) -> Result<MintBlockEventStream>;
    async fn miner_submit(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> Result<MintedBlockView>;
    async fn chain_info(&self) -> Result<ChainInfoView>;
    async fn chain_get_block_by_number(
        &self,
        number: u64,
        option: Option<GetBlockOption>,
    ) -> Result<Option<BlockView>>;
    async fn chain_get_block_txn_infos(
        &self,
        block_hash: HashValue,
    ) -> Result<Vec<TransactionInfoView>>;
    async fn chain_get_events_by_txn_hash(
        &self,
        txn_hash: HashValue,
        option: Option<GetEventOption>,
    ) -> Result<Vec<TransactionEventResponse>>;
}

pub trait NodeRpcSync: Send + Sync {
    fn chain_get_block_by_number(
        &self,
        number: u64,
        option: Option<GetBlockOption>,
    ) -> Result<Option<BlockView>>;
    fn chain_get_block_txn_infos(&self, block_hash: HashValue) -> Result<Vec<TransactionInfoView>>;
    fn chain_get_events_by_txn_hash(
        &self,
        txn_hash: HashValue,
        option: Option<GetEventOption>,
    ) -> Result<Vec<TransactionEventResponse>>;
}

#[async_trait]
impl NodeRpc for AsyncRpcClient {
    async fn miner_get_job(&self) -> Result<Option<MintBlockEvent>> {
        AsyncRpcClient::miner_get_job(self).await
    }

    async fn subscribe_new_mint_blocks(&self) -> Result<MintBlockEventStream> {
        let stream = AsyncRpcClient::subscribe_new_mint_blocks(self).await?;
        Ok(Box::pin(stream.into_stream()))
    }

    async fn miner_submit(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> Result<MintedBlockView> {
        AsyncRpcClient::miner_submit(self, minting_blob, nonce, extra).await
    }

    async fn chain_info(&self) -> Result<ChainInfoView> {
        AsyncRpcClient::chain_info(self).await
    }

    async fn chain_get_block_by_number(
        &self,
        number: u64,
        option: Option<GetBlockOption>,
    ) -> Result<Option<BlockView>> {
        AsyncRpcClient::chain_get_block_by_number(self, number, option).await
    }

    async fn chain_get_block_txn_infos(
        &self,
        block_hash: HashValue,
    ) -> Result<Vec<TransactionInfoView>> {
        AsyncRpcClient::chain_get_block_txn_infos(self, block_hash).await
    }

    async fn chain_get_events_by_txn_hash(
        &self,
        txn_hash: HashValue,
        option: Option<GetEventOption>,
    ) -> Result<Vec<TransactionEventResponse>> {
        AsyncRpcClient::chain_get_events_by_txn_hash(self, txn_hash, option).await
    }
}

impl NodeRpcSync for RpcClient {
    fn chain_get_block_by_number(
        &self,
        number: u64,
        option: Option<GetBlockOption>,
    ) -> Result<Option<BlockView>> {
        RpcClient::chain_get_block_by_number(self, number, option)
    }

    fn chain_get_block_txn_infos(&self, block_hash: HashValue) -> Result<Vec<TransactionInfoView>> {
        RpcClient::chain_get_block_txn_infos(self, block_hash)
    }

    fn chain_get_events_by_txn_hash(
        &self,
        txn_hash: HashValue,
        option: Option<GetEventOption>,
    ) -> Result<Vec<TransactionEventResponse>> {
        RpcClient::chain_get_events_by_txn_hash(self, txn_hash, option)
    }
}

pub fn parse_conn_source(node_rpc: &str) -> Result<ConnSource> {
    match Connect::from_str(node_rpc)? {
        Connect::WebSocket(url) => Ok(ConnSource::WebSocket(url)),
        Connect::IPC(Some(path)) => Ok(ConnSource::Ipc(path)),
        Connect::IPC(None) => Err(anyhow::anyhow!(
            "node rpc ipc path is empty, please set --node-rpc <path-to-ipc-file>"
        )),
    }
}

pub async fn build_async_rpc_client(node_rpc: &str) -> Result<Arc<dyn NodeRpc>> {
    let conn = parse_conn_source(node_rpc)?;
    let rpc = AsyncRpcClient::new(conn).await?;
    Ok(Arc::new(rpc))
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
