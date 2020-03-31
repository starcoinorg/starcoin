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
    account::AccountClient, node::NodeClient, state::StateClient, txpool::TxPoolClient,
};
use starcoin_state_api::StateWithProof;
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_wallet_api::WalletAccount;
use std::cell::RefCell;
use std::ops::Deref;
use std::path::Path;
use std::time::Duration;
use tokio01::reactor::Reactor;
use tokio_compat::prelude::*;
use tokio_compat::runtime::Runtime;

mod remote_state_reader;

pub use crate::remote_state_reader::RemoteStateReader;

pub struct RpcClient {
    inner: RpcClientInner,
    rt: RefCell<Runtime>,
}

impl RpcClient {
    pub(crate) fn new(inner: RpcClientInner, rt: Runtime) -> Self {
        Self {
            inner,
            rt: RefCell::new(rt),
        }
    }
    pub fn connect_http(url: &str) -> anyhow::Result<Self> {
        let mut rt = Runtime::new().unwrap();
        let client_inner = rt.block_on(http::connect(url).map_err(map_err))?;
        Ok(Self::new(client_inner, rt))
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
        Self::new(client, rt)
    }

    pub fn connect_ipc<P: AsRef<Path>>(sock_path: P) -> anyhow::Result<Self> {
        let mut rt = Runtime::new().unwrap();
        let reactor = Reactor::new().unwrap();

        let fut = ipc::connect(sock_path, &reactor.handle())?;
        let client_inner = rt.block_on(fut.map_err(map_err))?;

        Ok(Self::new(client_inner, rt))
    }

    pub fn node_status(&self) -> anyhow::Result<bool> {
        self.rt.borrow_mut().block_on_std(async {
            self.inner
                .node_client
                .status()
                .map_err(map_err)
                .compat()
                .await
        })
    }

    pub fn submit_transaction(&self, txn: SignedUserTransaction) -> anyhow::Result<bool> {
        self.rt.borrow_mut().block_on_std(async {
            self.inner
                .txpool_client
                .submit_transaction(txn)
                .map_err(map_err)
                .compat()
                .await
        })
    }
    //TODO should split client for different api ?
    // such as  RpcClient().account().create()
    pub fn account_create(&self, password: String) -> anyhow::Result<WalletAccount> {
        self.rt.borrow_mut().block_on_std(async {
            self.inner
                .account_client
                .create(password)
                .map_err(map_err)
                .compat()
                .await
        })
    }

    pub fn account_list(&self) -> anyhow::Result<Vec<WalletAccount>> {
        self.rt.borrow_mut().block_on_std(async {
            self.inner
                .account_client
                .list()
                .map_err(map_err)
                .compat()
                .await
        })
    }

    pub fn account_get(&self, address: AccountAddress) -> anyhow::Result<Option<WalletAccount>> {
        self.rt.borrow_mut().block_on_std(async {
            self.inner
                .account_client
                .get(address)
                .map_err(map_err)
                .compat()
                .await
        })
    }

    pub fn account_sign_txn(
        &self,
        raw_txn: RawUserTransaction,
    ) -> anyhow::Result<SignedUserTransaction> {
        self.rt.borrow_mut().block_on_std(async {
            self.inner
                .account_client
                .sign_txn(raw_txn)
                .map_err(map_err)
                .compat()
                .await
        })
    }

    pub fn state_get(&self, access_path: AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        self.rt.borrow_mut().block_on_std(async {
            self.inner
                .state_client
                .get(access_path)
                .map_err(map_err)
                .compat()
                .await
        })
    }

    pub fn state_get_with_proof(&self, access_path: AccessPath) -> anyhow::Result<StateWithProof> {
        self.rt.borrow_mut().block_on_std(async {
            self.inner
                .state_client
                .get_with_proof(access_path)
                .map_err(map_err)
                .compat()
                .await
        })
    }

    pub fn state_get_state_root(&self) -> anyhow::Result<HashValue> {
        self.rt.borrow_mut().block_on_std(async {
            self.inner
                .state_client
                .get_state_root()
                .map_err(map_err)
                .compat()
                .await
        })
    }

    pub fn state_get_account_state(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<Option<AccountState>> {
        self.rt.borrow_mut().block_on_std(async {
            self.inner
                .state_client
                .get_account_state(address)
                .map_err(map_err)
                .compat()
                .await
        })
    }
}

impl AsRef<RpcClientInner> for RpcClient {
    fn as_ref(&self) -> &RpcClientInner {
        &self.inner
    }
}

pub(crate) struct RpcClientInner {
    node_client: NodeClient,
    txpool_client: TxPoolClient,
    account_client: AccountClient,
    state_client: StateClient,
}

impl RpcClientInner {
    pub fn new(channel: RpcChannel) -> Self {
        Self {
            node_client: channel.clone().into(),
            txpool_client: channel.clone().into(),
            account_client: channel.clone().into(),
            state_client: channel.clone().into(),
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
