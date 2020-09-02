// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::service_actor::EventMessage;
use crate::{ServiceRef, ServiceRequest};
use actix::prelude::*;
use anyhow::Result;
use futures::channel::{mpsc, oneshot};
use std::fmt::Debug;
use std::marker::PhantomData;

mod service;
mod sys_bus;

pub use service::BusService;
pub use sys_bus::SysBus;

#[derive(Clone, Debug)]
pub struct Subscription<M>
where
    M: Send + Clone + Debug + 'static,
{
    pub recipient: Recipient<EventMessage<M>>,
}

impl<M> ServiceRequest for Subscription<M>
where
    M: Send + Clone + Debug,
{
    type Response = ();
}

#[derive(Debug, Default)]
pub struct Channel<M>
where
    M: Send + Clone + Debug + 'static,
{
    m: PhantomData<M>,
}

impl<M> Channel<M>
where
    M: Send + Clone + Debug,
{
    pub fn new() -> Self {
        Self {
            m: Default::default(),
        }
    }
}

impl<M> ServiceRequest for Channel<M>
where
    M: Send + Clone + Debug,
{
    type Response = Result<mpsc::UnboundedReceiver<M>>;
}

#[derive(Debug, Default)]
pub struct Oneshot<M>
where
    M: Send + Clone + Debug + 'static,
{
    m: PhantomData<M>,
}

impl<M> Oneshot<M>
where
    M: Send + Clone + Debug,
{
    pub fn new() -> Self {
        Self {
            m: Default::default(),
        }
    }
}

impl<M> ServiceRequest for Oneshot<M>
where
    M: Send + Clone + Debug,
{
    type Response = Result<oneshot::Receiver<M>>;
}

#[derive(Debug, Clone)]
pub struct Broadcast<M>
where
    M: Send + Clone + Debug + 'static,
{
    pub msg: M,
}

impl<M> Broadcast<M>
where
    M: Send + Clone + Debug,
{
    pub fn new(msg: M) -> Self {
        Self { msg }
    }
}

impl<M> ServiceRequest for Broadcast<M>
where
    M: Send + Clone + Debug,
{
    type Response = ();
}

#[async_trait::async_trait]
pub trait Bus {
    async fn subscribe<M>(&self, recipient: Recipient<EventMessage<M>>) -> Result<()>
    where
        M: Send + Clone + Debug + 'static;

    async fn channel<M>(&self) -> Result<mpsc::UnboundedReceiver<M>>
    where
        M: Send + Clone + Debug + 'static;

    async fn oneshot<M>(&self) -> Result<oneshot::Receiver<M>>
    where
        M: Send + Clone + Debug + 'static;

    async fn broadcast<M: 'static>(&self, msg: M) -> Result<()>
    where
        M: Send + Clone + Debug;
}

#[async_trait::async_trait]
impl Bus for ServiceRef<BusService> {
    async fn subscribe<M>(&self, recipient: Recipient<EventMessage<M>>) -> Result<()>
    where
        M: Send + Clone + Debug + 'static,
    {
        self.send(Subscription { recipient })
            .await
            .map_err(Into::<anyhow::Error>::into)
    }

    async fn channel<M>(&self) -> Result<mpsc::UnboundedReceiver<M>>
    where
        M: Send + Clone + Debug + 'static,
    {
        self.send(Channel::<M>::new())
            .await
            .map_err(Into::<anyhow::Error>::into)?
    }

    async fn oneshot<M>(&self) -> Result<oneshot::Receiver<M>>
    where
        M: Send + Clone + Debug + 'static,
    {
        self.send(Oneshot::<M>::new())
            .await
            .map_err(Into::<anyhow::Error>::into)?
    }

    async fn broadcast<M>(&self, msg: M) -> Result<()>
    where
        M: Send + Clone + Debug + 'static,
    {
        self.send(Broadcast { msg })
            .await
            .map_err(Into::<anyhow::Error>::into)
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use actix::clock::delay_for;
    // use futures::executor::block_on;
    // use futures::StreamExt;
    // use starcoin_logger::prelude::*;
    // use std::thread::sleep;
    // use std::time::Duration;
    //
    // #[derive(Debug, Message, Clone)]
    // #[rtype(result = "()")]
    // struct MyMessage {}
    //
    // #[derive(Debug, Message, Clone)]
    // #[rtype(result = "u64")]
    // struct GetCounterMessage {}
    //
    // #[derive(Debug, Message, Clone)]
    // #[rtype(result = "()")]
    // struct DoBroadcast {}
    //
    // #[derive(Debug, Message, Clone)]
    // #[rtype(result = "Result<()>")]
    // struct DoBroadcast2 {}
    //
    // struct MyActor {
    //     counter: u64,
    //     bus: Addr<BusActor>,
    // }
    //
    // impl Actor for MyActor {
    //     type Context = Context<Self>;
    // }
    //
    // impl Handler<MyMessage> for MyActor {
    //     type Result = ();
    //
    //     fn handle(&mut self, msg: MyMessage, _ctx: &mut Self::Context) {
    //         info!("handle MyMessage: {:?}", msg);
    //         self.counter += 1;
    //     }
    // }
    //
    // impl Handler<GetCounterMessage> for MyActor {
    //     type Result = u64;
    //
    //     fn handle(&mut self, _msg: GetCounterMessage, _ctx: &mut Self::Context) -> Self::Result {
    //         info!("handle GetCounterMessage: {:?}", self.counter);
    //         self.counter
    //     }
    // }
    //
    // impl Handler<DoBroadcast> for MyActor {
    //     type Result = ();
    //
    //     fn handle(&mut self, _msg: DoBroadcast, ctx: &mut Self::Context) {
    //         info!("handle DoBroadcast");
    //         self.bus
    //             .send(Broadcast { msg: MyMessage {} })
    //             .into_actor(self)
    //             //need convert act to static ActorFuture and call wait.
    //             .then(|_result, act, _ctx| async {}.into_actor(act))
    //             .wait(ctx);
    //     }
    // }
    //
    // impl Handler<DoBroadcast2> for MyActor {
    //     type Result = ResponseActFuture<Self, Result<()>>;
    //
    //     fn handle(&mut self, _msg: DoBroadcast2, _ctx: &mut Self::Context) -> Self::Result {
    //         let f = self.bus.clone().broadcast(MyMessage {});
    //         let f = actix::fut::wrap_future::<_, Self>(f);
    //         Box::pin(f)
    //     }
    // }
    //
    // #[stest::test]
    // async fn test_bus_actor() {
    //     let bus_actor = BusActor::launch();
    //
    //     let actor = MyActor {
    //         counter: 0,
    //         bus: bus_actor.clone(),
    //     };
    //     let addr = actor.start();
    //     let recipient = addr.clone().recipient::<MyMessage>();
    //
    //     bus_actor.send(Subscription { recipient }).await.unwrap();
    //     bus_actor
    //         .send(Broadcast { msg: MyMessage {} })
    //         .await
    //         .unwrap();
    //     delay_for(Duration::from_millis(100)).await;
    //     let counter = addr.send(GetCounterMessage {}).await.unwrap();
    //     assert_eq!(counter, 1);
    // }
    //
    // #[stest::test]
    // async fn test_bus_actor_send_message_in_handle() {
    //     let bus_actor = BusActor::launch();
    //     let actor = MyActor {
    //         counter: 0,
    //         bus: bus_actor.clone(),
    //     };
    //     let addr = actor.start();
    //     let recipient = addr.clone().recipient::<MyMessage>();
    //
    //     bus_actor.send(Subscription { recipient }).await.unwrap();
    //     addr.send(DoBroadcast {}).await.unwrap();
    //     delay_for(Duration::from_millis(100)).await;
    //     let counter = addr.send(GetCounterMessage {}).await.unwrap();
    //     assert_eq!(counter, 1);
    // }
    //
    // #[stest::test]
    // async fn test_bus_actor_async_trait() {
    //     let bus_actor = BusActor::launch();
    //     let actor = MyActor {
    //         counter: 0,
    //         bus: bus_actor.clone(),
    //     };
    //     let addr = actor.start();
    //     let recipient = addr.clone().recipient::<MyMessage>();
    //
    //     bus_actor.subscribe(recipient).await.unwrap();
    //     addr.send(DoBroadcast2 {}).await.unwrap().unwrap();
    //     delay_for(Duration::from_millis(100)).await;
    //     let counter = addr.send(GetCounterMessage {}).await.unwrap();
    //     assert_eq!(counter, 1);
    // }
    //
    // #[stest::test]
    // async fn test_onshot() {
    //     let bus_actor = BusActor::launch();
    //     let bus_actor2 = bus_actor.clone();
    //     let arbiter = Arbiter::new();
    //     arbiter.exec_fn(move || loop {
    //         let result =
    //             block_on(async { bus_actor2.clone().broadcast(MyMessage {}).await.is_ok() });
    //         debug!("broadcast result: {}", result);
    //         sleep(Duration::from_millis(50));
    //     });
    //     let msg = bus_actor
    //         .clone()
    //         .oneshot::<MyMessage>()
    //         .await
    //         .unwrap()
    //         .await;
    //     assert!(msg.is_ok());
    //     let msg = bus_actor
    //         .clone()
    //         .oneshot::<MyMessage>()
    //         .await
    //         .unwrap()
    //         .await;
    //     assert!(msg.is_ok());
    // }
    //
    // #[stest::test]
    // async fn test_channel() {
    //     let bus_actor = BusActor::launch();
    //     let bus_actor2 = bus_actor.clone();
    //     let arbiter = Arbiter::new();
    //     arbiter.exec_fn(move || loop {
    //         let result =
    //             block_on(async { bus_actor2.clone().broadcast(MyMessage {}).await.is_ok() });
    //         debug!("broadcast result: {}", result);
    //         sleep(Duration::from_millis(50));
    //     });
    //     let result = bus_actor.clone().channel::<MyMessage>().await;
    //     assert!(result.is_ok());
    //     let receiver = result.unwrap();
    //     let msgs: Vec<MyMessage> = receiver.take(3).collect().await;
    //     assert_eq!(3, msgs.len());
    //
    //     let receiver2 = bus_actor.clone().channel::<MyMessage>().await.unwrap();
    //     let msgs: Vec<MyMessage> = receiver2.take(3).collect().await;
    //     assert_eq!(3, msgs.len());
    // }
}
