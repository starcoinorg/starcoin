use super::notify;
use super::pubsub;
use super::Subscribers;
use super::TxnSubscribers;

use actix;
use actix::AsyncContext;
use futures_channel::mpsc;
use parking_lot::RwLock;
use starcoin_crypto::HashValue;
use std::sync::Arc;

pub struct TransactionSubscriptionActor {
    txn_receiver: Option<mpsc::UnboundedReceiver<Arc<Vec<HashValue>>>>,
    subscribers: TxnSubscribers,
}

impl TransactionSubscriptionActor {
    pub fn new(
        subscribers: TxnSubscribers,
        txn_receiver: mpsc::UnboundedReceiver<Arc<Vec<HashValue>>>,
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
        let rx = self.txn_receiver.take().unwrap();
        ctx.add_stream(rx);
    }
}
type TxnEvent = Arc<Vec<HashValue>>;
impl actix::StreamHandler<TxnEvent> for TransactionSubscriptionActor {
    fn handle(&mut self, item: TxnEvent, _ctx: &mut Self::Context) {
        for subscriber in self.subscribers.read().values() {
            notify::notify(
                subscriber,
                pubsub::Result::TransactionHash(item.as_ref().clone()),
            );
        }
    }
}
