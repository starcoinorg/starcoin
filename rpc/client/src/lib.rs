// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use failure::Fail;
use futures::{future::FutureExt, select, stream::StreamExt};
use futures01::future::Future as Future01;
use jsonrpc_core::{MetaIoHandler, Metadata};
use jsonrpc_core_client::{transports::http, transports::ipc, transports::local, RpcChannel};
use starcoin_logger::prelude::*;
use starcoin_rpc_api::{status::StatusClient, txpool::TxPoolClient};
use starcoin_types::transaction::SignedUserTransaction;
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

    pub fn connect_ipc<P: AsRef<Path>>(sock_path: P) -> anyhow::Result<Self> {
        let mut rt = Runtime::new().unwrap();
        let reactor = Reactor::new().unwrap();

        let fut = ipc::connect(sock_path, &reactor.handle())?;
        let client_inner = rt.block_on(fut.map_err(map_err))?;

        Ok(Self::new(client_inner, rt))
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
}

impl RpcClientInner {
    pub fn new(channel: RpcChannel) -> Self {
        Self {
            status_client: channel.clone().into(),
            txpool_client: channel.into(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix::prelude::*;
    use anyhow::Result;
    use futures::channel::oneshot;
    use starcoin_config::NodeConfig;
    use starcoin_rpc_server::JSONRpcActor;
    use starcoin_traits::mock::MockTxPoolService;
    use starcoin_traits::TxPoolAsyncService;
    use std::sync::Arc;
    use std::time::Duration;

    #[ignore]
    #[test]
    fn test_multi_client() -> Result<()> {
        starcoin_logger::init_for_test();
        let mut system = System::new("test");

        let config = Arc::new(NodeConfig::random_for_test());
        let http_address = config.rpc.http_address.as_ref().unwrap();
        let ipc_file = config.rpc.get_ipc_file(config.data_dir.as_path());
        let url = format!("http://{}", http_address.to_string());
        info!("url:{}", url);

        system.block_on(async {
            let (stop_sender, stop_receiver) = oneshot::channel::<bool>();
            let txpool = MockTxPoolService::new();
            let (_rpc_actor, iohandler) = JSONRpcActor::launch(config, txpool).unwrap();

            let client_task = move || {
                info!("client thread start.");
                std::thread::sleep(Duration::from_millis(500));

                let http_client = RpcClient::connect_http(url.as_str()).unwrap();
                let status = http_client.status().unwrap();
                info!("http_client status: {}", status);
                assert!(status);

                let ipc_client = RpcClient::connect_ipc(ipc_file).unwrap();
                let status1 = ipc_client.status().unwrap();
                info!("ipc_client status: {}", status1);
                assert_eq!(status, status1);

                let local_client = RpcClient::connect_local(iohandler);
                let status2 = local_client.status().unwrap();
                info!("local_client status: {}", status2);
                assert!(status2);

                drop(stop_sender);
            };

            let handle = std::thread::spawn(client_task);

            debug!("wait server stop");
            debug!("stop receiver: {}", stop_receiver.await.is_ok());
            handle.join().unwrap();
            debug!("server stop.");
        });

        Ok(())
    }

    //FIXME
    #[ignore]
    #[stest::test]
    async fn test_txpool() -> Result<()> {
        let config = Arc::new(NodeConfig::random_for_test());
        let ipc_file = config.rpc.get_ipc_file(config.data_dir.as_path());

        let (stop_sender, stop_receiver) = oneshot::channel::<bool>();
        let txpool = MockTxPoolService::new();
        let (_rpc_actor, _iohandler) = JSONRpcActor::launch(config, txpool.clone()).unwrap();

        let client_task = move || {
            info!("client thread start.");
            std::thread::sleep(Duration::from_millis(500));

            let ipc_client = RpcClient::connect_ipc(ipc_file).unwrap();
            let result = ipc_client
                .submit_transaction(SignedUserTransaction::mock())
                .unwrap();
            info!("ipc_client submit_transaction result: {}", result);
            assert!(result);

            drop(stop_sender);
        };

        let handle = std::thread::spawn(client_task);
        debug!("stop receiver: {}", stop_receiver.await.is_ok());
        handle.join().unwrap();
        assert_eq!(1, txpool.get_pending_txns(None).await?.len());
        Ok(())
    }
}
