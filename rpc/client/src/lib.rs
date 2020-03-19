// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use failure::Fail;
//use futures::compat::Future01CompatExt;
use jsonrpc_core::futures::future::{self, result, FutureResult};
use jsonrpc_core::{Error, Result};
use jsonrpc_core_client::{transports::http, transports::ipc, RpcChannel};
use starcoin_rpc_api::status::{StatusApi, StatusClient};
use std::path::Path;
//use tokio01::runtime::Runtime;
use futures::future::Future;
use futures01::future::Future as Future01;
use std::cell::RefCell;
use tokio01::reactor::Reactor;
use tokio_compat::prelude::*;
use tokio_compat::runtime::{current_thread, Runtime};

pub struct RpcClient {
    inner: RpcClientInner,
    rt: RefCell<Runtime>,
}

impl RpcClient {
    pub fn new(inner: RpcClientInner, rt: Runtime) -> Self {
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

pub struct RpcClientInner {
    status_client: StatusClient,
}

impl RpcClientInner {
    pub fn new(channel: RpcChannel) -> Self {
        Self {
            status_client: channel.into(),
        }
    }

    pub async fn status(&self) -> anyhow::Result<bool> {
        self.status_client.status().map_err(map_err).compat().await
    }
}

pub struct IpcClientHandle {
    pub client: RpcClient,
    _rt: Runtime,
}

impl IpcClientHandle {
    pub fn new(client: RpcClient, rt: Runtime) -> Self {
        Self { client, _rt: rt }
    }
}

impl AsRef<RpcClient> for IpcClientHandle {
    fn as_ref(&self) -> &RpcClient {
        &self.client
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
    use actix::{Arbiter, ContextFutureSpawner, System};
    use anyhow::Result;
    use failure::_core::time::Duration;
    use futures::channel::oneshot;
    use starcoin_config::NodeConfig;
    use starcoin_logger::prelude::*;
    use starcoin_rpc_server::JSONRpcActor;
    use starcoin_traits::mock::MockTxPoolService;
    use std::sync::Arc;

    #[test]
    fn test_client() -> Result<()> {
        starcoin_logger::init_for_test();
        //let rt = Runtime::new().unwrap();
        let config = Arc::new(NodeConfig::random_for_test());
        let http_address = config.rpc.http_address.as_ref().unwrap();
        let url = format!("http://{}", http_address.to_string());
        info!("url:{}", url);
        let (mut sender, mut receiver) = oneshot::channel::<bool>();
        let _system = actix_rt::System::new("test");
        let arbiter = Arbiter::new();
        arbiter.exec_fn(move || {
            Arbiter::spawn(async {
                let txpool = MockTxPoolService::new();
                let _rpc_actor = JSONRpcActor::launch(config, txpool).unwrap();
                debug!("wait server stop");
                receiver.await.unwrap();
                debug!("server stop.");
            });
        });
        std::thread::sleep(Duration::from_millis(500));
        let client = RpcClient::connect_http(url.as_str())?;
        let status = client.status()?;
        info!("status: {}", status);
        drop(sender);
        Ok(())
    }
}
