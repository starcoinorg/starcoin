use actix::{Actor, Addr};
use jsonrpc_pubsub::typed::{Sink, Subscriber};
use jsonrpc_pubsub::SubscriptionId;
use starcoin_logger::prelude::*;
use std::collections::HashMap;
use std::sync::{atomic, Arc};

use super::notify::{Notification, SubscriberNotifyActor};

pub struct Subscribers<T> {
    subscriber_id: Arc<atomic::AtomicU64>,
    subscriptions: HashMap<SubscriptionId, T>,
}

impl<T> Subscribers<T> {
    pub fn new(subscriber_id: Arc<atomic::AtomicU64>) -> Self {
        Self {
            subscriber_id,
            subscriptions: HashMap::default(),
        }
    }

    fn next_id(&self) -> SubscriptionId {
        let id = self.subscriber_id.fetch_add(1, atomic::Ordering::SeqCst);
        SubscriptionId::Number(id)
    }

    // /// Insert new subscription and return assigned id.
    // pub fn insert(&mut self, val: T) -> SubscriptionId {
    //     let id = self.next_id();
    //     debug!(target: "pubsub", "Adding subscription id={:?}", &id);
    //     self.subscriptions.insert(id.clone(), val);
    //     id
    // }

    /// Removes subscription with given id and returns it (if any).
    pub fn remove(&mut self, id: &SubscriptionId) -> Option<T> {
        trace!(target: "pubsub", "Removing subscription id={:?}", id);
        self.subscriptions.remove(id)
    }
}

impl<T> Subscribers<Addr<SubscriberNotifyActor<T>>>
where
    T: Unpin + 'static + Send,
{
    /// Assigns id and adds a subscriber to the list.
    pub fn add(&mut self, spawner: &actix_rt::Arbiter, sub: Subscriber<T>) {
        let id = self.next_id();
        if let Ok(sink) = sub.assign_id(id.clone()) {
            debug!(target: "pubsub", "Adding subscription id={:?}", &id);
            let addr =
                actix::Actor::start_in_arbiter(spawner, |_| SubscriberNotifyActor::new(sink));
            self.subscriptions.insert(id, addr);
        }
    }
}

impl<T, V> Subscribers<(Addr<SubscriberNotifyActor<T>>, V)>
where
    T: Unpin + 'static + Send,
{
    /// Assigns id and adds a subscriber to the list.
    pub fn add(&mut self, spawner: &actix_rt::Arbiter, sub: Subscriber<T>, val: V) {
        let id = self.next_id();
        if let Ok(sink) = sub.assign_id(id.clone()) {
            debug!(target: "pubsub", "Adding subscription id={:?}", &id);
            let addr =
                actix::Actor::start_in_arbiter(spawner, |_| SubscriberNotifyActor::new(sink));
            self.subscriptions.insert(id, (addr, val));
        }
    }
}

impl<T> std::ops::Deref for Subscribers<T> {
    type Target = HashMap<SubscriptionId, T>;

    fn deref(&self) -> &Self::Target {
        &self.subscriptions
    }
}
