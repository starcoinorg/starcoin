// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::delegates::RpcMethod;
use anyhow::{format_err, Result};
use futures::channel::mpsc::Sender;
use futures::SinkExt;
use logger::prelude::*;
use network_api::messages::RawRpcRequestMessage;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

pub struct NetworkRpcServer {
    //TODO remove this field after refactor RawRpcRequestMessage,because request should know it's protocol name.
    protocol_name: Cow<'static, [u8]>,
    methods: HashMap<String, Arc<dyn RpcMethod>>,
}

impl NetworkRpcServer {
    pub fn new<F>(protocol_name: Cow<'static, [u8]>, rpc_methods: F) -> Self
    where
        F: IntoIterator<Item = (String, Arc<dyn RpcMethod>)>,
    {
        let mut methods: HashMap<String, Arc<dyn RpcMethod>> = Default::default();
        methods.extend(rpc_methods);
        NetworkRpcServer {
            protocol_name,
            methods,
        }
    }
    async fn do_response(
        protocol_name: Cow<'static, [u8]>,
        responder: Sender<(Cow<'static, [u8]>, Vec<u8>)>,
        resp: Vec<u8>,
    ) -> Result<()> {
        if let Err(e) = responder.clone().send((protocol_name, resp)).await {
            Err(format_err!("sender to responder error: {:?}", e))
        } else {
            Ok(())
        }
    }

    pub async fn handle_request(&self, req_msg: RawRpcRequestMessage) -> Result<()> {
        let responder = req_msg.responder.clone();
        let (path, request, peer_id) = req_msg.request;
        if let Some(method) = self.methods.get(&path) {
            let method = method.clone();
            let response = method.call(peer_id, request).await;
            Self::do_response(self.protocol_name.clone(), responder, response).await
        } else {
            //TODO send method not found error to client.
            warn!(
                "network rpc method received not defined in server: {:?}",
                path
            );
            Ok(())
        }
    }
}
