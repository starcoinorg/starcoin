// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use failure::Fail;
use futures::{future::FutureExt, select, stream::StreamExt};
use futures01::future::Future as Future01;
use jsonrpc_core::{MetaIoHandler, Metadata};
use jsonrpc_core_client::{transports::http, transports::ipc, transports::local, RpcChannel};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::{
    debug::DebugClient, node::NodeClient, state::StateClient, txpool::TxPoolClient,
    wallet::WalletClient,
};
use starcoin_state_api::StateWithProof;
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_wallet_api::WalletAccount;
use std::cell::RefCell;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;
use tokio01::reactor::Reactor;
use tokio_compat::prelude::*;
use tokio_compat::runtime::Runtime;

mod remote_state_reader;

pub use crate::remote_state_reader::RemoteStateReader;
use starcoin_rpc_api::node::NodeInfo;
use starcoin_types::peer_info::PeerInfo;
use std::sync::Arc;

#[derive(Debug, Clone)]
enum ConnSource {
    Ipc(PathBuf, Arc<Reactor>),
    Http,
    Local,
}

pub struct RpcClient {
    inner: RefCell<Option<RpcClientInner>>,
    rt: RefCell<Runtime>,
    conn_source: ConnSource,
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
                conn_fut.compat().await.map_err(|e| ConnError::RpcError(e))
            }
            // only have ipc impl for now
            _ => unreachable!(),
        }
    }
}

impl RpcClient {
    pub(crate) fn new(conn_source: ConnSource, inner: RpcClientInner, rt: Runtime) -> Self {
        Self {
            inner: RefCell::new(Some(inner)),
            rt: RefCell::new(rt),
            conn_source,
        }
    }
    pub fn connect_http(url: &str) -> anyhow::Result<Self> {
        let mut rt = Runtime::new().unwrap();
        let client_inner = rt.block_on(http::connect(url).map_err(map_err))?;
        Ok(Self::new(ConnSource::Http, client_inner, rt))
    }

    pub fn connect_local<THandler, TMetadata>(handler: THandler) -> Self
    where
        THandler: Deref<Target = MetaIoHandler<TMetadata>> + std::marker::Send + 'static,
        TMetadata: Metadata + Default,
    {
        let rt = Runtime::new().unwrap();
        let (client, future) = local::connect(handler);
        // process server event interval.
        // TODO use more graceful method.
        rt.spawn_std(async {
            let mut future = future
                .map_err(|e| error!("rpc error: {:?}", e))
                .compat()
                .fuse();
            let mut timer = tokio::time::interval(Duration::from_millis(10)).fuse();
            loop {
                select! {
                res = future => {
                },
                t = timer.select_next_some() =>{
                }
                complete => break,
                };
            }
        });
        Self::new(ConnSource::Local, client, rt)
    }

    pub fn connect_ipc<P: AsRef<Path>>(sock_path: P) -> anyhow::Result<Self> {
        let mut rt = Runtime::new().unwrap();
        let reactor = Reactor::new().unwrap();
        let path = sock_path.as_ref().to_path_buf();
        let fut = ipc::connect(sock_path, &reactor.handle())?;
        let client_inner = rt.block_on(fut.map_err(map_err))?;

        Ok(Self::new(
            ConnSource::Ipc(path, Arc::new(reactor)),
            client_inner,
            rt,
        ))
    }

    pub fn node_status(&self) -> anyhow::Result<bool> {
        self.call_rpc_blocking(|inner| async move { inner.node_client.status().compat().await })
            .map_err(map_err)
    }

    pub fn node_info(&self) -> anyhow::Result<NodeInfo> {
        self.call_rpc_blocking(|inner| async move { inner.node_client.info().compat().await })
            .map_err(map_err)
    }

    pub fn node_peers(&self) -> anyhow::Result<Vec<PeerInfo>> {
        self.call_rpc_blocking(|inner| async move { inner.node_client.peers().compat().await })
            .map_err(map_err)
    }

    pub fn submit_transaction(&self, txn: SignedUserTransaction) -> anyhow::Result<bool> {
        self.call_rpc_blocking(|inner| async move {
            inner.txpool_client.submit_transaction(txn).compat().await
        })
        .map_err(map_err)
    }
    //TODO should split client for different api ?
    // such as  RpcClient().account().default()
    pub fn wallet_default(&self) -> anyhow::Result<Option<WalletAccount>> {
        self.call_rpc_blocking(|inner| async move { inner.wallet_client.default().compat().await })
            .map_err(map_err)
    }

    pub fn wallet_create(&self, password: String) -> anyhow::Result<WalletAccount> {
        self.call_rpc_blocking(|inner| async move {
            inner.wallet_client.create(password).compat().await
        })
        .map_err(map_err)
    }

    pub fn wallet_list(&self) -> anyhow::Result<Vec<WalletAccount>> {
        self.call_rpc_blocking(|inner| async move { inner.wallet_client.list().compat().await })
            .map_err(map_err)
    }

    pub fn wallet_get(&self, address: AccountAddress) -> anyhow::Result<Option<WalletAccount>> {
        self.call_rpc_blocking(
            |inner| async move { inner.wallet_client.get(address).compat().await },
        )
        .map_err(map_err)
    }

    pub fn wallet_sign_txn(
        &self,
        raw_txn: RawUserTransaction,
    ) -> anyhow::Result<SignedUserTransaction> {
        self.call_rpc_blocking(|inner| async move {
            inner.wallet_client.sign_txn(raw_txn).compat().await
        })
        .map_err(map_err)
    }

    pub fn wallet_unlock(
        &self,
        address: AccountAddress,
        password: String,
        duration: std::time::Duration,
    ) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .wallet_client
                .unlock(address, password, duration)
                .compat()
                .await
        })
        .map_err(map_err)
    }
    pub fn wallet_export(
        &self,
        address: AccountAddress,
        password: String,
    ) -> anyhow::Result<Vec<u8>> {
        self.call_rpc_blocking(|inner| async move {
            inner.wallet_client.export(address, password).compat().await
        })
        .map_err(map_err)
    }
    pub fn wallet_import(
        &self,
        address: AccountAddress,
        private_key: Vec<u8>,
        password: String,
    ) -> anyhow::Result<WalletAccount> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .wallet_client
                .import(address, private_key, password)
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

    pub fn debug_set_log_level(&self, level: Level) -> anyhow::Result<()> {
        self.call_rpc_blocking(|inner| async move {
            inner
                .debug_client
                .set_log_level(level.to_string())
                .compat()
                .await
        })
        .map_err(map_err)
    }

    fn call_rpc_blocking<F, T>(
        &self,
        f: impl FnOnce(RpcClientInner) -> F,
    ) -> Result<T, jsonrpc_client_transports::RpcError>
    where
        F: std::future::Future<Output = Result<T, jsonrpc_client_transports::RpcError>>,
    {
        let inner_opt = self.inner.borrow().as_ref().cloned();
        let inner = match inner_opt {
            Some(inner) => inner,
            None => {
                let new_inner: RpcClientInner = self.rt.borrow_mut().block_on_std(async {
                    Self::get_rpc_channel(self.conn_source.clone())
                        .await
                        .map(|c| c.into())
                })?;
                *self.inner.borrow_mut() = Some(new_inner.clone());
                new_inner
            }
        };

        let result = self.rt.borrow_mut().block_on_std(async { f(inner).await });

        if let Err(rpc_error) = &result {
            if let jsonrpc_client_transports::RpcError::Other(e) = rpc_error {
                error!("rpc error due to {:?}", e);
                *self.inner.borrow_mut() = None;
            }
        }

        result
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
}

// impl AsRef<RpcClientInner> for RpcClient {
//     fn as_ref(&self) -> &RpcClientInner {
//         &self.inner
//     }
// }

#[derive(Clone)]
pub(crate) struct RpcClientInner {
    node_client: NodeClient,
    txpool_client: TxPoolClient,
    wallet_client: WalletClient,
    state_client: StateClient,
    debug_client: DebugClient,
}

impl RpcClientInner {
    pub fn new(channel: RpcChannel) -> Self {
        Self {
            node_client: channel.clone().into(),
            txpool_client: channel.clone().into(),
            wallet_client: channel.clone().into(),
            state_client: channel.clone().into(),
            debug_client: channel.clone().into(),
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
