use crate::errors;
use crate::metadata::Metadata;
use crate::module::pubsub::notify::SubscriberNotifyActor;

use actix::{Addr};
use futures_channel::mpsc;
use jsonrpc_core::Result;
use jsonrpc_pubsub::typed::{Subscriber};
use jsonrpc_pubsub::SubscriptionId;
use parking_lot::RwLock;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::{pubsub::StarcoinPubSub, types::pubsub};

use std::sync::{atomic, Arc};

mod notify;
mod subscribers;
mod txn_subscription_actor;

use starcoin_txpool_api::TxPoolAsyncService;
use subscribers::Subscribers;
use txn_subscription_actor::TransactionSubscriptionActor;

type ClientNotifier = Addr<SubscriberNotifyActor<pubsub::Result>>;
type TxnSubscribers = Arc<RwLock<Subscribers<ClientNotifier>>>;

pub struct PubSubImpl {
    subscriber_id: Arc<atomic::AtomicU64>,
    spawner: actix_rt::Arbiter,
    transactions_subscribers: TxnSubscribers,
}

impl PubSubImpl {
    pub fn new() -> Self {
        let subscriber_id = Arc::new(atomic::AtomicU64::new(0));
        let transactions_subscribers =
            Arc::new(RwLock::new(Subscribers::new(subscriber_id.clone())));
        Self {
            spawner: actix_rt::Arbiter::new(),
            subscriber_id: subscriber_id.clone(),
            transactions_subscribers,
        }
    }

    pub fn start_transaction_subscription_handler<P>(
        &self,
        _tx_pool: P,
        txn_receiver: mpsc::UnboundedReceiver<Arc<Vec<HashValue>>>,
    ) where
        P: TxPoolAsyncService + 'static,
    {
        let actor =
            TransactionSubscriptionActor::new(self.transactions_subscribers.clone(), txn_receiver);

        actix::Actor::start_in_arbiter(&self.spawner, |_ctx| actor);
    }
}

impl StarcoinPubSub for PubSubImpl {
    type Metadata = Metadata;
    fn subscribe(
        &self,
        _meta: Metadata,
        subscriber: Subscriber<pubsub::Result>,
        kind: pubsub::Kind,
        params: Option<pubsub::Params>,
    ) {
        let error = match (kind, params) {
            (pubsub::Kind::NewHeads, None) => {
                // TODO: implement me.
                return;
            }
            (pubsub::Kind::NewHeads, _) => {
                errors::invalid_params("newHeads", "Expected no parameters.")
            }
            (pubsub::Kind::NewPendingTransactions, None) => {
                self.transactions_subscribers
                    .write()
                    .add(&self.spawner, subscriber);
                return;
            }
            (pubsub::Kind::NewPendingTransactions, _) => {
                errors::invalid_params("newPendingTransactions", "Expected no parameters.")
            } // _ => errors::unimplemented(None),
        };

        let _ = subscriber.reject(error);
    }

    fn unsubscribe(&self, _: Option<Self::Metadata>, id: SubscriptionId) -> Result<bool> {
        let res2 = self.transactions_subscribers.write().remove(&id).is_some();
        Ok(res2)
    }
}
