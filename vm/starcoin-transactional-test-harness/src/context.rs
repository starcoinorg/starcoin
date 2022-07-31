use anyhow::{anyhow, Result};
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_genesis::Genesis;
use starcoin_statedb::ChainStateDB;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

use jsonrpc_client_transports::{RawClient, RpcChannel};
use jsonrpc_core::futures::{self, future, TryFutureExt};
use jsonrpc_core::{BoxFuture, IoHandler, Params, Value};
use jsonrpc_core_client::transports::local;
use starcoin_rpc_api::chain::ChainApi;
use starcoin_rpc_api::state::StateApi;

use crate::fork_chain::{ForkBlockChain, MockApi};
use crate::in_memory_state_cache::InMemoryStateCache;
use crate::remote_state::{RemoteRpcAsyncClient, RemoteViewer, SelectableStateView};

pub struct MockServer {
    server_handle: JoinHandle<()>,
}

impl MockServer {
    pub fn create_and_start(api: MockApi, rt: Arc<Runtime>) -> Result<(Self, RawClient)> {
        let mut io = IoHandler::new();
        io.extend_with(ChainApi::to_delegate(api.clone()));
        io.extend_with(StateApi::to_delegate(api.clone()));

        let (client, server) = local::connect::<RawClient, _, _>(io);
        let server_handle = rt.spawn(async move { server.await.unwrap() });
        Ok((Self { server_handle }, client))
    }
}

pub struct ForkContext {
    pub chain: Arc<Mutex<ForkBlockChain>>,
    pub storage: SelectableStateView<ChainStateDB, InMemoryStateCache<RemoteViewer>>,
    server: MockServer,
    client: RawClient,
    rt: Arc<Runtime>,
}

impl ForkContext {
    pub fn new_local(network: BuiltinNetworkID) -> Result<Self> {
        let rt = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .thread_name("fork-context-worker")
                .enable_all()
                .build()?,
        );
        let net = ChainNetwork::new_builtin(network);
        let genesis_txn = Genesis::build_genesis_transaction(&net).unwrap();
        let data_store = ChainStateDB::mock();
        Genesis::execute_genesis_txn(&data_store, genesis_txn).unwrap();
        let storage = SelectableStateView::A(data_store);
        let chain = Arc::new(Mutex::new(ForkBlockChain::new()?));

        Self::new_inner(chain, storage, rt)
    }

    pub fn new_fork(rpc: &str, block_number: Option<u64>) -> Result<Self> {
        let rt = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .thread_name("fork-context-worker")
                .enable_all()
                .build()?,
        );

        let remote_async_client =
            Arc::new(rt.block_on(async {
                RemoteRpcAsyncClient::from_url(&rpc[..], block_number).await
            })?);
        let remote_viewer = RemoteViewer::new(remote_async_client.clone(), rt.clone());
        let storage = SelectableStateView::B(InMemoryStateCache::new(remote_viewer));

        let fork_nubmer = remote_async_client.get_fork_block_number();
        let chain = Arc::new(Mutex::new(ForkBlockChain::fork(
            remote_async_client,
            fork_nubmer,
        )?));

        Self::new_inner(chain, storage, rt)
    }

    fn new_inner(
        chain: Arc<Mutex<ForkBlockChain>>,
        storage: SelectableStateView<ChainStateDB, InMemoryStateCache<RemoteViewer>>,
        rt: Arc<Runtime>,
    ) -> Result<Self> {
        let mock_api = MockApi::new(chain.clone());
        let (server, client) = MockServer::create_and_start(mock_api, rt.clone())?;

        Ok(Self {
            chain,
            storage,
            server,
            client,
            rt,
        })
    }

    pub fn call_api(&self, method: &str, params: Params) -> Result<Value> {
        let handle = self.rt.handle().clone();
        let client = self.client.clone();
        handle
            .block_on(async move { client.call_method(method, params).await })
            .map_err(|e| anyhow!(format!("{}", e)))
    }
}
