// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::delegates::RpcMethod;
use crate::PeerId;
use crate::Result;
use crate::{NetRpcError, RawRpcServer};
use futures::future::BoxFuture;
use futures::FutureExt;
use log::warn;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

pub struct NetworkRpcServer {
    methods: HashMap<Cow<'static, str>, Arc<dyn RpcMethod>>,
}

impl NetworkRpcServer {
    pub fn new<F>(rpc_methods: F) -> Self
    where
        F: IntoIterator<Item = (Cow<'static, str>, Arc<dyn RpcMethod>)>,
    {
        let mut methods: HashMap<Cow<'static, str>, Arc<dyn RpcMethod>> = Default::default();
        methods.extend(rpc_methods);
        NetworkRpcServer { methods }
    }

    pub async fn handle_request_async(
        &self,
        peer_id: PeerId,
        rpc_path: Cow<'static, str>,
        message: Vec<u8>,
    ) -> Result<Vec<u8>> {
        if let Some(method) = self.methods.get(&rpc_path) {
            let method = method.clone();
            method
                .call(peer_id, message)
                .await
                .map_err(Into::<NetRpcError>::into)
        } else {
            warn!(
                "network rpc method received not defined in server: {:?}",
                rpc_path
            );
            Err(NetRpcError::method_not_fount(rpc_path))
        }
    }
}

impl RawRpcServer for NetworkRpcServer {
    fn handle_raw_request(
        &self,
        peer_id: PeerId,
        rpc_path: Cow<'static, str>,
        message: Vec<u8>,
    ) -> BoxFuture<Result<Vec<u8>>> {
        self.handle_request_async(peer_id, rpc_path, message)
            .boxed()
    }
}
