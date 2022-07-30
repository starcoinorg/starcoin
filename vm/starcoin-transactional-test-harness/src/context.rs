use anyhow::{anyhow, Result};
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use std::sync::{Arc, Mutex};

use jsonrpc_client_transports::RawClient;
use jsonrpc_core_client::transports::local;
use jsonrpc_core::{BoxFuture, IoHandler, Params, Value};
use jsonrpc_core::futures::{self, future, TryFutureExt};
use starcoin_rpc_api::chain::ChainApi;
use starcoin_rpc_api::state::StateApi;

use crate::fork_chain::{ForkBlockChain, MockApi};
use crate::remote_state::RemoteRpcAsyncClient;

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
  server: MockServer,
  client: RawClient,
  rt: Arc<Runtime>,
}

impl ForkContext {
  pub fn new_from_url(rpc_url: &str, block_number: Option<u64>) -> Result<Self> {
    let rt = Arc::new(
      tokio::runtime::Builder::new_multi_thread()
        .thread_name("fork-context-worker")
        .enable_all()
        .build()?
    );

    let remote_viewer = Arc::new(
        rt.block_on(async { RemoteRpcAsyncClient::from_url(rpc_url, block_number).await })?
    );
    
    let fork_nubmer = remote_viewer.get_fork_block_number();
    let chain = Arc::new(Mutex::new(ForkBlockChain::new(remote_viewer, fork_nubmer)?));
    let mock_api = MockApi::new(chain.clone());
    let (server, client) = MockServer::create_and_start(mock_api, rt.clone())?;

    Ok(Self {
      chain,
      server,
      client,
      rt,
    })
  }

  pub fn call_api(&self, method: &str, params: Params) -> Result<Value> {
    let handle = self.rt.handle().clone();
    let client = self.client.clone();
    handle.block_on(async move {
      client
      .call_method(method, params)
      .await
    })
    .map_err(|e| anyhow!(format!("{}", e)))
  }
}
