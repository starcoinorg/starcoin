// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::service_actor::EventMessage;
use actix::Recipient;
use anyhow::Result;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::{mpsc, oneshot};
use log::{debug, error, warn};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::{self, Debug};

pub enum SubscriptionRecord<M>
where
    M: Send + Clone + Debug,
{
    Recipient(Recipient<EventMessage<M>>),
    Channel(mpsc::UnboundedSender<M>),
    Oneshot(Option<oneshot::Sender<M>>),
}

impl<M> SubscriptionRecord<M>
where
    M: Send + Clone + Debug,
{
    pub fn get_subscription_type(&self) -> &str {
        match self {
            SubscriptionRecord::Channel(_) => "channel",
            SubscriptionRecord::Recipient(_) => "recipient",
            SubscriptionRecord::Oneshot(_) => "oneshot",
        }
    }

    pub fn get_subscription_id(&self) -> String {
        match self {
            SubscriptionRecord::Channel(s) => format!("{:p}", s),
            SubscriptionRecord::Recipient(s) => format!("{:p}", s),
            SubscriptionRecord::Oneshot(s) => format!("{:p}", s),
        }
    }
}

impl<M> Debug for SubscriptionRecord<M>
where
    M: Send + Clone + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sub_id = self.get_subscription_id();
        let sub_type = self.get_subscription_type();
        let msg_type = std::any::type_name::<M>();
        write!(
            f,
            "Subscription by {}@{} for {}",
            sub_id, sub_type, msg_type
        )
    }
}

pub struct SysBus {
    subscriptions: HashMap<TypeId, Vec<Box<dyn Any + Send>>>,
}

impl Default for SysBus {
    fn default() -> Self {
        Self::new()
    }
}

impl SysBus {
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.subscriptions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.subscriptions.is_empty()
    }

    pub fn len_by_type<M>(&self) -> usize
    where
        M: Send + Clone + Debug + 'static,
    {
        let type_id = TypeId::of::<M>();
        self.subscriptions
            .get(&type_id)
            .map(|subs| subs.len())
            .unwrap_or(0)
    }

    fn do_subscribe<M>(&mut self, subscription: SubscriptionRecord<M>)
    where
        M: Send + Clone + Debug + 'static,
    {
        let type_id = TypeId::of::<M>();
        let topic_subscribes = self.subscriptions.entry(type_id).or_insert_with(Vec::new);
        debug!("{:?}", subscription);
        topic_subscribes.push(Box::new(subscription));
    }

    pub fn subscribe<M>(&mut self, recipient: Recipient<EventMessage<M>>)
    where
        M: Send + Clone + Debug + 'static,
    {
        self.do_subscribe(SubscriptionRecord::Recipient(recipient));
    }

    pub fn channel<M>(&mut self) -> UnboundedReceiver<M>
    where
        M: Send + Clone + Debug + 'static,
    {
        let (sender, receiver) = mpsc::unbounded();
        self.do_subscribe(SubscriptionRecord::Channel(sender));
        receiver
    }

    pub fn oneshot<M>(&mut self) -> oneshot::Receiver<M>
    where
        M: Send + Clone + Debug + 'static,
    {
        let (sender, receiver) = oneshot::channel();
        self.do_subscribe(SubscriptionRecord::Oneshot(Some(sender)));
        receiver
    }

    pub fn broadcast<M>(&mut self, msg: M)
    where
        M: Send + Clone + Debug + 'static,
    {
        debug!("Broadcast {:?}", msg);
        let type_id = &TypeId::of::<M>();
        let mut clear = false;
        if let Some(topic_subscriptions) = self.subscriptions.get_mut(type_id) {
            for subscription in topic_subscriptions {
                let result: Result<(), (String, M)> = match subscription
                    .downcast_mut::<SubscriptionRecord<M>>()
                {
                    Some(subscription) => {
                        debug!("send message to {:?}", subscription);
                        match subscription {
                            SubscriptionRecord::Recipient(recipient) => recipient
                                .do_send(EventMessage { msg: msg.clone() })
                                .map_err(|e| {
                                    clear = true;
                                    warn!("Send message to recipient error:{:?}", e);
                                    (subscription.get_subscription_id(), e.into_inner().msg)
                                }),
                            SubscriptionRecord::Channel(sender) => {
                                sender.unbounded_send(msg.clone()).map_err(|e| {
                                    clear = true;
                                    warn!("Send message to channel error:{:?}", e);
                                    (subscription.get_subscription_id(), e.into_inner())
                                })
                            }
                            SubscriptionRecord::Oneshot(sender) => {
                                let sender = sender.take();
                                clear = true;
                                match sender {
                                    Some(sender) => sender
                                        .send(msg.clone())
                                        .map_err(|e| (subscription.get_subscription_id(), e)),
                                    None => Err((subscription.get_subscription_id(), msg.clone())),
                                }
                            }
                        }
                    }
                    None => panic!("downcast_ref fail, should not happen."),
                };
                if let Err((id, msg)) = result {
                    error!("Send message {:?} to {:?} fail.", msg, id);
                }
            }
        }
        // clear used oneshot subscription.
        if clear {
            if let Some(topic_subscriptions) = self.subscriptions.get_mut(type_id) {
                topic_subscriptions.retain(|subscription| -> bool {
                    let result = match subscription.downcast_ref::<SubscriptionRecord<M>>() {
                        Some(SubscriptionRecord::Oneshot(sender)) => sender.is_some(),
                        Some(SubscriptionRecord::Channel(sender)) => !sender.is_closed(),
                        Some(SubscriptionRecord::Recipient(recipient)) => recipient.connected(),
                        _ => true,
                    };
                    if !result {
                        debug!(
                            "Clear subscription: {:?}",
                            subscription.downcast_ref::<SubscriptionRecord<M>>()
                        );
                    }
                    result
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task;
    use futures_timer::Delay;
    use std::time::Duration;

    #[derive(Debug, Clone)]
    struct Message {}

    #[stest::test]
    async fn test_clear() {
        let mut bus = SysBus::new();
        let receiver = bus.oneshot::<Message>();
        assert_eq!(1, bus.len_by_type::<Message>());
        let job = task::spawn(async { receiver.await });
        Delay::new(Duration::from_millis(10)).await;
        bus.broadcast(Message {});
        let result = job.await;
        assert!(result.is_ok());
        assert_eq!(0, bus.len_by_type::<Message>());
    }
}
