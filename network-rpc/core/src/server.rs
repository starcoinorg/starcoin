use crate::delegates::RpcMethod;
use actix::{Actor, Addr, Arbiter, AsyncContext, Context, StreamHandler};
use anyhow::{format_err, Result};
use futures::channel::mpsc;
use futures::channel::mpsc::Sender;
use futures::SinkExt;
use logger::prelude::*;
use network_api::messages::RawRpcRequestMessage;
use starcoin_types::CHAIN_PROTOCOL_NAME;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

pub struct NetworkRpcServer {
    methods: HashMap<String, Arc<dyn RpcMethod>>,
}

impl NetworkRpcServer {
    pub fn start<F>(
        rpc_rx: mpsc::UnboundedReceiver<RawRpcRequestMessage>,
        rpc_methods: F,
    ) -> Result<Addr<NetworkRpcServer>>
    where
        F: IntoIterator<Item = (String, Arc<dyn RpcMethod>)>,
    {
        Ok(NetworkRpcServer::create(move |ctx| {
            let mut methods: HashMap<String, Arc<dyn RpcMethod>> = Default::default();
            methods.extend(rpc_methods);
            ctx.add_stream(rpc_rx);
            NetworkRpcServer { methods }
        }))
    }
    async fn do_response(
        responder: Sender<(Cow<'static, [u8]>, Vec<u8>)>,
        resp: Vec<u8>,
    ) -> Result<()> {
        if let Err(e) = responder
            .clone()
            .send((CHAIN_PROTOCOL_NAME.into(), resp))
            .await
        {
            Err(format_err!("{:?}", e))
        } else {
            Ok(())
        }
    }
}

impl Actor for NetworkRpcServer {
    type Context = Context<Self>;
}

impl StreamHandler<RawRpcRequestMessage> for NetworkRpcServer {
    fn handle(&mut self, req_msg: RawRpcRequestMessage, _ctx: &mut Self::Context) {
        let responder = req_msg.responder.clone();
        let (path, request, peer_id) = req_msg.request;
        if let Some(method) = self.methods.get(&path) {
            let method = method.clone();
            Arbiter::spawn(async move {
                if let Ok(response) = method.call(peer_id, request).await {
                    if let Err(e) = Self::do_response(responder, response).await {
                        error!("{:?}", e);
                    };
                } else {
                    // TODO: Do not handle for back compatibly
                    error!("network rpc call return custom error");
                }
            })
        } else {
            warn!(
                "network rpc method received not defined in server: {:?}",
                path
            );
        }
    }
}
