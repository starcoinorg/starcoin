// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::{Addr, ContextFutureSpawner, WrapFuture};
use futures::compat::Future01CompatExt;
use jsonrpc_pubsub::typed::Sink;
use starcoin_logger::prelude::*;

pub fn notify<T>(notifier: &Addr<SubscriberNotifyActor<T>>, result: T)
where
    T: Unpin + 'static + Send + serde::Serialize,
{
    let n = Notification(result);
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
pub struct Notification<T>(T);
impl<T> actix::Message for Notification<T> {
    type Result = ();
}

// Use an actor to keep notification ordered without blocking upper layer.
impl<T> actix::Handler<Notification<T>> for SubscriberNotifyActor<T>
where
    T: serde::Serialize + Unpin + 'static + Send,
{
    type Result = ();

    fn handle(&mut self, msg: Notification<T>, ctx: &mut Self::Context) -> Self::Result {
        let result = msg.0;
        let notify = self.client.notify(Ok(result));
        async move {
            match notify.compat().await {
                Ok(_) => {}
                Err(e) => warn!(target: "rpc", "Unable to send notification: {}", e),
            }
        }
        .into_actor(self)
        .wait(ctx);
    }
}
