// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use logger::prelude::*;
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct BusImpl {
    subscribers: HashMap<TypeId, Vec<Box<dyn Any + Send>>>,
}

impl BusImpl {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
        }
    }
    pub fn subscribe<M: 'static>(&mut self, recipient: Recipient<M>)
    where
        M: Message + Send + Clone,
        M::Result: Send,
    {
        let type_id = TypeId::of::<M>();
        let topic_subscribes = self.subscribers.entry(type_id).or_insert(vec![]);
        topic_subscribes.push(Box::new(recipient));
    }

    pub fn broadcast<M: 'static>(&self, msg: M)
    where
        M: Message + Send + Clone,
        M::Result: Send,
    {
        let type_id = &TypeId::of::<M>();
        self.subscribers.get(type_id).map(|topic_subscribes| {
            topic_subscribes.iter().for_each(|subscriber| {
                match subscriber.downcast_ref::<Recipient<M>>() {
                    Some(recipient) => {
                        //TODO smart clone.
                        match recipient.do_send(msg.clone()) {
                            Result::Ok(_) => {}
                            Result::Err(e) => {
                                //TODO retry.
                                error!("broadcast to {:?} error {:?}", recipient, e);
                            }
                        }
                    }
                    None => panic!("downcast_ref fail, should not happen."),
                }
            })
        });
    }
}
