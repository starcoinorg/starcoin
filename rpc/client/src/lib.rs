// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use failure::Fail;
use futures::{future::FutureExt, select, stream::StreamExt};
use futures01::future::Future as Future01;
use jsonrpc_core::{MetaIoHandler, Metadata};
use jsonrpc_core_client::{transports::http, transports::ipc, transports::local, RpcChannel};
use starcoin_logger::prelude::*;
use starcoin_rpc_api::{account::AccountClient, status::StatusClient, txpool::TxPoolClient};
use starcoin_types::transaction::{RawUserTransaction, SignedUserTransaction};
use starcoin_wallet_api::WalletAccount;
use std::cell::RefCell;
use std::ops::Deref;
use std::path::Path;
use std::time::Duration;
use tokio01::reactor::Reactor;
use tokio_compat::prelude::*;
use tokio_compat::runtime::Runtime;

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

    pub fn status(&self) -> anyhow::Result<bool> {
        self.rt.borrow_mut().block_on_std(async {
            self.inner
                .status_client
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
}

impl AsRef<RpcClientInner> for RpcClient {
    fn as_ref(&self) -> &RpcClientInner {
        &self.inner
    }
}

pub(crate) struct RpcClientInner {
    status_client: StatusClient,
    txpool_client: TxPoolClient,
    account_client: AccountClient,
}

impl RpcClientInner {
    pub fn new(channel: RpcChannel) -> Self {
        Self {
            status_client: channel.clone().into(),
            txpool_client: channel.clone().into(),
            account_client: channel.into(),
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
