// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::Result;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::{mpsc, oneshot};
use starcoin_logger::prelude::*;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::{self, Debug};

pub enum SubscriptionRecord<M>
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    Recipient(Recipient<M>),
    Channel(mpsc::UnboundedSender<M>),
    Oneshot(Option<oneshot::Sender<M>>),
}

impl<M> SubscriptionRecord<M>
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    pub fn get_subscription_type(&self) -> &str {
        match self {
            SubscriptionRecord::Channel(_) => "channel",
            SubscriptionRecord::Recipient(_) => "recipient",
            SubscriptionRecord::Oneshot(_) => "oneshot",
        }
    }
}

impl<M> Debug for SubscriptionRecord<M>
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sub_type = self.get_subscription_type();
        let msg_type = std::any::type_name::<M>();
        write!(f, "Subscription by {} for {}", sub_type, msg_type)
    }
}

pub struct BusImpl {
    subscriptions: HashMap<TypeId, Vec<Box<dyn Any + Send>>>,
}

impl BusImpl {
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
        }
    }

    fn do_subscribe<M: 'static>(&mut self, subscription: SubscriptionRecord<M>)
    where
        M: Message + Send + Clone + Debug,
        M::Result: Send,
    {
        let type_id = TypeId::of::<M>();
        let topic_subscribes = self.subscriptions.entry(type_id).or_insert(vec![]);
        info!("{:?}", subscription);
        topic_subscribes.push(Box::new(subscription));
    }

    pub fn subscribe<M: 'static>(&mut self, recipient: Recipient<M>)
    where
        M: Message + Send + Clone + Debug,
        M::Result: Send,
    {
        self.do_subscribe(SubscriptionRecord::Recipient(recipient));
    }

    pub fn channel<M: 'static>(&mut self) -> UnboundedReceiver<M>
    where
        M: Message + Send + Clone + Debug,
        M::Result: Send,
    {
        let (sender, receiver) = mpsc::unbounded();
        self.do_subscribe(SubscriptionRecord::Channel(sender));
        return receiver;
    }

    pub fn oneshot<M: 'static>(&mut self) -> oneshot::Receiver<M>
    where
        M: Message + Send + Clone + Debug,
        M::Result: Send,
    {
        let (sender, receiver) = oneshot::channel();
        self.do_subscribe(SubscriptionRecord::Oneshot(Some(sender)));
        return receiver;
    }

    pub fn broadcast<M: 'static>(&mut self, msg: M)
    where
        M: Message + Send + Clone + Debug,
        M::Result: Send,
    {
        debug!("Broadcast {:?}", msg);
        let type_id = &TypeId::of::<M>();
        let mut clear = false;
        self.subscriptions
            .get_mut(type_id)
            .map(|topic_subscriptions| {
                for subscription in topic_subscriptions {
                    let result: Result<(), M> =
                        match subscription.downcast_mut::<SubscriptionRecord<M>>() {
                            Some(subscription) => {
                                debug!("send message to {:?}", subscription);
                                match subscription {
                                    SubscriptionRecord::Recipient(recipient) => {
                                        //TODO smart clone.
                                        recipient.do_send(msg.clone()).map_err(|e| {
                                            clear = true;
                                            warn!("Send message to recipient error:{:?}", e);
                                            e.into_inner()
                                        })
                                    }
                                    SubscriptionRecord::Channel(sender) => {
                                        sender.unbounded_send(msg.clone()).map_err(|e| {
                                            clear = true;
                                            warn!("Send message to channel error:{:?}", e);
                                            e.into_inner()
                                        })
                                    }
                                    SubscriptionRecord::Oneshot(sender) => {
                                        let sender = std::mem::replace(sender, None);
                                        clear = true;
                                        match sender {
                                            Some(sender) => sender.send(msg.clone()),
                                            None => Err(msg.clone()),
                                        }
                                    }
                                }
                            }
                            None => panic!("downcast_ref fail, should not happen."),
                        };
                    if let Err(e) = result {
                        error!("Send message {:?} fail.", e);
                    }
                }
            });
        // clear used oneshot subscription.
        if clear {
            self.subscriptions
                .get_mut(type_id)
                .map(|topic_subscriptions| {
                    topic_subscriptions.retain(|subscription| -> bool {
                        let result = match subscription.downcast_ref::<SubscriptionRecord<M>>() {
                            Some(SubscriptionRecord::Oneshot(sender)) => sender.is_some(),
                            Some(SubscriptionRecord::Channel(sender)) => !sender.is_closed(),
                            Some(SubscriptionRecord::Recipient(recipient)) => recipient.connected(),
                            _ => true,
                        };
                        if result {
                            debug!(
                                "Clear subscription: {:?}",
                                subscription.downcast_ref::<SubscriptionRecord<M>>()
                            );
                        }
                        result
                    });
                });
        }
    }
}
