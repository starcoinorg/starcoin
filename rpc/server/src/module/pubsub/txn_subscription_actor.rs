// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::notify;
use super::pubsub;
use super::TxnSubscribers;
use actix::AsyncContext;
use futures::channel::mpsc;
use starcoin_txpool_api::TxnStatusFullEvent;

pub struct TransactionSubscriptionActor {
    txn_receiver: Option<mpsc::UnboundedReceiver<TxnStatusFullEvent>>,
    subscribers: TxnSubscribers,
}

impl TransactionSubscriptionActor {
    pub fn new(
        subscribers: TxnSubscribers,
        txn_receiver: mpsc::UnboundedReceiver<TxnStatusFullEvent>,
    ) -> Self {
        Self {
            subscribers,
            txn_receiver: Some(txn_receiver),
        }
    }
}

impl actix::Actor for TransactionSubscriptionActor {
    type Context = actix::Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.add_stream(self.txn_receiver.take().unwrap());
    }
}
impl actix::StreamHandler<TxnStatusFullEvent> for TransactionSubscriptionActor {
    fn handle(&mut self, item: TxnStatusFullEvent, _ctx: &mut Self::Context) {
        let hs = item.as_ref().iter().map(|(h, _)| *h).collect::<Vec<_>>();
        for subscriber in self.subscribers.read().values() {
            notify::notify(subscriber, pubsub::Result::TransactionHash(hs.clone()));
        }
    }
}
