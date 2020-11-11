// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::chain_watcher::{ChainWatcher, WatchBlock, WatchTxn};
use crate::pubsub_client::PubSubClient;
use actix::{Addr, System};
use failure::Fail;
use futures::future::Future as Future01;
use futures03::channel::oneshot;
use futures03::{TryStream, TryStreamExt};
use jsonrpc_core_client::{transports::ipc, transports::ws, RpcChannel};
use parking_lot::Mutex;
use starcoin_account_api::AccountInfo;
use starcoin_crypto::HashValue;
use starcoin_logger::{prelude::*, LogPattern};
use starcoin_rpc_api::node::NodeInfo;
use starcoin_rpc_api::types::pubsub::EventFilter;
use starcoin_rpc_api::types::pubsub::ThinHeadBlock;
use starcoin_rpc_api::types::pubsub::{Event, MintBlock};
use starcoin_rpc_api::{
    account::AccountClient, chain::ChainClient, debug::DebugClient, dev::DevClient,
    miner::MinerClient, node::NodeClient, node_manager::NodeManagerClient, state::StateClient,
    sync_manager::SyncManagerClient, txpool::TxPoolClient,
};
use starcoin_state_api::StateWithProof;
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::block::{Block, BlockNumber};
use starcoin_types::peer_info::PeerInfo;
use starcoin_types::startup_info::ChainInfo;
use starcoin_types::transaction::{
    RawUserTransaction, SignedUserTransaction, Transaction, TransactionInfo, TransactionOutput,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio01::reactor::Reactor;
use tokio_compat::prelude::*;
use tokio_compat::runtime::Runtime;

pub mod chain_watcher;
mod pubsub_client;
mod remote_state_reader;

pub use crate::remote_state_reader::RemoteStateReader;
use starcoin_rpc_api::service::RpcAsyncService;
use starcoin_rpc_api::types::{AnnotatedMoveValue, ContractCall};
use starcoin_service_registry::{ServiceInfo, ServiceStatus};
use starcoin_sync_api::TaskProgressReport;
use starcoin_txpool_api::TxPoolStatus;
use starcoin_types::sync_status::SyncStatus;
use starcoin_types::{contract_event::ContractEvent, system_events::SystemStop};
use starcoin_vm_types::on_chain_resource::{EpochInfo, GlobalTimeOnChain};
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::vm_status::VMStatus;
use std::thread::JoinHandle;

#[derive(Debug, Clone)]
enum ConnSource {
    Ipc(PathBuf, Arc<Reactor>),
    WebSocket(String),
    Local,
}

pub struct RpcClient {
    inner: Mutex<Option<RpcClientInner>>,
    conn_source: ConnSource,
    chain_watcher: Addr<ChainWatcher>,
    //hold the watch thread handle.
    watcher_handle: JoinHandle<()>,
}

struct ConnectionProvider {
    conn_source: ConnSource,
}

#[derive(Error, Debug)]
pub enum ConnError {
    #[error("io error, {0}")]
    Io(#[from] std::io::Error),
    #[error("rpc error, {0}")]
    RpcError(jsonrpc_client_transports::RpcError),
}

impl ConnectionProvider {
    async fn get_rpc_channel(&self) -> anyhow::Result<RpcChannel, ConnError> {
        match &self.conn_source {
            ConnSource::Ipc(sock_path, reactor) => {
                let conn_fut = ipc::connect(sock_path, &reactor.handle())?;
                conn_fut.compat().await.map_err(ConnError::RpcError)
            }
            // only have ipc impl for now
            _ => unreachable!(),
        }
    }
}

impl RpcClient {
    pub(crate) fn new(conn_source: ConnSource, inner: RpcClientInner) -> Self {
        let (tx, rx) = oneshot::channel();
        let pubsub_client = inner.pubsub_client.clone();
        let handle = std::thread::spawn(move || {
            let sys = System::new("client-actix-system");
            let watcher = ChainWatcher::launch(pubsub_client);

            tx.send(watcher).unwrap();
            let _ = sys.run();
        });
        let watcher = futures03::executor::block_on(rx).expect("Init chain watcher fail.");

        Self {
            inner: Mutex::new(Some(inner)),
            conn_source,
            chain_watcher: watcher,
            watcher_handle: handle,
        }
    }

    pub fn connect_websocket(url: &str, rt: &mut Runtime) -> anyhow::Result<Self> {
        let conn = ws::try_connect(url).map_err(|e| anyhow::Error::new(e.compat()))?;
        let client = rt.block_on(conn.map_err(map_err))?;
        Ok(Self::new(ConnSource::WebSocket(url.to_string()), client))
    }

    pub fn connect_local<S>(rpc_service: S) -> anyhow::Result<Self>
    where
        S: RpcAsyncService,
    {
        let client = futures03::executor::block_on(async { rpc_service.connect_local().await })?;
        Ok(Self::new(ConnSource::Local, client.into()))
    }

    pub fn connect_ipc<P: AsRef<Path>>(sock_path: P, rt: &mut Runtime) -> anyhow::Result<Self> {
        let reactor = Reactor::new().unwrap();
        let path = sock_path.as_ref().to_path_buf();
        let conn = ipc::connect(sock_path, &reactor.handle())?;
        let client_inner = rt.block_on(conn.map_err(map_err))?;
        //TODO use futures block_on replace rt.
        //let client_inner = futures03::executor::block_on(conn.map_err(map_err).compat())?;

        Ok(Self::new(
            ConnSource::Ipc(path, Arc::new(reactor)),
            client_inner,
        ))
    }

    pub fn watch_txn(
        &self,
        txn_hash: HashValue,
        timeout: Option<Duration>,
    ) -> anyhow::Result<ThinHeadBlock> {
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

    pub fn watch_block(&self, block_number: BlockNumber) -> anyhow::Result<ThinHeadBlock> {
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
    pub fn node_metrics(&self) -> anyhow::Result<HashMap<String, String>> {
        self.call_rpc_blocking(|inner| async move { inner.node_client.metrics().compat().await })
            .map_err(map_err)
    }

    pub fn node_peers(&self) -> anyhow::Result<Vec<PeerInfo>> {
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

    pub fn submit_transaction(
        &self,
        txn: SignedUserTransaction,
    ) -> anyhow::Result<Result<(), anyhow::Error>> {
        self.call_rpc_blocking(|inner| async move {
            inner.txpool_client.submit_transaction(txn).compat().await
        })
        .map(|r| r.map_err(|e| anyhow::format_err!("{}", e)))
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
                .unlock(address, password, duration)
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

    pub fn contract_call(&self, call: ContractCall) -> anyhow::Result<Vec<AnnotatedMoveValue>> {
        self.call_rpc_blocking(|inner| async move {
            inner.dev_client.call_contract(call).compat().await
        })
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

    pub fn sleep(&self, time: u64) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move { inner.debug_client.sleep(time).compat().await })
            .map_err(map_err)
    }

    pub fn chain_head(&self) -> anyhow::Result<ChainInfo> {
        self.call_rpc_blocking(|inner| async move { inner.chain_client.head().compat().await })
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

    pub fn tps(&self, number: Option<BlockNumber>) -> anyhow::Result<u64> {
        self.call_rpc_blocking(
            |inner| async move { inner.chain_client.tps(number).compat().await },
        )
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

    pub fn chain_get_block_by_hash(&self, hash: HashValue) -> anyhow::Result<Block> {
        self.call_rpc_blocking(|inner| async move {
            inner.chain_client.get_block_by_hash(hash).compat().await
        })
        .map_err(map_err)
    }

    pub fn chain_get_block_by_uncle(&self, uncle_id: HashValue) -> anyhow::Result<Option<Block>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_block_by_uncle(uncle_id)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn chain_get_block_by_number(&self, number: BlockNumber) -> anyhow::Result<Block> {
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
    ) -> anyhow::Result<Vec<Block>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_blocks_by_number(number, count)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn chain_get_transaction(&self, txn_id: HashValue) -> anyhow::Result<Transaction> {
        self.call_rpc_blocking(|inner| async move {
            inner.chain_client.get_transaction(txn_id).compat().await
        })
        .map_err(map_err)
    }

    pub fn chain_get_transaction_info(
        &self,
        txn_hash: HashValue,
    ) -> anyhow::Result<Option<TransactionInfo>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_transaction_info(txn_hash)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn chain_get_events_by_txn_info_id(
        &self,
        txn_info_id: HashValue,
    ) -> anyhow::Result<Vec<ContractEvent>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_events_by_txn_info_id(txn_info_id)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn chain_get_txn_by_block(
        &self,
        block_id: HashValue,
    ) -> anyhow::Result<Vec<TransactionInfo>> {
        self.call_rpc_blocking(|inner| async move {
            inner.chain_client.get_txn_by_block(block_id).compat().await
        })
        .map_err(map_err)
    }

    pub fn chain_get_txn_info_by_block_and_index(
        &self,
        block_id: HashValue,
        idx: u64,
    ) -> anyhow::Result<Option<TransactionInfo>> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .chain_client
                .get_txn_info_by_block_and_index(block_id, idx)
                .compat()
                .await
        })
        .map_err(map_err)
    }

    pub fn dry_run(
        &self,
        txn: SignedUserTransaction,
    ) -> anyhow::Result<(VMStatus, TransactionOutput)> {
        self.call_rpc_blocking(|inner| async move { inner.dev_client.dry_run(txn).compat().await })
            .map_err(map_err)
    }
    pub fn miner_submit(&self, header_hash: HashValue, nonce: u64) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move {
            inner.miner_client.submit(header_hash, nonce).compat().await
        })
        .map_err(map_err)
    }

    pub fn txpool_status(&self) -> anyhow::Result<TxPoolStatus> {
        self.call_rpc_blocking(|inner| async move { inner.txpool_client.state().compat().await })
            .map_err(map_err)
    }

    pub fn subscribe_events(
        &self,
        filter: EventFilter,
    ) -> anyhow::Result<impl TryStream<Ok = Event, Error = anyhow::Error>> {
        self.call_rpc_blocking(|inner| async move {
            let res = inner.pubsub_client.subscribe_events(filter).await;
            res.map(|s| s.compat().map_err(map_err))
        })
        .map_err(map_err)
    }
    pub fn subscribe_new_blocks(
        &self,
    ) -> anyhow::Result<impl TryStream<Ok = ThinHeadBlock, Error = anyhow::Error>> {
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
    fn call_rpc_blocking<F, T>(
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
                let conn_source = self.conn_source.clone();
                let f = async { Self::get_rpc_channel(conn_source).await.map(|c| c.into()) };
                let new_inner: RpcClientInner = futures03::executor::block_on(f)?;
                *(self.inner.lock()) = Some(new_inner.clone());
                new_inner
            }
        };

        let f = async { f(inner).await };
        let result = futures03::executor::block_on(f);

        if let Err(rpc_error) = &result {
            if let jsonrpc_client_transports::RpcError::Other(e) = rpc_error {
                error!("rpc error due to {:?}", e);
                *(self.inner.lock()) = None;
            }
        }

        result
    }

    pub fn sync_status(&self) -> anyhow::Result<SyncStatus> {
        self.call_rpc_blocking(|inner| async move { inner.sync_client.status().compat().await })
            .map_err(map_err)
    }

    pub fn sync_progress(&self) -> anyhow::Result<Option<TaskProgressReport>> {
        self.call_rpc_blocking(|inner| async move { inner.sync_client.progress().compat().await })
            .map_err(map_err)
    }

    pub fn sync_start(&self, force: bool) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move { inner.sync_client.start(force).compat().await })
            .map_err(map_err)
    }

    pub fn sync_cancel(&self) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move { inner.sync_client.cancel().compat().await })
            .map_err(map_err)
    }

    async fn get_rpc_channel(
        conn_source: ConnSource,
    ) -> anyhow::Result<RpcChannel, jsonrpc_client_transports::RpcError> {
        let conn_provider = ConnectionProvider { conn_source };
        match conn_provider.get_rpc_channel().await {
            Ok(channel) => Ok(channel),
            Err(ConnError::RpcError(e)) => Err(e),
            Err(ConnError::Io(e)) => Err(jsonrpc_client_transports::RpcError::Other(
                failure::Error::from(e),
            )),
        }
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
    node_client: NodeClient,
    node_manager_client: NodeManagerClient,
    txpool_client: TxPoolClient,
    account_client: AccountClient,
    state_client: StateClient,
    debug_client: DebugClient,
    chain_client: ChainClient,
    pubsub_client: PubSubClient,
    dev_client: DevClient,
    miner_client: MinerClient,
    sync_client: SyncManagerClient,
}

impl RpcClientInner {
    pub fn new(channel: RpcChannel) -> Self {
        Self {
            node_client: channel.clone().into(),
            node_manager_client: channel.clone().into(),
            txpool_client: channel.clone().into(),
            account_client: channel.clone().into(),
            state_client: channel.clone().into(),
            debug_client: channel.clone().into(),
            chain_client: channel.clone().into(),
            dev_client: channel.clone().into(),
            pubsub_client: channel.clone().into(),
            miner_client: channel.clone().into(),
            sync_client: channel.into(),
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
