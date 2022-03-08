// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::chain_watcher::{ChainWatcher, StartSubscribe, WatchBlock, WatchTxn};
use crate::pubsub_client::PubSubClient;
pub use crate::remote_state_reader::{RemoteStateReader, StateRootOption};
use actix::{Addr, System};
use anyhow::anyhow;
use bcs_ext::BCSCodec;
use futures::channel::oneshot;
use futures::{TryStream, TryStreamExt};
use jsonrpc_client_transports::RawClient;
pub use jsonrpc_core::Params;
use jsonrpc_core_client::{transports::ipc, transports::ws, RpcChannel};
use network_api::PeerStrategy;
use network_p2p_types::network_state::NetworkState;
use parking_lot::Mutex;
use serde_json::Value;
use starcoin_abi_types::{FunctionABI, ModuleABI, StructInstantiation};
use starcoin_account_api::AccountInfo;
use starcoin_crypto::HashValue;
use starcoin_logger::{prelude::*, LogPattern};
use starcoin_rpc_api::chain::{GetBlockOption, GetEventOption, GetTransactionOption};
use starcoin_rpc_api::node::NodeInfo;
use starcoin_rpc_api::service::RpcAsyncService;
use starcoin_rpc_api::state::{
    GetCodeOption, GetResourceOption, ListCodeOption, ListResourceOption,
};
use starcoin_rpc_api::types::pubsub::EventFilter;
use starcoin_rpc_api::types::{
    AccountStateSetView, AnnotatedMoveStructView, BlockHeaderView, BlockInfoView, BlockView,
    ChainId, ChainInfoView, CodeView, ContractCall, DecodedMoveValue, DryRunOutputView,
    DryRunTransactionRequest, FactoryAction, FunctionIdView, ListCodeView, ListResourceView,
    MintedBlockView, ModuleIdView, PeerInfoView, ResourceView, SignedMessageView,
    SignedUserTransactionView, StateWithProofView, StrView, StructTagView,
    TransactionEventResponse, TransactionInfoView, TransactionInfoWithProofView,
    TransactionRequest, TransactionView,
};
use starcoin_rpc_api::{
    account::AccountClient, chain::ChainClient, contract_api::ContractClient, debug::DebugClient,
    miner::MinerClient, network_manager::NetworkManagerClient, node::NodeClient,
    node_manager::NodeManagerClient, state::StateClient, sync_manager::SyncManagerClient,
    txpool::TxPoolClient, types::TransactionEventView,
};
use starcoin_service_registry::{ServiceInfo, ServiceStatus};
use starcoin_sync_api::{PeerScoreResponse, SyncProgressReport};
use starcoin_txpool_api::TxPoolStatus;
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::block::BlockNumber;
use starcoin_types::peer_info::{Multiaddr, PeerId};
use starcoin_types::sign_message::SigningMessage;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::MintBlockEvent;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_vm_types::language_storage::{ModuleId, StructTag};
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::transaction::DryRunTransaction;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::Duration;
use tokio::runtime::Runtime;

pub mod chain_watcher;
mod pubsub_client;
mod remote_state_reader;

#[derive(Clone)]
enum ConnSource {
    Ipc(PathBuf),
    WebSocket(String),
    Local(Box<RpcChannel>),
}

impl std::fmt::Debug for ConnSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnSource::Ipc(path) => write!(f, "Ipc({})", path.as_path().to_string_lossy()),
            ConnSource::WebSocket(url) => write!(f, "WebSocket({})", url),
            ConnSource::Local(_) => write!(f, "Local"),
        }
    }
}

pub struct RpcClient {
    inner: Mutex<Option<RpcClientInner>>,
    provider: ConnectionProvider,
    chain_watcher: Addr<ChainWatcher>,
    //hold the watch thread handle.
    watcher_handle: JoinHandle<()>,
}

struct ConnectionProvider {
    conn_source: ConnSource,
    runtime: Mutex<Runtime>,
}

impl ConnectionProvider {
    fn new(conn_source: ConnSource, runtime: Runtime) -> Self {
        Self {
            conn_source,
            runtime: Mutex::new(runtime),
        }
    }

    fn block_on<F>(&self, future: F) -> F::Output
    where
        F: futures::Future + std::marker::Send,
        F::Output: std::marker::Send,
    {
        self.runtime.lock().block_on(future)
    }

    fn get_rpc_channel(&self) -> anyhow::Result<RpcChannel, jsonrpc_client_transports::RpcError> {
        self.block_on(async { self.get_rpc_channel_async().await })
    }

    async fn get_rpc_channel_async(
        &self,
    ) -> anyhow::Result<RpcChannel, jsonrpc_client_transports::RpcError> {
        match self.conn_source.clone() {
            ConnSource::Ipc(sock_path) => ipc::connect(sock_path).await,
            ConnSource::WebSocket(url) => ws::try_connect(url.as_str())?.await,
            ConnSource::Local(channel) => Ok(*channel),
        }
    }
}

impl RpcClient {
    pub(crate) fn new(conn_source: ConnSource) -> anyhow::Result<Self> {
        let (tx, rx) = oneshot::channel();
        let provider = ConnectionProvider::new(conn_source, Runtime::new()?);
        let inner: RpcClientInner = provider.get_rpc_channel().map_err(map_err)?.into(); //Self::create_client_inner(conn_source.clone()).map_err(map_err)?;
        let pubsub_client = inner.pubsub_client.clone();
        let handle = std::thread::spawn(move || {
            let sys = System::new("client-actix-system");
            let watcher = ChainWatcher::launch();

            tx.send(watcher).unwrap();
            let _ = sys.run();
        });
        let watcher = futures::executor::block_on(rx).expect("Init chain watcher fail.");
        watcher.do_send(StartSubscribe {
            client: pubsub_client,
        });
        Ok(Self {
            inner: Mutex::new(Some(inner)),
            provider,
            chain_watcher: watcher,
            watcher_handle: handle,
        })
    }

    pub fn connect_websocket(url: &str) -> anyhow::Result<Self> {
        Self::new(ConnSource::WebSocket(url.to_string()))
    }

    pub fn connect_local<S>(rpc_service: S) -> anyhow::Result<Self>
    where
        S: RpcAsyncService,
    {
        let client = futures::executor::block_on(async { rpc_service.connect_local().await })?;
        Self::new(ConnSource::Local(Box::new(client)))
    }

    pub fn connect_ipc<P: AsRef<Path>>(sock_path: P) -> anyhow::Result<Self> {
        let path = sock_path.as_ref().to_path_buf();
        Self::new(ConnSource::Ipc(path))
    }

    pub fn watch_txn(
        &self,
        txn_hash: HashValue,
        timeout: Option<Duration>,
    ) -> anyhow::Result<chain_watcher::ThinHeadBlock> {
        let chain_watcher = self.chain_watcher.clone();
        let f = async move {
            let r = chain_watcher.send(WatchTxn { txn_hash }).await?;
            match timeout {
                Some(t) => async_std::future::timeout(t, r).await??,
                None => r.await?,
            }
        };
        futures::executor::block_on(f)
    }

    pub fn watch_block(
        &self,
        block_number: BlockNumber,
    ) -> anyhow::Result<chain_watcher::ThinHeadBlock> {
        let chain_watcher = self.chain_watcher.clone();
        let f = async move {
            let r = chain_watcher.send(WatchBlock(block_number)).await?;
            r.await?
        };
        futures::executor::block_on(f)
    }

    pub fn node_status(&self) -> anyhow::Result<bool> {
        self.call_rpc_blocking(|inner| inner.node_client.status())
            .map_err(map_err)
    }

    pub fn node_info(&self) -> anyhow::Result<NodeInfo> {
        self.call_rpc_blocking(|inner| inner.node_client.info())
            .map_err(map_err)
    }

    pub async fn node_info_async(&self) -> anyhow::Result<NodeInfo> {
        self.call_rpc_async(|inner| inner.node_client.info())
            .await
            .map_err(map_err)
    }

    pub fn node_metrics(&self) -> anyhow::Result<HashMap<String, String>> {
        self.call_rpc_blocking(|inner| inner.node_client.metrics())
            .map_err(map_err)
    }

    pub fn node_peers(&self) -> anyhow::Result<Vec<PeerInfoView>> {
        self.call_rpc_blocking(|inner| inner.node_client.peers())
            .map_err(map_err)
    }

    pub fn node_list_service(&self) -> anyhow::Result<Vec<ServiceInfo>> {
        self.call_rpc_blocking(|inner| inner.node_manager_client.list_service())
            .map_err(map_err)
    }

    pub fn node_start_service(&self, service_name: String) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| inner.node_manager_client.start_service(service_name))
            .map_err(map_err)
    }

    pub fn node_check_service(&self, service_name: String) -> anyhow::Result<ServiceStatus> {
        self.call_rpc_blocking(|inner| inner.node_manager_client.check_service(service_name))
            .map_err(map_err)
    }

    pub fn node_stop_service(&self, service_name: String) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| inner.node_manager_client.stop_service(service_name))
            .map_err(map_err)
    }

    pub fn node_shutdown_system(&self) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| inner.node_manager_client.shutdown_system())
            .map_err(map_err)
    }

    pub fn node_reset(&self, block_hash: HashValue) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| inner.node_manager_client.reset_to_block(block_hash))
            .map_err(map_err)
    }

    pub fn node_re_execute_block(&self, block_id: HashValue) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| inner.node_manager_client.re_execute_block(block_id))
            .map_err(map_err)
    }

    pub fn node_delete_block(&self, block_id: HashValue) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| inner.node_manager_client.delete_block(block_id))
            .map_err(map_err)
    }

    pub fn node_delete_failed_block(&self, block_id: HashValue) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| inner.node_manager_client.delete_failed_block(block_id))
            .map_err(map_err)
    }

    pub fn next_sequence_number_in_txpool(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<Option<u64>> {
        self.call_rpc_blocking(|inner| inner.txpool_client.next_sequence_number(address))
            .map_err(map_err)
    }

    pub fn submit_transaction(&self, txn: SignedUserTransaction) -> anyhow::Result<HashValue> {
        self.call_rpc_blocking(|inner| inner.txpool_client.submit_transaction(txn))
            .map_err(map_err)
    }

    pub fn submit_hex_transaction(&self, txn: String) -> anyhow::Result<HashValue> {
        self.call_rpc_blocking(|inner| inner.txpool_client.submit_hex_transaction(txn))
            .map_err(map_err)
    }

    pub fn get_pending_txn_by_hash(
        &self,
        txn_hash: HashValue,
    ) -> anyhow::Result<Option<SignedUserTransactionView>> {
        self.call_rpc_blocking(|inner| inner.txpool_client.pending_txn(txn_hash))
            .map_err(map_err)
    }

    pub fn get_pending_txns_of_sender(
        &self,
        sender: AccountAddress,
        max_len: Option<u32>,
    ) -> anyhow::Result<Vec<SignedUserTransactionView>> {
        self.call_rpc_blocking(|inner| inner.txpool_client.pending_txns(sender, max_len))
            .map_err(map_err)
    }

    //TODO should split client for different api ?
    // such as  RpcClient().account().default()
    pub fn account_default(&self) -> anyhow::Result<Option<AccountInfo>> {
        self.call_rpc_blocking(|inner| inner.account_client.default())
            .map_err(map_err)
    }

    pub fn set_default_account(&self, addr: AccountAddress) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| inner.account_client.set_default_account(addr))
            .map_err(map_err)
    }

    pub fn account_create(&self, password: String) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| inner.account_client.create(password))
            .map_err(map_err)
    }

    pub fn account_list(&self) -> anyhow::Result<Vec<AccountInfo>> {
        self.call_rpc_blocking(|inner| inner.account_client.list())
            .map_err(map_err)
    }

    pub fn account_get(&self, address: AccountAddress) -> anyhow::Result<Option<AccountInfo>> {
        self.call_rpc_blocking(|inner| inner.account_client.get(address))
            .map_err(map_err)
    }

    /// partial sign a multisig account's txn
    pub fn account_sign_multisig_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer_address: AccountAddress,
    ) -> anyhow::Result<SignedUserTransaction> {
        self.call_rpc_blocking(|inner| inner.account_client.sign_txn(raw_txn, signer_address))
            .map_err(map_err)
    }

    pub fn account_sign_txn_request(
        &self,
        txn_request: TransactionRequest,
    ) -> anyhow::Result<SignedUserTransaction> {
        self.call_rpc_blocking(|inner| inner.account_client.sign_txn_request(txn_request))
            .map_err(map_err)
            .and_then(|d: String| {
                hex::decode(d.as_str().strip_prefix("0x").unwrap_or_else(|| d.as_str()))
                    .map_err(anyhow::Error::new)
                    .and_then(|d| bcs_ext::from_bytes::<SignedUserTransaction>(d.as_slice()))
            })
    }

    pub fn account_sign_txn(
        &self,
        raw_txn: RawUserTransaction,
    ) -> anyhow::Result<SignedUserTransaction> {
        let signer = raw_txn.sender();
        self.call_rpc_blocking(|inner| inner.account_client.sign_txn(raw_txn, signer))
            .map_err(map_err)
    }

    pub fn account_sign_message(
        &self,
        signer: AccountAddress,
        message: SigningMessage,
    ) -> anyhow::Result<SignedMessageView> {
        self.call_rpc_blocking(|inner| inner.account_client.sign(signer, message))
            .map_err(map_err)
    }

    pub fn account_change_password(
        &self,
        address: AccountAddress,
        new_password: String,
    ) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| {
            inner
                .account_client
                .change_account_password(address, new_password)
        })
        .map_err(map_err)
    }

    pub fn account_lock(&self, address: AccountAddress) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| inner.account_client.lock(address))
            .map_err(map_err)
    }
    pub fn account_unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: std::time::Duration,
    ) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| {
            inner
                .account_client
                .unlock(address, password, Some(duration.as_secs() as u32))
        })
        .map_err(map_err)
    }
    pub fn account_export(
        &self,
        address: AccountAddress,
        password: String,
    ) -> anyhow::Result<Vec<u8>> {
        self.call_rpc_blocking(|inner| inner.account_client.export(address, password))
            .map_err(map_err)
    }
    pub fn account_import(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| {
            inner
                .account_client
                .import(address, StrView(private_key), password)
        })
        .map_err(map_err)
    }

    pub fn account_import_readonly(
        &self,
        address: AccountAddress,
        public_key: Vec<u8>,
    ) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| {
            inner
                .account_client
                .import_readonly(address, StrView(public_key))
        })
        .map_err(map_err)
    }

    pub fn account_accepted_tokens(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<Vec<TokenCode>> {
        self.call_rpc_blocking(|inner| inner.account_client.accepted_tokens(address))
            .map_err(map_err)
    }

    pub fn account_remove(
        &self,
        address: AccountAddress,
        password: Option<String>,
    ) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| inner.account_client.remove(address, password))
            .map_err(map_err)
    }

    pub fn get_code(&self, module_id: ModuleId) -> anyhow::Result<Option<String>> {
        let result: Option<StrView<Vec<u8>>> = self
            .call_rpc_blocking(|inner| inner.contract_client.get_code(StrView(module_id)))
            .map_err(map_err)?;
        Ok(result.map(|s| s.to_string()))
    }

    pub fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StructTag,
    ) -> anyhow::Result<Option<AnnotatedMoveStructView>> {
        self.call_rpc_blocking(|inner| {
            inner
                .contract_client
                .get_resource(addr, StrView(resource_type))
        })
        .map_err(map_err)
    }

    pub fn state_reader(
        &self,
        state_root_opt: StateRootOption,
    ) -> anyhow::Result<RemoteStateReader> {
        RemoteStateReader::new(self, state_root_opt)
    }

    pub fn state_get(&self, access_path: AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        self.call_rpc_blocking(|inner| inner.state_client.get(access_path))
            .map_err(map_err)
    }

    pub fn state_get_with_proof(
        &self,
        access_path: AccessPath,
    ) -> anyhow::Result<StateWithProofView> {
        self.call_rpc_blocking(|inner| inner.state_client.get_with_proof(access_path))
            .map_err(map_err)
    }

    pub fn state_get_with_proof_by_root(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> anyhow::Result<StateWithProofView> {
        self.call_rpc_blocking(|inner| {
            inner
                .state_client
                .get_with_proof_by_root(access_path, state_root)
        })
        .map_err(map_err)
    }

    pub fn state_get_state_root(&self) -> anyhow::Result<HashValue> {
        self.call_rpc_blocking(|inner| inner.state_client.get_state_root())
            .map_err(map_err)
    }

    pub fn state_get_account_state(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<Option<AccountState>> {
        self.call_rpc_blocking(|inner| inner.state_client.get_account_state(address))
            .map_err(map_err)
    }

    pub fn state_get_account_state_set(
        &self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> anyhow::Result<Option<AccountStateSetView>> {
        self.call_rpc_blocking(|inner| {
            inner
                .state_client
                .get_account_state_set(address, state_root)
        })
        .map_err(map_err)
    }

    pub fn state_get_resource(
        &self,
        address: AccountAddress,
        resource_type: StructTag,
        decode: bool,
        state_root: Option<HashValue>,
    ) -> anyhow::Result<Option<ResourceView>> {
        self.call_rpc_blocking(|inner| {
            inner.state_client.get_resource(
                address,
                StrView(resource_type),
                Some(GetResourceOption { decode, state_root }),
            )
        })
        .map_err(map_err)
    }

    pub fn state_list_resource(
        &self,
        address: AccountAddress,
        decode: bool,
        state_root: Option<HashValue>,
    ) -> anyhow::Result<ListResourceView> {
        self.call_rpc_blocking(|inner| {
            inner
                .state_client
                .list_resource(address, Some(ListResourceOption { decode, state_root }))
        })
        .map_err(map_err)
    }

    pub fn state_get_code(
        &self,
        module_id: ModuleId,
        resolve: bool,
        state_root: Option<HashValue>,
    ) -> anyhow::Result<Option<CodeView>> {
        self.call_rpc_blocking(|inner| {
            inner.state_client.get_code(
                StrView(module_id),
                Some(GetCodeOption {
                    resolve,
                    state_root,
                }),
            )
        })
        .map_err(map_err)
    }

    pub fn state_list_code(
        &self,
        address: AccountAddress,
        resolve: bool,
        state_root: Option<HashValue>,
    ) -> anyhow::Result<ListCodeView> {
        self.call_rpc_blocking(|inner| {
            inner.state_client.list_code(
                address,
                Some(ListCodeOption {
                    resolve,
                    state_root,
                }),
            )
        })
        .map_err(map_err)
    }

    pub fn contract_call(&self, call: ContractCall) -> anyhow::Result<Vec<DecodedMoveValue>> {
        self.call_rpc_blocking(|inner| inner.contract_client.call_v2(call))
            .map_err(map_err)
    }

    pub fn contract_resolve_function(
        &self,
        function_id: FunctionIdView,
    ) -> anyhow::Result<FunctionABI> {
        self.call_rpc_blocking(|inner| inner.contract_client.resolve_function(function_id))
            .map_err(map_err)
    }

    pub fn contract_resolve_struct(
        &self,
        struct_tag: StructTagView,
    ) -> anyhow::Result<StructInstantiation> {
        self.call_rpc_blocking(|inner| inner.contract_client.resolve_struct(struct_tag))
            .map_err(map_err)
    }

    pub fn contract_resolve_module(&self, module_id: ModuleIdView) -> anyhow::Result<ModuleABI> {
        self.call_rpc_blocking(|inner| inner.contract_client.resolve_module(module_id))
            .map_err(map_err)
    }

    pub fn debug_set_log_level(
        &self,
        logger_name: Option<String>,
        level: Level,
    ) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| {
            inner
                .debug_client
                .set_log_level(logger_name, level.to_string())
        })
        .map_err(map_err)
    }

    pub fn debug_set_log_pattern(&self, pattern: LogPattern) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| inner.debug_client.set_log_pattern(pattern))
            .map_err(map_err)
    }

    pub fn debug_panic(&self) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| inner.debug_client.panic())
            .map_err(map_err)
    }

    pub fn debug_txfactory_status(&self, action: FactoryAction) -> anyhow::Result<bool> {
        self.call_rpc_blocking(|inner| inner.debug_client.txfactory_status(action))
            .map_err(map_err)
    }

    pub fn sleep(&self, time: u64) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| inner.debug_client.sleep(time))
            .map_err(map_err)
    }

    pub fn chain_id(&self) -> anyhow::Result<ChainId> {
        self.call_rpc_blocking(|inner| inner.chain_client.id())
            .map_err(map_err)
    }

    pub fn chain_info(&self) -> anyhow::Result<ChainInfoView> {
        self.call_rpc_blocking(|inner| inner.chain_client.info())
            .map_err(map_err)
    }

    pub fn get_headers(
        &self,
        block_hashes: Vec<HashValue>,
    ) -> anyhow::Result<Vec<BlockHeaderView>> {
        self.call_rpc_blocking(|inner| inner.chain_client.get_headers(block_hashes))
            .map_err(map_err)
    }

    pub fn chain_get_block_by_hash(
        &self,
        hash: HashValue,
        option: Option<GetBlockOption>,
    ) -> anyhow::Result<Option<BlockView>> {
        self.call_rpc_blocking(|inner| inner.chain_client.get_block_by_hash(hash, option))
            .map_err(map_err)
    }

    pub fn chain_get_block_by_number(
        &self,
        number: BlockNumber,
        option: Option<GetBlockOption>,
    ) -> anyhow::Result<Option<BlockView>> {
        self.call_rpc_blocking(|inner| inner.chain_client.get_block_by_number(number, option))
            .map_err(map_err)
    }

    pub fn chain_get_block_info_by_number(
        &self,
        number: BlockNumber,
    ) -> anyhow::Result<Option<BlockInfoView>> {
        self.call_rpc_blocking(|inner| inner.chain_client.get_block_info_by_number(number))
            .map_err(map_err)
    }

    pub fn chain_get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> anyhow::Result<Vec<BlockView>> {
        self.call_rpc_blocking(|inner| inner.chain_client.get_blocks_by_number(number, count))
            .map_err(map_err)
    }

    pub fn chain_get_transaction(
        &self,
        txn_id: HashValue,
        option: Option<GetTransactionOption>,
    ) -> anyhow::Result<Option<TransactionView>> {
        self.call_rpc_blocking(|inner| inner.chain_client.get_transaction(txn_id, option))
            .map_err(map_err)
    }

    pub fn chain_get_transaction_info(
        &self,
        txn_hash: HashValue,
    ) -> anyhow::Result<Option<TransactionInfoView>> {
        self.call_rpc_blocking(|inner| inner.chain_client.get_transaction_info(txn_hash))
            .map_err(map_err)
    }

    pub fn chain_get_events_by_txn_hash(
        &self,
        txn_hash: HashValue,
        option: Option<GetEventOption>,
    ) -> anyhow::Result<Vec<TransactionEventResponse>> {
        self.call_rpc_blocking(|inner| inner.chain_client.get_events_by_txn_hash(txn_hash, option))
            .map_err(map_err)
    }

    pub fn chain_get_block_txn_infos(
        &self,
        block_id: HashValue,
    ) -> anyhow::Result<Vec<TransactionInfoView>> {
        self.call_rpc_blocking(|inner| inner.chain_client.get_block_txn_infos(block_id))
            .map_err(map_err)
    }

    pub fn chain_get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> anyhow::Result<Option<TransactionInfoView>> {
        self.call_rpc_blocking(|inner| {
            inner
                .chain_client
                .get_txn_info_by_block_and_index(block_id, idx)
        })
        .map_err(map_err)
    }

    pub fn chain_get_transaction_infos(
        &self,
        start_global_index: u64,
        reverse: bool,
        max_size: u64,
    ) -> anyhow::Result<Vec<TransactionInfoView>> {
        self.call_rpc_blocking(|inner| {
            inner
                .chain_client
                .get_transaction_infos(start_global_index, reverse, max_size)
        })
        .map_err(map_err)
    }

    pub fn chain_get_transaction_proof(
        &self,
        block_hash: HashValue,
        transaction_global_index: u64,
        event_index: Option<u64>,
        access_path: Option<AccessPath>,
    ) -> anyhow::Result<Option<TransactionInfoWithProofView>> {
        self.call_rpc_blocking(|inner| {
            inner.chain_client.get_transaction_proof(
                block_hash,
                transaction_global_index,
                event_index,
                access_path.map(Into::into),
            )
        })
        .map_err(map_err)
    }

    pub fn dry_run(&self, txn: DryRunTransactionRequest) -> anyhow::Result<DryRunOutputView> {
        self.call_rpc_blocking(|inner| inner.contract_client.dry_run(txn))
            .map_err(map_err)
    }
    pub fn dry_run_raw(&self, txn: DryRunTransaction) -> anyhow::Result<DryRunOutputView> {
        let DryRunTransaction {
            raw_txn,
            public_key,
        } = txn;
        let raw_txn_str = hex::encode(raw_txn.encode()?);
        self.call_rpc_blocking(|inner| {
            inner
                .contract_client
                .dry_run_raw(raw_txn_str, StrView(public_key))
        })
        .map_err(map_err)
    }
    pub fn miner_submit(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> anyhow::Result<MintedBlockView> {
        self.call_rpc_blocking(|inner| inner.miner_client.submit(minting_blob, nonce, extra))
            .map_err(map_err)
    }
    pub async fn miner_submit_async(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> anyhow::Result<MintedBlockView> {
        self.call_rpc_async(|inner| inner.miner_client.submit(minting_blob, nonce, extra))
            .await
            .map_err(map_err)
    }

    pub fn txpool_status(&self) -> anyhow::Result<TxPoolStatus> {
        self.call_rpc_blocking(|inner| inner.txpool_client.state())
            .map_err(map_err)
    }

    pub fn subscribe_events(
        &self,
        filter: EventFilter,
        decode: bool,
    ) -> anyhow::Result<impl TryStream<Ok = TransactionEventView, Error = anyhow::Error>> {
        self.call_rpc_blocking(|inner| async move {
            let res = inner.pubsub_client.subscribe_events(filter, decode).await;
            res.map(|s| s.map_err(map_err))
        })
        .map_err(map_err)
    }
    pub fn subscribe_new_blocks(
        &self,
    ) -> anyhow::Result<impl TryStream<Ok = BlockView, Error = anyhow::Error>> {
        self.call_rpc_blocking(|inner| async move {
            let res = inner.pubsub_client.subscribe_new_block().await;
            res.map(|s| s.map_err(map_err))
        })
        .map_err(map_err)
    }
    pub fn subscribe_new_transactions(
        &self,
    ) -> anyhow::Result<impl TryStream<Ok = Vec<HashValue>, Error = anyhow::Error>> {
        self.call_rpc_blocking(|inner| async move {
            let res = inner.pubsub_client.subscribe_new_transactions().await;
            res.map(|s| s.map_err(map_err))
        })
        .map_err(map_err)
    }

    pub fn subscribe_new_mint_blocks(
        &self,
    ) -> anyhow::Result<impl TryStream<Ok = MintBlockEvent, Error = anyhow::Error>> {
        self.call_rpc_blocking(|inner| async move {
            let res = inner.pubsub_client.subscribe_new_mint_block().await;
            res.map(|s| s.map_err(map_err))
        })
        .map_err(map_err)
    }

    pub async fn subscribe_new_mint_blocks_async(
        &self,
    ) -> anyhow::Result<impl TryStream<Ok = MintBlockEvent, Error = anyhow::Error>> {
        self.call_rpc_async(|inner| async move {
            let res = inner.pubsub_client.subscribe_new_mint_block().await;
            res.map(|s| s.map_err(map_err))
        })
        .await
        .map_err(map_err)
    }

    fn call_rpc_blocking<F, T>(
        &self,
        f: impl FnOnce(RpcClientInner) -> F + Send,
    ) -> Result<T, jsonrpc_client_transports::RpcError>
    where
        T: Send,
        F: std::future::Future<Output = Result<T, jsonrpc_client_transports::RpcError>> + Send,
    {
        self.provider
            .block_on(async { self.call_rpc_async(f).await })
    }

    async fn call_rpc_async<F, T>(
        &self,
        f: impl FnOnce(RpcClientInner) -> F + Send,
    ) -> Result<T, jsonrpc_client_transports::RpcError>
    where
        F: std::future::Future<Output = Result<T, jsonrpc_client_transports::RpcError>> + Send,
    {
        let inner_opt = self.inner.lock().as_ref().cloned();
        let inner = match inner_opt {
            Some(inner) => inner,
            None => {
                info!(
                    "Connection is lost, try reconnect by {:?}",
                    &self.provider.conn_source
                );
                let new_inner: RpcClientInner = self
                    .provider
                    .get_rpc_channel_async()
                    .await
                    .map(|c| c.into())?;
                *(self.inner.lock()) = Some(new_inner.clone());
                self.chain_watcher.do_send(StartSubscribe {
                    client: new_inner.pubsub_client.clone(),
                });
                new_inner
            }
        };
        let result = f(inner).await;
        if let Err(jsonrpc_client_transports::RpcError::Other(e)) = &result {
            error!("rpc error due to {}", e);
            *(self.inner.lock()) = None;
        }
        result
    }

    pub fn sync_status(&self) -> anyhow::Result<SyncStatus> {
        self.call_rpc_blocking(|inner| inner.sync_client.status())
            .map_err(map_err)
    }

    pub fn sync_progress(&self) -> anyhow::Result<Option<SyncProgressReport>> {
        self.call_rpc_blocking(|inner| inner.sync_client.progress())
            .map_err(map_err)
    }

    pub fn sync_peer_score(&self) -> anyhow::Result<PeerScoreResponse> {
        self.call_rpc_blocking(|inner| inner.sync_client.peer_score())
            .map_err(map_err)
    }

    pub fn sync_start(
        &self,
        force: bool,
        peers: Vec<PeerId>,
        skip_pow_verify: bool,
        strategy: Option<PeerStrategy>,
    ) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| {
            inner
                .sync_client
                .start(force, peers, skip_pow_verify, strategy)
        })
        .map_err(map_err)
    }

    pub fn sync_cancel(&self) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| inner.sync_client.cancel())
            .map_err(map_err)
    }

    pub fn network_known_peers(&self) -> anyhow::Result<Vec<PeerId>> {
        self.call_rpc_blocking(|inner| inner.network_client.known_peers())
            .map_err(map_err)
    }

    pub fn network_state(&self) -> anyhow::Result<NetworkState> {
        self.call_rpc_blocking(|inner| inner.network_client.state())
            .map_err(map_err)
    }

    pub fn network_get_address(&self, peer_id: String) -> anyhow::Result<Vec<Multiaddr>> {
        self.call_rpc_blocking(|inner| inner.network_client.get_address(peer_id))
            .map_err(map_err)
    }

    pub fn network_add_peer(&self, peer: String) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| inner.network_client.add_peer(peer))
            .map_err(map_err)
    }

    pub fn network_call_peer(
        &self,
        peer_id: String,
        rpc_method: String,
        message: StrView<Vec<u8>>,
    ) -> anyhow::Result<StrView<Vec<u8>>> {
        self.call_rpc_blocking(|inner| {
            inner
                .network_client
                .call_peer(peer_id, rpc_method.into(), message)
        })
        .map_err(map_err)
    }

    pub fn call_raw_api(&self, api: &str, params: Params) -> anyhow::Result<Value> {
        self.call_rpc_blocking(|inner| inner.raw_client.call_method(api, params))
            .map_err(map_err)
    }
    pub fn set_peer_reputation(&self, peer_id: String, reput: i32) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| inner.network_client.set_peer_reputation(peer_id, reput))
            .map_err(map_err)
    }
    pub fn ban_peer(&self, peer_id: String, ban: bool) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| inner.network_client.ban_peer(peer_id, ban))
            .map_err(map_err)
    }
    pub fn close(self) {
        if let Err(e) = self.chain_watcher.try_send(chain_watcher::StopWatcher) {
            error!("Try to stop chain watcher error: {:?}", e);
        }
        if let Err(e) = self.watcher_handle.join() {
            error!("Wait chain watcher thread stop error: {:?}", e);
        }
    }
}

#[derive(Clone)]
pub(crate) struct RpcClientInner {
    raw_client: RawClient,
    node_client: NodeClient,
    node_manager_client: NodeManagerClient,
    txpool_client: TxPoolClient,
    account_client: AccountClient,
    state_client: StateClient,
    debug_client: DebugClient,
    chain_client: ChainClient,
    pubsub_client: PubSubClient,
    contract_client: ContractClient,
    miner_client: MinerClient,
    sync_client: SyncManagerClient,
    network_client: NetworkManagerClient,
}

impl RpcClientInner {
    pub fn new(channel: RpcChannel) -> Self {
        Self {
            raw_client: channel.clone().into(),
            node_client: channel.clone().into(),
            node_manager_client: channel.clone().into(),
            txpool_client: channel.clone().into(),
            account_client: channel.clone().into(),
            state_client: channel.clone().into(),
            debug_client: channel.clone().into(),
            chain_client: channel.clone().into(),
            contract_client: channel.clone().into(),
            pubsub_client: channel.clone().into(),
            miner_client: channel.clone().into(),
            sync_client: channel.clone().into(),
            network_client: channel.into(),
        }
    }
}

fn map_err(rpc_err: jsonrpc_client_transports::RpcError) -> anyhow::Error {
    anyhow!(format!("{}", rpc_err))
}

impl From<RpcChannel> for RpcClientInner {
    fn from(channel: RpcChannel) -> Self {
        Self::new(channel)
    }
}
