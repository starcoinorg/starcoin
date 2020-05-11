// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::{Addr, ContextFutureSpawner, WrapFuture};
use futures::compat::Sink01CompatExt;
use futures::SinkExt;
use jsonrpc_core::error::Error as JsonRpcError;
use jsonrpc_pubsub::typed::Sink;
use starcoin_logger::prelude::*;

pub fn notify<T>(notifier: &Addr<SubscriberNotifyActor<T>>, result: T)
where
    T: Unpin + 'static + Send + serde::Serialize + Clone,
{
    let n = Notification(vec![result]);
    notifier.do_send(n);
}
pub fn notify_many<T>(notifier: &Addr<SubscriberNotifyActor<T>>, results: Vec<T>)
where
    T: Unpin + 'static + Send + serde::Serialize + Clone,
{
    let n = Notification(results);
    notifier.do_send(n);
}

#[derive(Debug, Clone)]
pub struct SubscriberNotifyActor<T> {
    client: Sink<T>,
}

impl<T> SubscriberNotifyActor<T> {
    pub fn new(sink: Sink<T>) -> Self {
        Self { client: sink }
    }
}

impl<T> actix::Actor for SubscriberNotifyActor<T>
where
    T: Unpin + 'static + Send,
{
    type Context = actix::Context<Self>;
}

#[derive(Debug, Clone)]
pub struct Notification<T>(Vec<T>);
impl<T> actix::Message for Notification<T> {
    type Result = ();
}

// Use an actor to keep notification ordered without blocking upper layer.
impl<T> actix::Handler<Notification<T>> for SubscriberNotifyActor<T>
where
    T: serde::Serialize + Unpin + 'static + Send + Clone,
{
    type Result = ();

    fn handle(&mut self, msg: Notification<T>, ctx: &mut Self::Context) -> Self::Result {
        let result = msg.0;
        // let notify = self.client.notify(Ok(result));

        let mut s = futures::stream::iter(result.into_iter().map(Ok::<_, JsonRpcError>).map(Ok));
        let mut client_sink = self.client.clone().sink_compat();
        async move {
            match client_sink.send_all(&mut s).await {
                Ok(_) => {}
                Err(e) => warn!(target: "rpc", "Unable to send notification: {}", e),
            }
        }
        .into_actor(self)
        .wait(ctx);
    }
}
