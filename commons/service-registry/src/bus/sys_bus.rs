// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::EventNotifier;
use anyhow::Result;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::{mpsc, oneshot};
use log::{debug, error, info, warn};
use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::sync::mpsc::TrySendError;

enum SubscriptionRecord<M>
where
    M: Send + Clone + Debug,
{
    Notifier(EventNotifier<M>),
    Channel(mpsc::UnboundedSender<M>),
    Oneshot(Option<oneshot::Sender<M>>),
}

impl<M> SubscriptionRecord<M>
where
    M: Send + Clone + Debug,
{
    pub fn get_subscription_id(&self) -> String {
        match self {
            SubscriptionRecord::Channel(s) => format!("{:p}::{}::Channel", s, type_name::<M>()),
            SubscriptionRecord::Notifier(s) => format!(
                "{:p}::{}::Notifier({})",
                s,
                type_name::<M>(),
                s.target_service()
            ),
            SubscriptionRecord::Oneshot(s) => format!("{:p}::{}::Oneshot", s, type_name::<M>()),
        }
    }
}

impl<M> Debug for SubscriptionRecord<M>
where
    M: Send + Clone + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_subscription_id())
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
        debug!("do_subscribe: {:?}", subscription);
        topic_subscribes.push(Box::new(subscription));
    }

    pub fn subscribe<M>(&mut self, notifier: EventNotifier<M>)
    where
        M: Send + Clone + Debug + 'static,
    {
        self.do_subscribe(SubscriptionRecord::Notifier(notifier));
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

    // remove subscription when F return true.
    fn remove_subscription<M, F>(&mut self, f: F)
    where
        M: Send + Clone + Debug + 'static,
        F: Fn(&SubscriptionRecord<M>) -> bool,
    {
        let type_id = TypeId::of::<M>();
        if let Some(topic_subscriptions) = self.subscriptions.get_mut(&type_id) {
            topic_subscriptions.retain(move |subscription| {
                match subscription.downcast_ref::<SubscriptionRecord<M>>() {
                    Some(subscription) => !f(subscription),
                    _ => false,
                }
            });
        }
    }

    /// Only Notifier supported unsubscribe, channel and onshot just close receiver.
    pub fn unsubscribe<M>(&mut self, target_service: &str)
    where
        M: Send + Clone + Debug + 'static,
    {
        debug!("unsubscribe: {:?}", target_service);
        self.remove_subscription(|record: &SubscriptionRecord<M>| {
            if let SubscriptionRecord::Notifier(notifier) = record {
                if notifier.target_service() == target_service {
                    return true;
                }
            }
            false
        });
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
                            SubscriptionRecord::Notifier(notifier) => {
                                notifier.notify(msg.clone()).map_err(|e| match e {
                                    TrySendError::Full(m) => {
                                        warn!("Send message to notifier error: TrySendError::Full");
                                        (subscription.get_subscription_id(), m)
                                    }
                                    TrySendError::Disconnected(m) => {
                                        clear = true;
                                        warn!("Send message to notifier error TrySendError::Disconnected");
                                        (subscription.get_subscription_id(), m)
                                    }
                                })
                            }
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
        //TODO buffer SendError full message and retry.
        // clear used oneshot or closed subscription.
        if clear {
            self.remove_subscription(|record: &SubscriptionRecord<M>| {
                let result = match record {
                    SubscriptionRecord::Oneshot(sender) => sender.is_none(),
                    SubscriptionRecord::Channel(sender) => sender.is_closed(),
                    SubscriptionRecord::Notifier(notifier) => notifier.is_closed(),
                };
                if result {
                    info!("Clear subscription: {:?}", record);
                }
                result
            });
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
