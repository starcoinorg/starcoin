// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::Result;
use jsonrpsee::{
    core::{
        client::{ClientT, Subscription, SubscriptionClientT},
        params::{ArrayParams, ObjectParams},
        traits::ToRpcParams,
    },
    rpc_params,
};
use jsonrpsee_http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee_ws_client::WsClientBuilder;
use log::debug;
use network_api::PeerStrategy;
use network_p2p_types::network_state::NetworkState;
use network_p2p_types::peer_id::PeerId;
use network_types::peer_info::Multiaddr;
use serde::de::DeserializeOwned;
use serde_json::Value;
use starcoin_abi_types::{FunctionABI, ModuleABI, StructInstantiation};
use starcoin_account_api::AccountInfo;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::account::AccountApiRpcClient;
use starcoin_rpc_api::chain::ChainApiRpcClient;
use starcoin_rpc_api::chain::{
    GetBlockOption, GetBlocksOption, GetEventOption, GetTransactionOption,
};
use starcoin_rpc_api::contract_api::ContractApiRpcClient;
use starcoin_rpc_api::debug::DebugApiRpcClient;
use starcoin_rpc_api::miner::MinerApiRpcClient;
use starcoin_rpc_api::multi_types::MultiSignedUserTransactionView;
use starcoin_rpc_api::network_manager::NetworkManagerApiRpcClient;
use starcoin_rpc_api::node::NodeApiRpcClient;
use starcoin_rpc_api::node::NodeInfo;
use starcoin_rpc_api::node_manager::NodeManagerApiRpcClient;
use starcoin_rpc_api::pubsub::StarcoinPubSubApiClient;
use starcoin_rpc_api::state::StateApiRpcClient;
use starcoin_rpc_api::state::{
    GetCodeOption, GetResourceOption, ListCodeOption, ListResourceOption,
};
use starcoin_rpc_api::sync_manager::SyncManagerApiRpcClient;
use starcoin_rpc_api::txpool::TxPoolApiRpcClient;
use starcoin_rpc_api::types::pubsub::{EventFilter, EventFilterV2};
use starcoin_rpc_api::types::{
    AccountStateSetView, AnnotatedMoveStructView, BlockHeaderView, BlockInfoView, BlockView,
    ChainId, ChainInfoView, CodeView, ContractCall, DecodedMoveValue, DryRunOutputView,
    DryRunTransactionRequest, FactoryAction, FunctionIdView, ListCodeView, ListResourceView,
    MintedBlockView, ModuleIdView, MultiStateView, PeerInfoView, ResourceView, SignedMessageView,
    StateWithProofView, StateWithTableItemProofView, StrView, StructTagView, SyncStatusView,
    TransactionEventResponse, TransactionEventView, TransactionInfoView, TransactionInfoViewEnum,
    TransactionInfoWithProofView, TransactionRequest, TransactionView,
};
use starcoin_service_registry::{ServiceInfo, ServiceStatus};
use starcoin_sync_api::{PeerScoreResponse, SyncProgressReport};
use starcoin_txpool_api::TxPoolStatus;
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::block::BlockNumber;
use starcoin_types::multi_access_path::MultiAccessPath;
use starcoin_types::sign_message::SigningMessage;
use starcoin_types::system_events::MintBlockEvent;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_vm2_account_api::AccountInfo as AccountInfo2;
use starcoin_vm2_rpc_api::account_api::AccountApiRpcClient as AccountApiRpcClient2;
use starcoin_vm2_rpc_api::block_info_view2::BlockInfoView2;
use starcoin_vm2_rpc_api::contract_api::ContractApiRpcClient as ContractApiRpcClient2;
use starcoin_vm2_rpc_api::state_api::StateApiRpcClient as StateApiRpcClient2;
use starcoin_vm2_rpc_api::transaction_view2::TransactionView2;
use starcoin_vm2_types::account_address::AccountAddress as AccountAddress2;
use starcoin_vm2_types::view::TransactionInfoView as TransactionInfoView2;
use starcoin_vm2_vm_types::transaction::SignedUserTransaction as SignedUserTransaction2;
use starcoin_vm_types::language_storage::{ModuleId, StructTag};
use starcoin_vm_types::state_store::table::TableHandle;
use starcoin_vm_types::token::token_code::TokenCode;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub enum RpcChannel {
    Async(Arc<jsonrpsee::async_client::Client>),
    Http(Arc<HttpClient>),
}

impl RpcChannel {
    pub fn new_async(client: jsonrpsee::async_client::Client) -> Self {
        Self::Async(Arc::new(client))
    }

    pub async fn request<R, Params>(&self, method: &str, params: Params) -> Result<R, RpcError>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        match self {
            Self::Async(client) => client.request(method, params).await.map_err(Into::into),
            Self::Http(client) => client.request(method, params).await.map_err(Into::into),
        }
    }

    pub async fn subscribe<Notif, Params>(
        &self,
        subscribe_method: &str,
        params: Params,
        unsubscribe_method: &str,
    ) -> Result<Subscription<Notif>, RpcError>
    where
        Notif: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        match self {
            Self::Async(client) => client
                .subscribe(subscribe_method, params, unsubscribe_method)
                .await
                .map_err(Into::into),
            Self::Http(client) => client
                .subscribe(subscribe_method, params, unsubscribe_method)
                .await
                .map_err(Into::into),
        }
    }

    pub fn supports_pubsub(&self) -> bool {
        matches!(self, Self::Async(_))
    }
}

pub type RpcError = anyhow::Error;

pub async fn connect_ipc(sock_path: PathBuf) -> Result<RpcChannel, RpcError> {
    let client = starcoin_rpc_ipc::client::IpcClientBuilder::default()
        .build(
            sock_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("invalid ipc path"))?,
        )
        .await?;
    Ok(RpcChannel::new_async(client))
}

pub async fn connect_ws(url: &str) -> Result<RpcChannel, RpcError> {
    let normalized_url = normalize_ws_connect_url(url);
    if normalized_url.as_str() != url {
        debug!(
            "rewrite websocket endpoint from {} to {} for local connect",
            url, normalized_url
        );
    }
    let client = WsClientBuilder::default().build(normalized_url).await?;
    Ok(RpcChannel::new_async(client))
}

pub async fn connect_http(url: &str) -> Result<RpcChannel, RpcError> {
    let normalized_url = normalize_http_connect_url(url);
    if normalized_url.as_str() != url {
        debug!(
            "rewrite http endpoint from {} to {} for local connect",
            url, normalized_url
        );
    }
    let client = HttpClientBuilder::default().build(normalized_url)?;
    Ok(RpcChannel::Http(Arc::new(client)))
}

fn normalize_ws_connect_url(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("ws://0.0.0.0") {
        return format!("ws://127.0.0.1{}", rest);
    }
    if let Some(rest) = url.strip_prefix("wss://0.0.0.0") {
        return format!("wss://127.0.0.1{}", rest);
    }
    if let Some(rest) = url.strip_prefix("ws://[::]") {
        return format!("ws://[::1]{}", rest);
    }
    if let Some(rest) = url.strip_prefix("wss://[::]") {
        return format!("wss://[::1]{}", rest);
    }
    url.to_string()
}

fn normalize_http_connect_url(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("http://0.0.0.0") {
        return format!("http://127.0.0.1{}", rest);
    }
    if let Some(rest) = url.strip_prefix("https://0.0.0.0") {
        return format!("https://127.0.0.1{}", rest);
    }
    if let Some(rest) = url.strip_prefix("http://[::]") {
        return format!("http://[::1]{}", rest);
    }
    if let Some(rest) = url.strip_prefix("https://[::]") {
        return format!("https://[::1]{}", rest);
    }
    url.to_string()
}

#[cfg(test)]
mod connect_tests {
    use super::{normalize_http_connect_url, normalize_ws_connect_url};

    #[test]
    fn test_normalize_ipv4_any_addr() {
        assert_eq!(
            normalize_ws_connect_url("ws://0.0.0.0:9870"),
            "ws://127.0.0.1:9870"
        );
        assert_eq!(
            normalize_ws_connect_url("wss://0.0.0.0:9870/path"),
            "wss://127.0.0.1:9870/path"
        );
    }

    #[test]
    fn test_normalize_ipv6_any_addr() {
        assert_eq!(
            normalize_ws_connect_url("ws://[::]:9870"),
            "ws://[::1]:9870"
        );
        assert_eq!(
            normalize_ws_connect_url("wss://[::]:9870/path"),
            "wss://[::1]:9870/path"
        );
    }

    #[test]
    fn test_normalize_regular_addr() {
        let url = "ws://127.0.0.1:9870";
        assert_eq!(normalize_ws_connect_url(url), url);
    }

    #[test]
    fn test_normalize_http_ipv4_any_addr() {
        assert_eq!(
            normalize_http_connect_url("http://0.0.0.0:9850"),
            "http://127.0.0.1:9850"
        );
        assert_eq!(
            normalize_http_connect_url("https://0.0.0.0:9850/path"),
            "https://127.0.0.1:9850/path"
        );
    }

    #[test]
    fn test_normalize_http_ipv6_any_addr() {
        assert_eq!(
            normalize_http_connect_url("http://[::]:9850"),
            "http://[::1]:9850"
        );
        assert_eq!(
            normalize_http_connect_url("https://[::]:9850/path"),
            "https://[::1]:9850/path"
        );
    }
}

#[derive(Clone)]
pub struct RawClient {
    inner: RpcChannel,
}

impl From<RpcChannel> for RawClient {
    fn from(inner: RpcChannel) -> Self {
        Self { inner }
    }
}

impl RawClient {
    pub async fn call_method(self, api: &str, params: Value) -> Result<Value, RpcError> {
        match params {
            Value::Null => self.inner.request(api, rpc_params![]).await,
            Value::Array(values) => {
                let mut array = ArrayParams::new();
                for v in values {
                    array.insert(v)?;
                }
                self.inner.request(api, array).await
            }
            Value::Object(values) => {
                let mut object = ObjectParams::new();
                for (k, v) in values {
                    object.insert(&k, v)?;
                }
                self.inner.request(api, object).await
            }
            other => self.inner.request(api, rpc_params![other]).await,
        }
    }
}

macro_rules! def_client {
    ($name:ident) => {
        #[derive(Clone)]
        pub struct $name {
            inner: RpcChannel,
        }
        impl From<RpcChannel> for $name {
            fn from(inner: RpcChannel) -> Self {
                Self { inner }
            }
        }
    };
}

def_client!(NodeClient);
def_client!(NodeManagerClient);
def_client!(TxPoolClient);
def_client!(AccountClient);
def_client!(AccountClient2);
def_client!(StateClient);
def_client!(StateClient2);
def_client!(DebugClient);
def_client!(ChainClient);
def_client!(ContractClient);
def_client!(ContractClient2);
def_client!(MinerClient);
def_client!(SyncManagerClient);
def_client!(NetworkManagerClient);

#[derive(Clone)]
pub struct PubSubClient {
    inner: RpcChannel,
}

impl From<RpcChannel> for PubSubClient {
    fn from(inner: RpcChannel) -> Self {
        Self { inner }
    }
}

impl NodeClient {
    pub async fn status(self) -> Result<bool, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                NodeApiRpcClient::status(&*client).await.map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                NodeApiRpcClient::status(&*client).await.map_err(Into::into)
            }
        }
    }
    pub async fn info(self) -> Result<NodeInfo, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => NodeApiRpcClient::info(&*client).await.map_err(Into::into),
            RpcChannel::Http(client) => NodeApiRpcClient::info(&*client).await.map_err(Into::into),
        }
    }
    pub async fn peers(self) -> Result<Vec<PeerInfoView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.peers().await.map_err(Into::into),
            RpcChannel::Http(client) => client.peers().await.map_err(Into::into),
        }
    }
    pub async fn metrics(self) -> Result<HashMap<String, String>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.metrics().await.map_err(Into::into),
            RpcChannel::Http(client) => client.metrics().await.map_err(Into::into),
        }
    }
}

impl NodeManagerClient {
    pub async fn list_service(self) -> Result<Vec<ServiceInfo>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.list_service().await.map_err(Into::into),
            RpcChannel::Http(client) => client.list_service().await.map_err(Into::into),
        }
    }
    pub async fn start_service(self, service_name: String) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                client.start_service(service_name).await.map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                client.start_service(service_name).await.map_err(Into::into)
            }
        }
    }
    pub async fn check_service(self, service_name: String) -> Result<ServiceStatus, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                client.check_service(service_name).await.map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                client.check_service(service_name).await.map_err(Into::into)
            }
        }
    }
    pub async fn stop_service(self, service_name: String) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                client.stop_service(service_name).await.map_err(Into::into)
            }
            RpcChannel::Http(client) => client.stop_service(service_name).await.map_err(Into::into),
        }
    }
    pub async fn shutdown_system(self) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.shutdown_system().await.map_err(Into::into),
            RpcChannel::Http(client) => client.shutdown_system().await.map_err(Into::into),
        }
    }
    pub async fn reset_to_block(self, block_hash: HashValue) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                client.reset_to_block(block_hash).await.map_err(Into::into)
            }
            RpcChannel::Http(client) => client.reset_to_block(block_hash).await.map_err(Into::into),
        }
    }
    pub async fn re_execute_block(self, block_hash: HashValue) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .re_execute_block(block_hash)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .re_execute_block(block_hash)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn delete_block(self, block_hash: HashValue) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.delete_block(block_hash).await.map_err(Into::into),
            RpcChannel::Http(client) => client.delete_block(block_hash).await.map_err(Into::into),
        }
    }
    pub async fn delete_failed_block(self, block_hash: HashValue) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .delete_failed_block(block_hash)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .delete_failed_block(block_hash)
                .await
                .map_err(Into::into),
        }
    }
}

impl TxPoolClient {
    pub async fn submit_transaction(
        self,
        txn: SignedUserTransaction,
    ) -> Result<HashValue, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.submit_transaction(txn).await.map_err(Into::into),
            RpcChannel::Http(client) => client.submit_transaction(txn).await.map_err(Into::into),
        }
    }
    pub async fn submit_transactions(
        self,
        txns: Vec<SignedUserTransaction>,
    ) -> Result<Vec<HashValue>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.submit_transactions(txns).await.map_err(Into::into),
            RpcChannel::Http(client) => client.submit_transactions(txns).await.map_err(Into::into),
        }
    }
    pub async fn submit_hex_transaction(self, txn: String) -> Result<HashValue, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                client.submit_hex_transaction(txn).await.map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                client.submit_hex_transaction(txn).await.map_err(Into::into)
            }
        }
    }
    pub async fn pending_txn_multi(
        self,
        txn_hash: HashValue,
    ) -> Result<Option<MultiSignedUserTransactionView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                client.pending_txn_multi(txn_hash).await.map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                client.pending_txn_multi(txn_hash).await.map_err(Into::into)
            }
        }
    }
    pub async fn pending_txns_multi(
        self,
        sender: AccountAddress,
        max_len: Option<u32>,
    ) -> Result<Vec<MultiSignedUserTransactionView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .pending_txns_multi(sender, max_len)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .pending_txns_multi(sender, max_len)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn next_sequence_number(
        self,
        address: AccountAddress,
    ) -> Result<Option<u64>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .next_sequence_number(address)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .next_sequence_number(address)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn next_sequence_number_in_batch(
        self,
        addresses: Vec<AccountAddress>,
    ) -> Result<Option<Vec<(AccountAddress, Option<u64>)>>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .next_sequence_number_in_batch(addresses)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .next_sequence_number_in_batch(addresses)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn state(self) -> Result<TxPoolStatus, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => TxPoolApiRpcClient::state(&*client)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => TxPoolApiRpcClient::state(&*client)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn submit_transaction2(
        self,
        txn: SignedUserTransaction2,
    ) -> Result<HashValue, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.submit_transaction2(txn).await.map_err(Into::into),
            RpcChannel::Http(client) => client.submit_transaction2(txn).await.map_err(Into::into),
        }
    }
    pub async fn submit_hex_transaction2(self, txn: String) -> Result<HashValue, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .submit_hex_transaction2(txn)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .submit_hex_transaction2(txn)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn next_sequence_number2(
        self,
        address: AccountAddress2,
    ) -> Result<Option<u64>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .next_sequence_number2(address)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .next_sequence_number2(address)
                .await
                .map_err(Into::into),
        }
    }
}

impl AccountClient {
    pub async fn default(self) -> Result<Option<AccountInfo>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient::default(&*client)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient::default(&*client)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn set_default_account(self, addr: AccountAddress) -> Result<AccountInfo, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient::set_default_account(&*client, addr)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient::set_default_account(&*client, addr)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn create(self, password: String) -> Result<AccountInfo, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient::create(&*client, password)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient::create(&*client, password)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn list(self) -> Result<Vec<AccountInfo>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient::list(&*client)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient::list(&*client)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get(self, address: AccountAddress) -> Result<Option<AccountInfo>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient::get(&*client, address)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient::get(&*client, address)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn sign_txn(
        self,
        raw_txn: RawUserTransaction,
        signer: AccountAddress,
    ) -> Result<SignedUserTransaction, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient::sign_txn(&*client, raw_txn, signer)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient::sign_txn(&*client, raw_txn, signer)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn sign_txn_request(
        self,
        txn_request: TransactionRequest,
    ) -> Result<String, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                AccountApiRpcClient::sign_txn_request(&*client, txn_request)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                AccountApiRpcClient::sign_txn_request(&*client, txn_request)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn sign_txn_in_batch(
        self,
        raw_txns: Vec<RawUserTransaction>,
    ) -> Result<Vec<SignedUserTransaction>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient::sign_txn_in_batch(&*client, raw_txns)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient::sign_txn_in_batch(&*client, raw_txns)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn sign(
        self,
        signer: AccountAddress,
        message: SigningMessage,
    ) -> Result<SignedMessageView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient::sign(&*client, signer, message)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient::sign(&*client, signer, message)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn change_account_password(
        self,
        address: AccountAddress,
        new_password: String,
    ) -> Result<AccountInfo, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                AccountApiRpcClient::change_account_password(&*client, address, new_password)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                AccountApiRpcClient::change_account_password(&*client, address, new_password)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn lock(self, address: AccountAddress) -> Result<AccountInfo, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient::lock(&*client, address)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient::lock(&*client, address)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn unlock(
        self,
        address: AccountAddress,
        password: String,
        duration: Option<u32>,
    ) -> Result<AccountInfo, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                AccountApiRpcClient::unlock(&*client, address, password, duration)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                AccountApiRpcClient::unlock(&*client, address, password, duration)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn unlock_in_batch(
        self,
        batch: Vec<(AccountAddress, String)>,
        duration: Option<u32>,
    ) -> Result<Vec<AccountInfo>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                AccountApiRpcClient::unlock_in_batch(&*client, batch, duration)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                AccountApiRpcClient::unlock_in_batch(&*client, batch, duration)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn export(
        self,
        address: AccountAddress,
        password: String,
    ) -> Result<Vec<u8>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient::export(&*client, address, password)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient::export(&*client, address, password)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn import(
        self,
        address: AccountAddress,
        private_key: StrView<Vec<u8>>,
        password: String,
    ) -> Result<AccountInfo, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                AccountApiRpcClient::import(&*client, address, private_key, password)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                AccountApiRpcClient::import(&*client, address, private_key, password)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn import_readonly(
        self,
        address: AccountAddress,
        public_key: StrView<Vec<u8>>,
    ) -> Result<AccountInfo, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                AccountApiRpcClient::import_readonly(&*client, address, public_key)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                AccountApiRpcClient::import_readonly(&*client, address, public_key)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn accepted_tokens(
        self,
        address: AccountAddress,
    ) -> Result<Vec<TokenCode>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient::accepted_tokens(&*client, address)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient::accepted_tokens(&*client, address)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn remove(
        self,
        address: AccountAddress,
        password: Option<String>,
    ) -> Result<AccountInfo, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient::remove(&*client, address, password)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient::remove(&*client, address, password)
                .await
                .map_err(Into::into),
        }
    }
}

impl AccountClient2 {
    pub async fn default(self) -> Result<Option<AccountInfo2>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient2::default(&*client)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient2::default(&*client)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn set_default_account(
        self,
        addr: AccountAddress2,
    ) -> Result<AccountInfo2, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient2::set_default_account(&*client, addr)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient2::set_default_account(&*client, addr)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn list(self) -> Result<Vec<AccountInfo2>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient2::list(&*client)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient2::list(&*client)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get(self, address: AccountAddress2) -> Result<Option<AccountInfo2>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient2::get(&*client, address)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient2::get(&*client, address)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn sign(
        self,
        signer: AccountAddress2,
        message: starcoin_vm2_vm_types::sign_message::SigningMessage,
    ) -> Result<starcoin_vm2_types::view::SignedMessageView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient2::sign(&*client, signer, message)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient2::sign(&*client, signer, message)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn sign_txn_request(
        self,
        txn_request: starcoin_vm2_types::view::TransactionRequest,
    ) -> Result<String, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                AccountApiRpcClient2::sign_txn_request(&*client, txn_request)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                AccountApiRpcClient2::sign_txn_request(&*client, txn_request)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn unlock(
        self,
        address: AccountAddress2,
        password: String,
        duration: Option<u32>,
    ) -> Result<AccountInfo2, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                AccountApiRpcClient2::unlock(&*client, address, password, duration)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                AccountApiRpcClient2::unlock(&*client, address, password, duration)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn lock(self, address: AccountAddress2) -> Result<AccountInfo2, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient2::lock(&*client, address)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient2::lock(&*client, address)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn import(
        self,
        address: AccountAddress2,
        private_key: starcoin_vm2_types::view::StrView<Vec<u8>>,
        password: String,
    ) -> Result<AccountInfo2, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                AccountApiRpcClient2::import(&*client, address, private_key, password)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                AccountApiRpcClient2::import(&*client, address, private_key, password)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn import_readonly(
        self,
        address: AccountAddress2,
        public_key: starcoin_vm2_types::view::StrView<Vec<u8>>,
    ) -> Result<AccountInfo2, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                AccountApiRpcClient2::import_readonly(&*client, address, public_key)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                AccountApiRpcClient2::import_readonly(&*client, address, public_key)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn export(
        self,
        address: AccountAddress2,
        password: String,
    ) -> Result<Vec<u8>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient2::export(&*client, address, password)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient2::export(&*client, address, password)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn change_account_password(
        self,
        address: AccountAddress2,
        new_password: String,
    ) -> Result<AccountInfo2, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                AccountApiRpcClient2::change_account_password(&*client, address, new_password)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                AccountApiRpcClient2::change_account_password(&*client, address, new_password)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn accepted_tokens(
        self,
        address: AccountAddress2,
    ) -> Result<Vec<starcoin_vm2_vm_types::account_config::token_code::TokenCode>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient2::accepted_tokens(&*client, address)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient2::accepted_tokens(&*client, address)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn remove(
        self,
        address: AccountAddress2,
        password: Option<String>,
    ) -> Result<AccountInfo2, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient2::remove(&*client, address, password)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient2::remove(&*client, address, password)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn sign_txn(
        self,
        raw_txn: starcoin_vm2_vm_types::transaction::RawUserTransaction,
        signer: AccountAddress2,
    ) -> Result<SignedUserTransaction2, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient2::sign_txn(&*client, raw_txn, signer)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient2::sign_txn(&*client, raw_txn, signer)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn create(self, password: String) -> Result<AccountInfo2, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => AccountApiRpcClient2::create(&*client, password)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => AccountApiRpcClient2::create(&*client, password)
                .await
                .map_err(Into::into),
        }
    }
}

impl StateClient {
    pub async fn get(self, access_path: AccessPath) -> Result<Option<Vec<u8>>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => StateApiRpcClient::get(&*client, access_path)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => StateApiRpcClient::get(&*client, access_path)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_with_proof(
        self,
        access_path: AccessPath,
    ) -> Result<StateWithProofView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => StateApiRpcClient::get_with_proof(&*client, access_path)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => StateApiRpcClient::get_with_proof(&*client, access_path)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_with_proof_by_root(
        self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> Result<StateWithProofView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                { StateApiRpcClient::get_with_proof_by_root(&*client, access_path, state_root) }
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                { StateApiRpcClient::get_with_proof_by_root(&*client, access_path, state_root) }
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn get_with_proof_by_root_raw(
        self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> Result<StrView<Vec<u8>>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                { StateApiRpcClient::get_with_proof_by_root_raw(&*client, access_path, state_root) }
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                { StateApiRpcClient::get_with_proof_by_root_raw(&*client, access_path, state_root) }
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn get_state_root(self) -> Result<HashValue, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => StateApiRpcClient::get_state_root(&*client)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => StateApiRpcClient::get_state_root(&*client)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_account_state(
        self,
        address: AccountAddress,
    ) -> Result<Option<AccountState>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => StateApiRpcClient::get_account_state(&*client, address)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => StateApiRpcClient::get_account_state(&*client, address)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_account_state_set(
        self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> Result<Option<AccountStateSetView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                { StateApiRpcClient::get_account_state_set(&*client, address, state_root) }
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                { StateApiRpcClient::get_account_state_set(&*client, address, state_root) }
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn get_resource(
        self,
        address: AccountAddress,
        resource_type: StrView<StructTag>,
        option: Option<GetResourceOption>,
    ) -> Result<Option<ResourceView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                StateApiRpcClient::get_resource(&*client, address, resource_type, option)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                StateApiRpcClient::get_resource(&*client, address, resource_type, option)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn list_resource(
        self,
        address: AccountAddress,
        option: Option<ListResourceOption>,
    ) -> Result<ListResourceView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                { StateApiRpcClient::list_resource(&*client, address, option) }
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                { StateApiRpcClient::list_resource(&*client, address, option) }
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn get_code(
        self,
        module_id: StrView<ModuleId>,
        option: Option<GetCodeOption>,
    ) -> Result<Option<CodeView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => StateApiRpcClient::get_code(&*client, module_id, option)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => StateApiRpcClient::get_code(&*client, module_id, option)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn list_code(
        self,
        address: AccountAddress,
        option: Option<ListCodeOption>,
    ) -> Result<ListCodeView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => StateApiRpcClient::list_code(&*client, address, option)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => StateApiRpcClient::list_code(&*client, address, option)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_with_table_item_proof_by_root(
        self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> Result<StateWithTableItemProofView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                StateApiRpcClient::get_with_table_item_proof_by_root(
                    &*client, handle, key, state_root,
                )
            }
            .await
            .map_err(Into::into),
            RpcChannel::Http(client) => {
                StateApiRpcClient::get_with_table_item_proof_by_root(
                    &*client, handle, key, state_root,
                )
            }
            .await
            .map_err(Into::into),
        }
    }
    pub async fn get_state_node_by_node_hash(
        self,
        key_hash: HashValue,
    ) -> Result<Option<Vec<u8>>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                { StateApiRpcClient::get_state_node_by_node_hash(&*client, key_hash) }
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                { StateApiRpcClient::get_state_node_by_node_hash(&*client, key_hash) }
                    .await
                    .map_err(Into::into)
            }
        }
    }
}

impl StateClient2 {
    pub async fn get(
        self,
        state_key: starcoin_vm2_vm_types::state_store::state_key::StateKey,
    ) -> Result<Option<Vec<u8>>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => StateApiRpcClient2::get(&*client, state_key)
                .await
                .map(|maybe_bytes| maybe_bytes.map(|bytes| bytes.to_vec()))
                .map_err(Into::into),
            RpcChannel::Http(client) => StateApiRpcClient2::get(&*client, state_key)
                .await
                .map(|maybe_bytes| maybe_bytes.map(|bytes| bytes.to_vec()))
                .map_err(Into::into),
        }
    }
    pub async fn get_state_node_by_node_hash(
        self,
        key_hash: HashValue,
    ) -> Result<Option<Vec<u8>>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                StateApiRpcClient2::get_state_node_by_node_hash(&*client, key_hash)
                    .await
                    .map(|maybe_bytes| maybe_bytes.map(|bytes| bytes.to_vec()))
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                StateApiRpcClient2::get_state_node_by_node_hash(&*client, key_hash)
                    .await
                    .map(|maybe_bytes| maybe_bytes.map(|bytes| bytes.to_vec()))
                    .map_err(Into::into)
            }
        }
    }
    pub async fn get_with_proof(
        self,
        state_key: starcoin_vm2_vm_types::state_store::state_key::StateKey,
    ) -> Result<starcoin_vm2_types::view::StateWithProofView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => StateApiRpcClient2::get_with_proof(&*client, state_key)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => StateApiRpcClient2::get_with_proof(&*client, state_key)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_account_state(
        self,
        address: AccountAddress2,
    ) -> Result<starcoin_vm2_types::account_state::AccountState, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => StateApiRpcClient2::get_account_state(&*client, address)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => StateApiRpcClient2::get_account_state(&*client, address)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_account_state_set(
        self,
        address: AccountAddress2,
        state_root: Option<HashValue>,
    ) -> Result<Option<starcoin_vm2_types::view::AccountStateSetView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                StateApiRpcClient2::get_account_state_set(&*client, address, state_root)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                StateApiRpcClient2::get_account_state_set(&*client, address, state_root)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn get_state_root(self) -> Result<HashValue, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => StateApiRpcClient2::get_state_root(&*client)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => StateApiRpcClient2::get_state_root(&*client)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_with_proof_by_root(
        self,
        state_key: starcoin_vm2_vm_types::state_store::state_key::StateKey,
        state_root: HashValue,
    ) -> Result<starcoin_vm2_types::view::StateWithProofView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                StateApiRpcClient2::get_with_proof_by_root(&*client, state_key, state_root)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                StateApiRpcClient2::get_with_proof_by_root(&*client, state_key, state_root)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn get_with_proof_by_root_raw(
        self,
        state_key: starcoin_vm2_vm_types::state_store::state_key::StateKey,
        state_root: HashValue,
    ) -> Result<starcoin_vm2_types::view::StrView<Vec<u8>>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                StateApiRpcClient2::get_with_proof_by_root_raw(&*client, state_key, state_root)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                StateApiRpcClient2::get_with_proof_by_root_raw(&*client, state_key, state_root)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn get_with_table_item_proof_by_root(
        self,
        handle: starcoin_vm2_vm_types::state_store::table::TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> Result<starcoin_vm2_types::view::StateWithTableItemProofView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => StateApiRpcClient2::get_with_table_item_proof_by_root(
                &*client, handle, key, state_root,
            )
            .await
            .map_err(Into::into),
            RpcChannel::Http(client) => StateApiRpcClient2::get_with_table_item_proof_by_root(
                &*client, handle, key, state_root,
            )
            .await
            .map_err(Into::into),
        }
    }
    pub async fn get_code(
        self,
        module_id: starcoin_vm2_types::view::StrView<
            starcoin_vm2_vm_types::language_storage::ModuleId,
        >,
        option: Option<starcoin_vm2_rpc_api::state_api::GetCodeOption>,
    ) -> Result<Option<starcoin_vm2_types::view::CodeView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => StateApiRpcClient2::get_code(&*client, module_id, option)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => StateApiRpcClient2::get_code(&*client, module_id, option)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_resource(
        self,
        address: AccountAddress2,
        resource_type: starcoin_vm2_types::view::StrView<
            starcoin_vm2_vm_types::language_storage::StructTag,
        >,
        option: Option<starcoin_vm2_rpc_api::state_api::GetResourceOption>,
    ) -> Result<Option<starcoin_vm2_types::view::ResourceView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                StateApiRpcClient2::get_resource(&*client, address, resource_type, option)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                StateApiRpcClient2::get_resource(&*client, address, resource_type, option)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn list_resource(
        self,
        address: AccountAddress2,
        option: Option<starcoin_vm2_rpc_api::state_api::ListResourceOption>,
    ) -> Result<starcoin_vm2_types::view::ListResourceView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                StateApiRpcClient2::list_resource(&*client, address, option)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                StateApiRpcClient2::list_resource(&*client, address, option)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn list_code(
        self,
        address: AccountAddress2,
        option: Option<starcoin_vm2_rpc_api::state_api::ListCodeOption>,
    ) -> Result<starcoin_vm2_types::view::ListCodeView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => StateApiRpcClient2::list_code(&*client, address, option)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => StateApiRpcClient2::list_code(&*client, address, option)
                .await
                .map_err(Into::into),
        }
    }
}

impl ContractClient {
    pub async fn get_code(
        self,
        module_id: StrView<ModuleId>,
    ) -> Result<Option<StrView<Vec<u8>>>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => ContractApiRpcClient::get_code(&*client, module_id)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => ContractApiRpcClient::get_code(&*client, module_id)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_resource(
        self,
        addr: AccountAddress,
        resource_type: StrView<StructTag>,
    ) -> Result<Option<AnnotatedMoveStructView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                ContractApiRpcClient::get_resource(&*client, addr, resource_type)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                ContractApiRpcClient::get_resource(&*client, addr, resource_type)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn call_v2(self, call: ContractCall) -> Result<Vec<DecodedMoveValue>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => ContractApiRpcClient::call_v2(&*client, call)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => ContractApiRpcClient::call_v2(&*client, call)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn resolve_function(
        self,
        function_id: FunctionIdView,
    ) -> Result<FunctionABI, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                ContractApiRpcClient::resolve_function(&*client, function_id)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                ContractApiRpcClient::resolve_function(&*client, function_id)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn resolve_struct(
        self,
        struct_tag: StructTagView,
    ) -> Result<StructInstantiation, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => ContractApiRpcClient::resolve_struct(&*client, struct_tag)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => ContractApiRpcClient::resolve_struct(&*client, struct_tag)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn resolve_module(self, module_id: ModuleIdView) -> Result<ModuleABI, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => ContractApiRpcClient::resolve_module(&*client, module_id)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => ContractApiRpcClient::resolve_module(&*client, module_id)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn dry_run(
        self,
        txn: DryRunTransactionRequest,
    ) -> Result<DryRunOutputView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => ContractApiRpcClient::dry_run(&*client, txn)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => ContractApiRpcClient::dry_run(&*client, txn)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn dry_run_raw(
        self,
        raw_txn: String,
        sender_public_key: StrView<starcoin_vm_types::transaction::authenticator::AccountPublicKey>,
    ) -> Result<DryRunOutputView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                ContractApiRpcClient::dry_run_raw(&*client, raw_txn, sender_public_key)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                ContractApiRpcClient::dry_run_raw(&*client, raw_txn, sender_public_key)
                    .await
                    .map_err(Into::into)
            }
        }
    }
}

impl ContractClient2 {
    pub async fn get_code(
        self,
        module_id: starcoin_vm2_types::view::StrView<
            starcoin_vm2_vm_types::language_storage::ModuleId,
        >,
    ) -> Result<Option<starcoin_vm2_types::view::StrView<Vec<u8>>>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => ContractApiRpcClient2::get_code(&*client, module_id)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => ContractApiRpcClient2::get_code(&*client, module_id)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_resource(
        self,
        addr: AccountAddress2,
        resource_type: starcoin_vm2_types::view::StrView<
            starcoin_vm2_vm_types::language_storage::StructTag,
        >,
    ) -> Result<Option<starcoin_vm2_types::view::AnnotatedMoveStructView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                ContractApiRpcClient2::get_resource(&*client, addr, resource_type)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                ContractApiRpcClient2::get_resource(&*client, addr, resource_type)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn call_v2(
        self,
        call: starcoin_vm2_types::view::ContractCall,
    ) -> Result<Vec<starcoin_vm2_rpc_api::DecodedMoveValue>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => ContractApiRpcClient2::call_v2(&*client, call)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => ContractApiRpcClient2::call_v2(&*client, call)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn resolve_function(
        self,
        function_id: starcoin_vm2_types::view::FunctionIdView,
    ) -> Result<starcoin_vm2_abi_types::FunctionABI, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                ContractApiRpcClient2::resolve_function(&*client, function_id)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                ContractApiRpcClient2::resolve_function(&*client, function_id)
                    .await
                    .map_err(Into::into)
            }
        }
    }
    pub async fn resolve_struct(
        self,
        struct_tag: starcoin_vm2_types::view::StructTagView,
    ) -> Result<starcoin_vm2_abi_types::StructInstantiation, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                ContractApiRpcClient2::resolve_struct(&*client, struct_tag)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => ContractApiRpcClient2::resolve_struct(&*client, struct_tag)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn resolve_module(
        self,
        module_id: starcoin_vm2_types::view::ModuleIdView,
    ) -> Result<starcoin_vm2_abi_types::ModuleABI, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => ContractApiRpcClient2::resolve_module(&*client, module_id)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => ContractApiRpcClient2::resolve_module(&*client, module_id)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn dry_run(
        self,
        txn: starcoin_vm2_types::view::DryRunTransactionRequest,
    ) -> Result<starcoin_vm2_types::view::DryRunOutputView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => ContractApiRpcClient2::dry_run(&*client, txn)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => ContractApiRpcClient2::dry_run(&*client, txn)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn dry_run_raw(
        self,
        raw_txn: String,
        sender_public_key: starcoin_vm2_types::view::StrView<
            starcoin_vm2_vm_types::transaction::authenticator::AccountPublicKey,
        >,
    ) -> Result<starcoin_vm2_types::view::DryRunOutputView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                ContractApiRpcClient2::dry_run_raw(&*client, raw_txn, sender_public_key)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                ContractApiRpcClient2::dry_run_raw(&*client, raw_txn, sender_public_key)
                    .await
                    .map_err(Into::into)
            }
        }
    }
}

impl DebugClient {
    pub async fn set_log_level(
        self,
        logger_name: Option<String>,
        level: String,
    ) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .set_log_level(logger_name, level)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .set_log_level(logger_name, level)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn set_log_pattern(
        self,
        pattern: starcoin_logger::LogPattern,
    ) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.set_log_pattern(pattern).await.map_err(Into::into),
            RpcChannel::Http(client) => client.set_log_pattern(pattern).await.map_err(Into::into),
        }
    }
    pub async fn panic(self) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.panic().await.map_err(Into::into),
            RpcChannel::Http(client) => client.panic().await.map_err(Into::into),
        }
    }
    pub async fn txfactory_status(self, action: FactoryAction) -> Result<bool, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.txfactory_status(action).await.map_err(Into::into),
            RpcChannel::Http(client) => client.txfactory_status(action).await.map_err(Into::into),
        }
    }
    pub async fn sleep(self, time: u64) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.sleep(time).await.map_err(Into::into),
            RpcChannel::Http(client) => client.sleep(time).await.map_err(Into::into),
        }
    }
    pub async fn set_concurrency_level(self, level: usize) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .set_concurrency_level(level)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .set_concurrency_level(level)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_concurrency_level(self) -> Result<usize, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.get_concurrency_level().await.map_err(Into::into),
            RpcChannel::Http(client) => client.get_concurrency_level().await.map_err(Into::into),
        }
    }
    pub async fn set_logger_balance_amount(self, balance_amount: u64) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .set_logger_balance_amount(balance_amount)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .set_logger_balance_amount(balance_amount)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_logger_balance_amount(self) -> Result<u64, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                client.get_logger_balance_amount().await.map_err(Into::into)
            }
            RpcChannel::Http(client) => {
                client.get_logger_balance_amount().await.map_err(Into::into)
            }
        }
    }
}

impl ChainClient {
    pub async fn id(self) -> Result<ChainId, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.id().await.map_err(Into::into),
            RpcChannel::Http(client) => client.id().await.map_err(Into::into),
        }
    }
    pub async fn info(self) -> Result<ChainInfoView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                ChainApiRpcClient::info(&*client).await.map_err(Into::into)
            }
            RpcChannel::Http(client) => ChainApiRpcClient::info(&*client).await.map_err(Into::into),
        }
    }
    pub async fn get_headers(
        self,
        block_hashes: Vec<HashValue>,
    ) -> Result<Vec<BlockHeaderView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.get_headers(block_hashes).await.map_err(Into::into),
            RpcChannel::Http(client) => client.get_headers(block_hashes).await.map_err(Into::into),
        }
    }
    pub async fn get_block_by_hash(
        self,
        hash: HashValue,
        option: Option<GetBlockOption>,
    ) -> Result<Option<BlockView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_block_by_hash(hash, option)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_block_by_hash(hash, option)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_block_by_number(
        self,
        number: BlockNumber,
        option: Option<GetBlockOption>,
    ) -> Result<Option<BlockView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_block_by_number(number, option)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_block_by_number(number, option)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_block_info_by_number(
        self,
        number: BlockNumber,
    ) -> Result<Option<BlockInfoView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_block_info_by_number(number)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_block_info_by_number(number)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_block_info_by_number2(
        self,
        number: BlockNumber,
    ) -> Result<Option<BlockInfoView2>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_block_info_by_number2(number)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_block_info_by_number2(number)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_block_info_by_hash(
        self,
        id: HashValue,
    ) -> Result<Option<BlockInfoView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                client.get_block_info_by_hash(id).await.map_err(Into::into)
            }
            RpcChannel::Http(client) => client.get_block_info_by_hash(id).await.map_err(Into::into),
        }
    }
    pub async fn get_blocks_by_number(
        self,
        number: Option<BlockNumber>,
        count: u64,
        option: Option<GetBlocksOption>,
    ) -> Result<Vec<BlockView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_blocks_by_number(number, count, option)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_blocks_by_number(number, count, option)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_transaction(
        self,
        txn_id: HashValue,
        option: Option<GetTransactionOption>,
    ) -> Result<Option<TransactionView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_transaction(txn_id, option)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_transaction(txn_id, option)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_transaction2(
        self,
        txn_id: HashValue,
        option: Option<GetTransactionOption>,
    ) -> Result<Option<TransactionView2>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_transaction2(txn_id, option)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_transaction2(txn_id, option)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_transaction_info(
        self,
        txn_hash: HashValue,
    ) -> Result<Option<TransactionInfoView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_transaction_info(txn_hash)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_transaction_info(txn_hash)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_transaction_info2(
        self,
        txn_hash: HashValue,
    ) -> Result<Option<TransactionInfoView2>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_transaction_info2(txn_hash)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_transaction_info2(txn_hash)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_events_by_txn_hash(
        self,
        txn_hash: HashValue,
        option: Option<GetEventOption>,
    ) -> Result<Vec<TransactionEventResponse>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_events_by_txn_hash(txn_hash, option)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_events_by_txn_hash(txn_hash, option)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_events_by_txn_hash2(
        self,
        txn_hash: HashValue,
        option: Option<GetEventOption>,
    ) -> Result<Vec<starcoin_vm2_types::view::TransactionEventResponse>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_events_by_txn_hash2(txn_hash, option)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_events_by_txn_hash2(txn_hash, option)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_block_txn_infos(
        self,
        block_id: HashValue,
    ) -> Result<Vec<TransactionInfoView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_block_txn_infos(block_id)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_block_txn_infos(block_id)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_block_txn_infos2(
        self,
        block_id: HashValue,
    ) -> Result<Vec<TransactionInfoView2>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_block_txn_infos2(block_id)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_block_txn_infos2(block_id)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_block_txn_infos_in_seq(
        self,
        block_id: HashValue,
    ) -> Result<Vec<TransactionInfoViewEnum>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_block_txn_infos_in_seq(block_id)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_block_txn_infos_in_seq(block_id)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_txn_info_by_block_and_index(
        self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<TransactionInfoView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_txn_info_by_block_and_index(block_id, idx)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_txn_info_by_block_and_index(block_id, idx)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_txn_info_by_block_and_index2(
        self,
        block_id: HashValue,
        idx: u64,
    ) -> Result<Option<TransactionInfoView2>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_txn_info_by_block_and_index2(block_id, idx)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_txn_info_by_block_and_index2(block_id, idx)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_transaction_infos(
        self,
        start_global_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<TransactionInfoView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_transaction_infos(start_global_index, reverse, max_size)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_transaction_infos(start_global_index, reverse, max_size)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_transaction_infos2(
        self,
        start_global_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> Result<Vec<TransactionInfoView2>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_transaction_infos2(start_global_index, reverse, max_size)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_transaction_infos2(start_global_index, reverse, max_size)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_transaction_proof(
        self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<starcoin_types::access_path::AccessPath>,
    ) -> Result<Option<TransactionInfoWithProofView>, RpcError> {
        let access_path = access_path.map(StrView);
        match self.inner {
            RpcChannel::Async(client) => client
                .get_transaction_proof(
                    block_hash,
                    transaction_global_index,
                    event_index,
                    access_path,
                )
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_transaction_proof(
                    block_hash,
                    transaction_global_index,
                    event_index,
                    access_path,
                )
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_transaction_proof_raw(
        self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<starcoin_types::access_path::AccessPath>,
    ) -> Result<Option<StrView<Vec<u8>>>, RpcError> {
        let access_path = access_path.map(StrView);
        match self.inner {
            RpcChannel::Async(client) => client
                .get_transaction_proof_raw(
                    block_hash,
                    transaction_global_index,
                    event_index,
                    access_path,
                )
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_transaction_proof_raw(
                    block_hash,
                    transaction_global_index,
                    event_index,
                    access_path,
                )
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_transaction_proof2(
        self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<MultiAccessPath>,
    ) -> Result<Option<TransactionInfoWithProofView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_transaction_proof2(
                    block_hash,
                    transaction_global_index,
                    event_index,
                    access_path,
                )
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_transaction_proof2(
                    block_hash,
                    transaction_global_index,
                    event_index,
                    access_path,
                )
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_transaction_proof2_raw(
        self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<MultiAccessPath>,
    ) -> Result<Option<starcoin_vm2_types::view::StrView<Vec<u8>>>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_transaction_proof2_raw(
                    block_hash,
                    transaction_global_index,
                    event_index,
                    access_path,
                )
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_transaction_proof2_raw(
                    block_hash,
                    transaction_global_index,
                    event_index,
                    access_path,
                )
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_vm_multi_state(
        self,
        block_hash: HashValue,
    ) -> Result<Option<MultiStateView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .get_vm_multi_state(block_hash)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .get_vm_multi_state(block_hash)
                .await
                .map_err(Into::into),
        }
    }
}

impl MinerClient {
    pub async fn submit(
        self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> Result<MintedBlockView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .submit(minting_blob, nonce, extra)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .submit(minting_blob, nonce, extra)
                .await
                .map_err(Into::into),
        }
    }
}

impl SyncManagerClient {
    pub async fn status(self) -> Result<SyncStatusView, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => SyncManagerApiRpcClient::status(&*client)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => SyncManagerApiRpcClient::status(&*client)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn progress(self) -> Result<Option<SyncProgressReport>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.progress().await.map_err(Into::into),
            RpcChannel::Http(client) => client.progress().await.map_err(Into::into),
        }
    }
    pub async fn peer_score(self) -> Result<PeerScoreResponse, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.peer_score().await.map_err(Into::into),
            RpcChannel::Http(client) => client.peer_score().await.map_err(Into::into),
        }
    }
    pub async fn start(
        self,
        force: bool,
        peers: Vec<PeerId>,
        skip_pow_verify: bool,
        strategy: Option<PeerStrategy>,
    ) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .start(force, peers, skip_pow_verify, strategy)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .start(force, peers, skip_pow_verify, strategy)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn cancel(self) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.cancel().await.map_err(Into::into),
            RpcChannel::Http(client) => client.cancel().await.map_err(Into::into),
        }
    }
}

impl NetworkManagerClient {
    pub async fn known_peers(self) -> Result<Vec<PeerId>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.known_peers().await.map_err(Into::into),
            RpcChannel::Http(client) => client.known_peers().await.map_err(Into::into),
        }
    }
    pub async fn state(self) -> Result<NetworkState, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => NetworkManagerApiRpcClient::state(&*client)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => NetworkManagerApiRpcClient::state(&*client)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn get_address(self, peer_id: String) -> Result<Vec<Multiaddr>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.get_address(peer_id).await.map_err(Into::into),
            RpcChannel::Http(client) => client.get_address(peer_id).await.map_err(Into::into),
        }
    }
    pub async fn add_peer(self, peer: String) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.add_peer(peer).await.map_err(Into::into),
            RpcChannel::Http(client) => client.add_peer(peer).await.map_err(Into::into),
        }
    }
    pub async fn call_peer(
        self,
        peer_id: String,
        rpc_method: String,
        message: StrView<Vec<u8>>,
    ) -> Result<StrView<Vec<u8>>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .call_peer(peer_id, rpc_method, message)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .call_peer(peer_id, rpc_method, message)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn set_peer_reputation(self, peer_id: String, reput: i32) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client
                .set_peer_reputation(peer_id, reput)
                .await
                .map_err(Into::into),
            RpcChannel::Http(client) => client
                .set_peer_reputation(peer_id, reput)
                .await
                .map_err(Into::into),
        }
    }
    pub async fn ban_peer(self, peer_id: String, ban: bool) -> Result<(), RpcError> {
        match self.inner {
            RpcChannel::Async(client) => client.ban_peer(peer_id, ban).await.map_err(Into::into),
            RpcChannel::Http(client) => client.ban_peer(peer_id, ban).await.map_err(Into::into),
        }
    }
}

impl PubSubClient {
    pub async fn subscribe_events(
        self,
        filter: EventFilter,
        decode: bool,
    ) -> Result<Subscription<TransactionEventView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                StarcoinPubSubApiClient::subscribe_events(&*client, filter, decode)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(_) => Err(anyhow::anyhow!(
                "http/https transport does not support pubsub"
            )),
        }
    }

    pub async fn subscribe_events_v2(
        self,
        filter: EventFilterV2,
        decode: bool,
    ) -> Result<Subscription<starcoin_vm2_types::view::TransactionEventView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                StarcoinPubSubApiClient::subscribe_events_v2(&*client, filter, decode)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(_) => Err(anyhow::anyhow!(
                "http/https transport does not support pubsub"
            )),
        }
    }

    pub async fn subscribe_new_block(self) -> Result<Subscription<BlockView>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => StarcoinPubSubApiClient::subscribe_new_heads(&*client)
                .await
                .map_err(Into::into),
            RpcChannel::Http(_) => Err(anyhow::anyhow!(
                "http/https transport does not support pubsub"
            )),
        }
    }

    pub async fn subscribe_new_transactions(
        self,
    ) -> Result<Subscription<Vec<HashValue>>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                StarcoinPubSubApiClient::subscribe_new_pending_transactions(&*client)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(_) => Err(anyhow::anyhow!(
                "http/https transport does not support pubsub"
            )),
        }
    }

    pub async fn subscribe_new_mint_block(self) -> Result<Subscription<MintBlockEvent>, RpcError> {
        match self.inner {
            RpcChannel::Async(client) => {
                StarcoinPubSubApiClient::subscribe_new_mint_block(&*client)
                    .await
                    .map_err(Into::into)
            }
            RpcChannel::Http(_) => Err(anyhow::anyhow!(
                "http/https transport does not support pubsub"
            )),
        }
    }
}
