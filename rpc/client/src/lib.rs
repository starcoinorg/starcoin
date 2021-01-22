// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::chain_watcher::{ChainWatcher, StartSubscribe, WatchBlock, WatchTxn};
use crate::pubsub_client::PubSubClient;
use actix::{Addr, System};
use failure::Fail;
use futures03::channel::oneshot;
use futures03::{TryStream, TryStreamExt};
use jsonrpc_client_transports::RawClient;
use jsonrpc_core_client::{transports::ipc, transports::ws, RpcChannel};
use network_p2p_types::network_state::NetworkState;
use parking_lot::Mutex;
use serde_json::Value;
use starcoin_account_api::AccountInfo;
use starcoin_crypto::HashValue;
use starcoin_logger::{prelude::*, LogPattern};
use starcoin_rpc_api::node::NodeInfo;
use starcoin_rpc_api::service::RpcAsyncService;
use starcoin_rpc_api::types::pubsub::EventFilter;
use starcoin_rpc_api::types::pubsub::MintBlock;
use starcoin_rpc_api::types::{
    AccountStateSetView, AnnotatedMoveStructView, AnnotatedMoveValueView, BlockHeaderView,
    BlockSummaryView, BlockView, ChainId, ChainInfoView, ContractCall, DryRunTransactionRequest,
    EpochUncleSummaryView, FactoryAction, PeerInfoView, SignedUserTransactionView, StrView,
    TransactionInfoView, TransactionOutputView, TransactionRequest, TransactionView,
};
use starcoin_rpc_api::{
    account::AccountClient, chain::ChainClient, contract_api::ContractClient, debug::DebugClient,
    dev::DevClient, miner::MinerClient, network_manager::NetworkManagerClient, node::NodeClient,
    node_manager::NodeManagerClient, state::StateClient, sync_manager::SyncManagerClient,
    txpool::TxPoolClient, types::TransactionEventView,
};
use starcoin_service_registry::{ServiceInfo, ServiceStatus};
use starcoin_state_api::StateWithProof;
use starcoin_sync_api::SyncProgressReport;
use starcoin_txpool_api::TxPoolStatus;
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::block::BlockNumber;
use starcoin_types::peer_info::{Multiaddr, PeerId};
use starcoin_types::stress_test::TPS;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::system_events::SystemStop;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_vm_types::language_storage::{ModuleId, StructTag};
use starcoin_vm_types::on_chain_resource::{EpochInfo, GlobalTimeOnChain};
use starcoin_vm_types::token::token_code::TokenCode;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use tokio01::reactor::Reactor;
use tokio_compat::prelude::*;
use tokio_compat::runtime::Runtime;

pub mod chain_watcher;
mod pubsub_client;
mod remote_state_reader;
pub use crate::remote_state_reader::RemoteStateReader;
pub use jsonrpc_core::Params;

#[derive(Clone)]
enum ConnSource {
    Ipc(PathBuf, Arc<Reactor>),
    WebSocket(String),
    Local(Box<RpcChannel>),
}

impl std::fmt::Debug for ConnSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnSource::Ipc(path, _) => write!(f, "Ipc({})", path.as_path().to_string_lossy()),
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
    //TODO remove runtime after jsonrpc upgrade.
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
        F: futures03::Future + std::marker::Send,
        F::Output: std::marker::Send,
    {
        let result = self.runtime.lock().block_on_std(future);
        result
    }

    fn get_rpc_channel(&self) -> anyhow::Result<RpcChannel, jsonrpc_client_transports::RpcError> {
        self.block_on(async { self.get_rpc_channel_async().await })
    }

    async fn get_rpc_channel_async(
        &self,
    ) -> anyhow::Result<RpcChannel, jsonrpc_client_transports::RpcError> {
        match self.conn_source.clone() {
            ConnSource::Ipc(sock_path, reactor) => {
                let conn_fut = ipc::connect(sock_path, &reactor.handle()).map_err(|e| {
                    jsonrpc_client_transports::RpcError::Other(failure::Error::from(e))
                })?;
                conn_fut.compat().await
            }
            ConnSource::WebSocket(url) => {
                ws::try_connect(url.as_str())
                    .map_err(jsonrpc_client_transports::RpcError::Other)?
                    .compat()
                    .await
            }
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
        let watcher = futures03::executor::block_on(rx).expect("Init chain watcher fail.");
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
        let client = futures03::executor::block_on(async { rpc_service.connect_local().await })?;
        Self::new(ConnSource::Local(Box::new(client)))
    }

    pub fn connect_ipc<P: AsRef<Path>>(sock_path: P) -> anyhow::Result<Self> {
        let reactor = Reactor::new().unwrap();
        let path = sock_path.as_ref().to_path_buf();
        Self::new(ConnSource::Ipc(path, Arc::new(reactor)))
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
        futures03::executor::block_on(f)
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
        futures03::executor::block_on(f)
    }

    pub fn node_status(&self) -> anyhow::Result<bool> {
        self.call_rpc_blocking(|inner| async move { inner.node_client.status().compat().await })
            .map_err(map_err)
    }

    pub fn node_info(&self) -> anyhow::Result<NodeInfo> {
        self.call_rpc_blocking(|inner| async move { inner.node_client.info().compat().await })
            .map_err(map_err)
    }

    pub async fn node_info_async(&self) -> anyhow::Result<NodeInfo> {
        self.call_rpc_async(|inner| async move { inner.node_client.info().compat().await })
            .await
            .map_err(map_err)
    }

    pub fn node_metrics(&self) -> anyhow::Result<HashMap<String, String>> {
        self.call_rpc_blocking(|inner| async move { inner.node_client.metrics().compat().await })
            .map_err(map_err)
    }

    pub fn node_peers(&self) -> anyhow::Result<Vec<PeerInfoView>> {
        self.call_rpc_blocking(|inner| async move { inner.node_client.peers().compat().await })
            .map_err(map_err)
    }

    pub fn node_list_service(&self) -> anyhow::Result<Vec<ServiceInfo>> {
        self.call_rpc_blocking(|inner| async move {
            inner.node_manager_client.list_service().compat().await
        })
        .map_err(map_err)
    }

    pub fn node_start_service(&self, service_name: String) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .node_manager_client
                .start_service(service_name)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn node_check_service(&self, service_name: String) -> anyhow::Result<ServiceStatus> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .node_manager_client
                .check_service(service_name)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn node_stop_service(&self, service_name: String) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .node_manager_client
                .stop_service(service_name)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn node_shutdown_system(&self) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move {
            inner.node_manager_client.shutdown_system().compat().await
        })
        .map_err(map_err)
    }

    pub fn next_sequence_number_in_txpool(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<Option<u64>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .txpool_client
                .next_sequence_number(address)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn submit_transaction(&self, txn: SignedUserTransaction) -> anyhow::Result<HashValue> {
        self.call_rpc_blocking(|inner| async move {
            inner.txpool_client.submit_transaction(txn).compat().await
        })
        .map_err(map_err)
    }

    pub fn get_pending_txn_by_hash(
        &self,
        txn_hash: HashValue,
    ) -> anyhow::Result<Option<SignedUserTransactionView>> {
        self.call_rpc_blocking(|inner| async move {
            inner.txpool_client.pending_txn(txn_hash).compat().await
        })
        .map_err(map_err)
    }

    pub fn get_pending_txns_of_sender(
        &self,
        sender: AccountAddress,
        max_len: Option<u32>,
    ) -> anyhow::Result<Vec<SignedUserTransactionView>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .txpool_client
                .pending_txns(sender, max_len)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    //TODO should split client for different api ?
    // such as  RpcClient().account().default()
    pub fn account_default(&self) -> anyhow::Result<Option<AccountInfo>> {
        self.call_rpc_blocking(|inner| async move { inner.account_client.default().compat().await })
            .map_err(map_err)
    }

    pub fn set_default_account(&self, addr: AccountAddress) -> anyhow::Result<Option<AccountInfo>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .account_client
                .set_default_account(addr)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn account_create(&self, password: String) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| async move {
            inner.account_client.create(password).compat().await
        })
        .map_err(map_err)
    }

    pub fn account_list(&self) -> anyhow::Result<Vec<AccountInfo>> {
        self.call_rpc_blocking(|inner| async move { inner.account_client.list().compat().await })
            .map_err(map_err)
    }

    pub fn account_get(&self, address: AccountAddress) -> anyhow::Result<Option<AccountInfo>> {
        self.call_rpc_blocking(
            |inner| async move { inner.account_client.get(address).compat().await },
        )
        .map_err(map_err)
    }

    /// partial sign a multisig account's txn
    pub fn account_sign_multisig_txn(
        &self,
        raw_txn: RawUserTransaction,
        signer_address: AccountAddress,
    ) -> anyhow::Result<SignedUserTransaction> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .account_client
                .sign_txn(raw_txn, signer_address)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn account_sign_txn_request(
        &self,
        txn_request: TransactionRequest,
    ) -> anyhow::Result<SignedUserTransaction> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .account_client
                .sign_txn_request(txn_request)
                .compat()
                .await
        })
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
        self.call_rpc_blocking(|inner| async move {
            inner
                .account_client
                .sign_txn(raw_txn, signer)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn account_change_password(
        &self,
        address: AccountAddress,
        new_password: String,
    ) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .account_client
                .change_account_password(address, new_password)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn account_lock(&self, address: AccountAddress) -> anyhow::Result<()> {
        self.call_rpc_blocking(
            |inner| async move { inner.account_client.lock(address).compat().await },
        )
        .map_err(map_err)
    }
    pub fn account_unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: std::time::Duration,
    ) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .account_client
                .unlock(address, password, Some(duration.as_secs() as u32))
                .compat()
                .await
        })
        .map_err(map_err)
    }
    pub fn account_export(
        &self,
        address: AccountAddress,
        password: String,
    ) -> anyhow::Result<Vec<u8>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .account_client
                .export(address, password)
                .compat()
                .await
        })
        .map_err(map_err)
    }
    pub fn account_import(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> anyhow::Result<AccountInfo> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .account_client
                .import(address, private_key, password)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn account_accepted_tokens(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<Vec<TokenCode>> {
        self.call_rpc_blocking(|inner| async move {
            inner.account_client.accepted_tokens(address).compat().await
        })
        .map_err(map_err)
    }

    pub fn get_code(&self, module_id: ModuleId) -> anyhow::Result<Option<String>> {
        let result: Option<StrView<Vec<u8>>> = self
            .call_rpc_blocking(|inner| async move {
                inner
                    .contract_client
                    .get_code(StrView(module_id))
                    .compat()
                    .await
            })
            .map_err(map_err)?;
        Ok(result.map(|s| s.to_string()))
    }

    pub fn get_resource(
        &self,
        addr: AccountAddress,
        resource_type: StructTag,
    ) -> anyhow::Result<Option<AnnotatedMoveStructView>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .contract_client
                .get_resource(addr, StrView(resource_type))
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn state_get(&self, access_path: AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        self.call_rpc_blocking(
            |inner| async move { inner.state_client.get(access_path).compat().await },
        )
        .map_err(map_err)
    }

    pub fn state_get_with_proof(&self, access_path: AccessPath) -> anyhow::Result<StateWithProof> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .state_client
                .get_with_proof(access_path)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn state_get_with_proof_by_root(
        &self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> anyhow::Result<StateWithProof> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .state_client
                .get_with_proof_by_root(access_path, state_root)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn state_get_state_root(&self) -> anyhow::Result<HashValue> {
        self.call_rpc_blocking(
            |inner| async move { inner.state_client.get_state_root().compat().await },
        )
        .map_err(map_err)
    }

    pub fn state_get_account_state(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<Option<AccountState>> {
        self.call_rpc_blocking(|inner| async move {
            inner.state_client.get_account_state(address).compat().await
        })
        .map_err(map_err)
    }

    pub fn get_account_state_set(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<Option<AccountStateSetView>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .state_client
                .get_account_state_set(address)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn contract_call(&self, call: ContractCall) -> anyhow::Result<Vec<AnnotatedMoveValueView>> {
        self.call_rpc_blocking(
            |inner| async move { inner.contract_client.call(call).compat().await },
        )
        .map_err(map_err)
    }

    pub fn debug_set_log_level(
        &self,
        logger_name: Option<String>,
        level: Level,
    ) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .debug_client
                .set_log_level(logger_name, level.to_string())
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn debug_set_log_pattern(&self, pattern: LogPattern) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move {
            inner.debug_client.set_log_pattern(pattern).compat().await
        })
        .map_err(map_err)
    }

    pub fn debug_panic(&self) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move { inner.debug_client.panic().compat().await })
            .map_err(map_err)
    }

    pub fn debug_txfactory_status(&self, action: FactoryAction) -> anyhow::Result<bool> {
        self.call_rpc_blocking(|inner| async move {
            inner.debug_client.txfactory_status(action).compat().await
        })
        .map_err(map_err)
    }

    pub fn sleep(&self, time: u64) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move { inner.debug_client.sleep(time).compat().await })
            .map_err(map_err)
    }

    pub fn chain_id(&self) -> anyhow::Result<ChainId> {
        self.call_rpc_blocking(|inner| async move { inner.chain_client.id().compat().await })
            .map_err(map_err)
    }

    pub fn chain_info(&self) -> anyhow::Result<ChainInfoView> {
        self.call_rpc_blocking(|inner| async move { inner.chain_client.info().compat().await })
            .map_err(map_err)
    }

    pub fn epoch_info(&self) -> anyhow::Result<EpochInfo> {
        self.call_rpc_blocking(
            |inner| async move { inner.chain_client.current_epoch().compat().await },
        )
        .map_err(map_err)
    }

    pub fn get_epoch_info_by_number(&self, number: BlockNumber) -> anyhow::Result<EpochInfo> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_epoch_info_by_number(number)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn tps(&self, number: Option<BlockNumber>) -> anyhow::Result<TPS> {
        self.call_rpc_blocking(|inner| async move { inner.chain_client.tps(number).compat().await })
            .map_err(map_err)
    }

    pub fn get_epoch_uncles_by_number(
        &self,
        number: BlockNumber,
    ) -> anyhow::Result<Vec<BlockSummaryView>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_epoch_uncles_by_number(number)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn epoch_uncle_summary_by_number(
        &self,
        number: BlockNumber,
    ) -> anyhow::Result<EpochUncleSummaryView> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .epoch_uncle_summary_by_number(number)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn get_headers(
        &self,
        block_hashes: Vec<HashValue>,
    ) -> anyhow::Result<Vec<BlockHeaderView>> {
        self.call_rpc_blocking(|inner| async move {
            inner.chain_client.get_headers(block_hashes).compat().await
        })
        .map_err(map_err)
    }

    pub fn uncle_path(
        &self,
        block_id: HashValue,
        uncle_id: HashValue,
    ) -> anyhow::Result<Vec<BlockHeaderView>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .uncle_path(block_id, uncle_id)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn get_global_time_by_number(
        &self,
        number: BlockNumber,
    ) -> anyhow::Result<GlobalTimeOnChain> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_global_time_by_number(number)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn chain_get_block_by_hash(&self, hash: HashValue) -> anyhow::Result<Option<BlockView>> {
        self.call_rpc_blocking(|inner| async move {
            inner.chain_client.get_block_by_hash(hash).compat().await
        })
        .map_err(map_err)
    }

    pub fn chain_get_block_by_uncle(
        &self,
        uncle_id: HashValue,
    ) -> anyhow::Result<Option<BlockView>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_block_by_uncle(uncle_id)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn chain_get_block_by_number(
        &self,
        number: BlockNumber,
    ) -> anyhow::Result<Option<BlockView>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_block_by_number(number)
                .compat()
                .await
        })
        .map_err(map_err)
    }
    pub fn chain_get_blocks_by_number(
        &self,
        number: Option<BlockNumber>,
        count: u64,
    ) -> anyhow::Result<Vec<BlockView>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_blocks_by_number(number, count)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn chain_get_transaction(
        &self,
        txn_id: HashValue,
    ) -> anyhow::Result<Option<TransactionView>> {
        self.call_rpc_blocking(|inner| async move {
            inner.chain_client.get_transaction(txn_id).compat().await
        })
        .map_err(map_err)
    }

    pub fn chain_get_transaction_info(
        &self,
        txn_hash: HashValue,
    ) -> anyhow::Result<Option<TransactionInfoView>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_transaction_info(txn_hash)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn chain_get_events_by_txn_hash(
        &self,
        txn_hash: HashValue,
    ) -> anyhow::Result<Vec<TransactionEventView>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_events_by_txn_hash(txn_hash)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn chain_get_block_txn_infos(
        &self,
        block_id: HashValue,
    ) -> anyhow::Result<Vec<TransactionInfoView>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_block_txn_infos(block_id)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn chain_get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> anyhow::Result<Option<TransactionInfoView>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_txn_info_by_block_and_index(block_id, idx)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn dry_run(&self, txn: DryRunTransactionRequest) -> anyhow::Result<TransactionOutputView> {
        self.call_rpc_blocking(
            |inner| async move { inner.contract_client.dry_run(txn).compat().await },
        )
        .map_err(map_err)
    }
    pub fn miner_submit(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .miner_client
                .submit(minting_blob, nonce, extra)
                .compat()
                .await
        })
        .map_err(map_err)
    }
    pub async fn miner_submit_async(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> anyhow::Result<()> {
        self.call_rpc_async(|inner| async move {
            inner
                .miner_client
                .submit(minting_blob, nonce, extra)
                .compat()
                .await
        })
        .await
        .map_err(map_err)
    }

    pub fn txpool_status(&self) -> anyhow::Result<TxPoolStatus> {
        self.call_rpc_blocking(|inner| async move { inner.txpool_client.state().compat().await })
            .map_err(map_err)
    }

    pub fn subscribe_events(
        &self,
        filter: EventFilter,
    ) -> anyhow::Result<impl TryStream<Ok = TransactionEventView, Error = anyhow::Error>> {
        self.call_rpc_blocking(|inner| async move {
            let res = inner.pubsub_client.subscribe_events(filter).await;
            res.map(|s| s.compat().map_err(map_err))
        })
        .map_err(map_err)
    }
    pub fn subscribe_new_blocks(
        &self,
    ) -> anyhow::Result<impl TryStream<Ok = BlockView, Error = anyhow::Error>> {
        self.call_rpc_blocking(|inner| async move {
            let res = inner.pubsub_client.subscribe_new_block().await;
            res.map(|s| s.compat().map_err(map_err))
        })
        .map_err(map_err)
    }
    pub fn subscribe_new_transactions(
        &self,
    ) -> anyhow::Result<impl TryStream<Ok = Vec<HashValue>, Error = anyhow::Error>> {
        self.call_rpc_blocking(|inner| async move {
            let res = inner.pubsub_client.subscribe_new_transactions().await;
            res.map(|s| s.compat().map_err(map_err))
        })
        .map_err(map_err)
    }

    pub fn subscribe_new_mint_blocks(
        &self,
    ) -> anyhow::Result<impl TryStream<Ok = MintBlock, Error = anyhow::Error>> {
        self.call_rpc_blocking(|inner| async move {
            let res = inner.pubsub_client.subscribe_new_mint_block().await;
            res.map(|s| s.compat().map_err(map_err))
        })
        .map_err(map_err)
    }

    pub async fn subscribe_new_mint_blocks_async(
        &self,
    ) -> anyhow::Result<impl TryStream<Ok = MintBlock, Error = anyhow::Error>> {
        self.call_rpc_async(|inner| async move {
            let res = inner.pubsub_client.subscribe_new_mint_block().await;
            res.map(|s| s.compat().map_err(map_err))
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
        if let Err(rpc_error) = &result {
            if let jsonrpc_client_transports::RpcError::Other(e) = rpc_error {
                error!("rpc error due to {}", e);
                *(self.inner.lock()) = None;
            }
        }
        result
    }

    pub fn sync_status(&self) -> anyhow::Result<SyncStatus> {
        self.call_rpc_blocking(|inner| async move { inner.sync_client.status().compat().await })
            .map_err(map_err)
    }

    pub fn sync_progress(&self) -> anyhow::Result<Option<SyncProgressReport>> {
        self.call_rpc_blocking(|inner| async move { inner.sync_client.progress().compat().await })
            .map_err(map_err)
    }

    pub fn sync_start(
        &self,
        force: bool,
        peers: Vec<PeerId>,
        skip_pow_verify: bool,
    ) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .sync_client
                .start(force, peers, skip_pow_verify)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn sync_cancel(&self) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move { inner.sync_client.cancel().compat().await })
            .map_err(map_err)
    }

    pub fn network_known_peers(&self) -> anyhow::Result<Vec<PeerId>> {
        self.call_rpc_blocking(
            |inner| async move { inner.network_client.known_peers().compat().await },
        )
        .map_err(map_err)
    }

    pub fn network_state(&self) -> anyhow::Result<NetworkState> {
        self.call_rpc_blocking(|inner| async move { inner.network_client.state().compat().await })
            .map_err(map_err)
    }

    pub fn network_get_address(&self, peer_id: String) -> anyhow::Result<Vec<Multiaddr>> {
        self.call_rpc_blocking(|inner| async move {
            inner.network_client.get_address(peer_id).compat().await
        })
        .map_err(map_err)
    }

    pub fn network_add_peer(&self, peer: String) -> anyhow::Result<()> {
        self.call_rpc_blocking(
            |inner| async move { inner.network_client.add_peer(peer).compat().await },
        )
        .map_err(map_err)
    }

    pub fn call_raw_api(&self, api: &str, params: Params) -> anyhow::Result<Value> {
        self.call_rpc_blocking(|inner| async move {
            inner.raw_client.call_method(api, params).compat().await
        })
        .map_err(map_err)
    }

    pub fn close(self) {
        if let Err(e) = self.chain_watcher.try_send(SystemStop) {
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
    dev_client: DevClient,
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
            dev_client: channel.clone().into(),
            contract_client: channel.clone().into(),
            pubsub_client: channel.clone().into(),
            miner_client: channel.clone().into(),
            sync_client: channel.clone().into(),
            network_client: channel.into(),
        }
    }
}

fn map_err(rpc_err: jsonrpc_client_transports::RpcError) -> anyhow::Error {
    rpc_err.compat().into()
}

impl From<RpcChannel> for RpcClientInner {
    fn from(channel: RpcChannel) -> Self {
        Self::new(channel)
    }
}
