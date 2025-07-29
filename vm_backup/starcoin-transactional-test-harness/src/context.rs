// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use futures::executor::block_on;
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_crypto::HashValue;
use starcoin_genesis::Genesis;
use starcoin_rpc_server::module::StateRpcImpl;
use starcoin_state_api::{ChainStateReader, ChainStateWriter, StateNodeStore};
use starcoin_statedb::ChainStateDB;
use starcoin_types::write_set::WriteSet;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use tokio::runtime::Runtime;

use jsonrpc_client_transports::RawClient;
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_core_client::transports::local;
use starcoin_rpc_api::chain::ChainApi;
use starcoin_rpc_api::state::StateApi;
use starcoin_state_tree;

use crate::fork_chain::{ForkBlockChain, MockChainApi};
use crate::fork_state::{MockChainStateAsyncService, MockStateNodeStore};
use crate::remote_state::RemoteRpcAsyncClient;

pub struct MockServer {
    _server_handle: JoinHandle<()>,
}

impl MockServer {
    pub fn create_and_start(
        chain_api: MockChainApi,
        state_api: impl StateApi,
    ) -> Result<(Self, RawClient)> {
        let mut io = IoHandler::new();
        io.extend_with(ChainApi::to_delegate(chain_api));
        io.extend_with(StateApi::to_delegate(state_api));

        let (client, server) = local::connect::<RawClient, _, _>(io);
        let server_handle = thread::spawn(move || block_on(server).unwrap());

        Ok((
            Self {
                _server_handle: server_handle,
            },
            client,
        ))
    }
}

pub struct ForkContext {
    pub chain: Arc<Mutex<ForkBlockChain>>,
    pub storage: ChainStateDB,
    _server: MockServer,
    client: RawClient,
    rt: Arc<Runtime>,
    state_root: Arc<Mutex<HashValue>>,
}

impl ForkContext {
    pub fn new_local(
        network: BuiltinNetworkID,
        stdlib_modules: Option<Vec<Vec<u8>>>,
    ) -> Result<Self> {
        let rt = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .thread_name("fork-context-worker")
                .enable_all()
                .build()?,
        );
        let net = ChainNetwork::new_builtin(network);
        let genesis_txn = match stdlib_modules {
            Some(module) => Genesis::build_genesis_transaction_with_stdlib(&net, module).unwrap(),
            None => Genesis::build_genesis_transaction(&net).unwrap(),
        };
        let data_store = Arc::new(starcoin_state_tree::mock::MockStateNodeStore::new());
        let state_db = ChainStateDB::new(data_store.clone(), None);
        Genesis::execute_genesis_txn(&state_db, genesis_txn).unwrap();

        let state_root = state_db.state_root();
        let state_root = Arc::new(Mutex::new(state_root));
        let chain = Arc::new(Mutex::new(ForkBlockChain::new(state_root.clone())?));
        Self::new_inner(chain, state_db, data_store, rt, state_root)
    }

    pub fn new_fork(rpc: &str, block_number: Option<u64>) -> Result<Self> {
        let rt = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .thread_name("fork-context-worker")
                .enable_all()
                .build()?,
        );

        let remote_async_client = Arc::new(
            rt.block_on(async { RemoteRpcAsyncClient::from_url(rpc, block_number).await })?,
        );
        let state_api_client = Arc::new(remote_async_client.get_state_client().clone());
        let root_hash = remote_async_client.get_fork_state_root();
        let data_store = Arc::new(MockStateNodeStore::new(state_api_client, rt.clone()));
        let state_db = ChainStateDB::new(data_store.clone(), Some(root_hash));

        let fork_nubmer = remote_async_client.get_fork_block_number();
        let fork_block_hash = remote_async_client.get_fork_block_hash();
        let state_root = Arc::new(Mutex::new(root_hash));
        let chain = Arc::new(Mutex::new(ForkBlockChain::fork(
            remote_async_client,
            fork_nubmer,
            fork_block_hash,
            state_root.clone(),
        )?));
        Self::new_inner(chain, state_db, data_store, rt, state_root)
    }

    fn new_inner(
        chain: Arc<Mutex<ForkBlockChain>>,
        storage: ChainStateDB,
        data_store: Arc<dyn StateNodeStore>,
        rt: Arc<Runtime>,
        state_root: Arc<Mutex<HashValue>>,
    ) -> Result<Self> {
        let chain_api = MockChainApi::new(chain.clone());
        let state_svc = MockChainStateAsyncService::new(data_store.clone(), state_root.clone());
        let state_api = StateRpcImpl::new(state_svc, data_store);
        let (server, client) = MockServer::create_and_start(chain_api, state_api)?;

        Ok(Self {
            chain,
            storage,
            _server: server,
            client,
            rt,
            state_root,
        })
    }

    pub fn call_api(&self, method: &str, params: Params) -> Result<Value> {
        let handle = self.rt.handle().clone();
        let client = self.client.clone();
        handle
            .block_on(async move { client.call_method(method, params).await })
            .map_err(|e| anyhow!(format!("{}", e)))
    }

    pub fn apply_write_set(&self, write_set: WriteSet) -> anyhow::Result<()> {
        self.storage.apply_write_set(write_set)?;
        let state_root = self.storage.commit()?;
        *self.state_root.lock().unwrap() = state_root;
        self.storage.flush()?;
        Ok(())
    }
}
