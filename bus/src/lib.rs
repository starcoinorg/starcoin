// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::bus::SysBus;
use actix::prelude::*;
use anyhow::Result;
use futures::{
    channel::{mpsc, oneshot},
    FutureExt,
};
use std::fmt::Debug;
use std::marker::PhantomData;

pub mod bus;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Subscription<M: 'static>
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    pub recipient: Recipient<M>,
}

#[derive(Debug, Default)]
pub struct Channel<M: 'static>
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    m: PhantomData<M>,
}

impl<M> Channel<M>
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    pub fn new() -> Self {
        Self {
            m: Default::default(),
        }
    }
}

impl<M> Message for Channel<M>
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    type Result = Result<mpsc::UnboundedReceiver<M>>;
}

#[derive(Debug, Default)]
pub struct Oneshot<M: 'static>
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    m: PhantomData<M>,
}

impl<M> Oneshot<M>
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    pub fn new() -> Self {
        Self {
            m: Default::default(),
        }
    }
}

impl<M> Message for Oneshot<M>
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    type Result = Result<oneshot::Receiver<M>>;
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Broadcast<M: 'static>
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    pub msg: M,
}

#[async_trait::async_trait]
pub trait Bus {
    async fn subscribe<M: 'static>(self, recipient: Recipient<M>) -> Result<()>
    where
        M: Message + Send + Clone + Debug,
        M::Result: Send;

    async fn channel<M: 'static>(self) -> Result<mpsc::UnboundedReceiver<M>>
    where
        M: Message + Send + Clone + Debug,
        M::Result: Send;

    async fn oneshot<M: 'static>(self) -> Result<M>
    where
        M: Message + Send + Clone + Debug,
        M::Result: Send;

    async fn broadcast<M: 'static>(self, msg: M) -> Result<()>
    where
        M: Message + Send + Clone + Debug,
        M::Result: Send;
}

pub struct BusActor {
    bus: SysBus,
}

impl BusActor {
    pub fn launch() -> Addr<BusActor> {
        let bus = BusActor { bus: SysBus::new() };
        bus.start()
    }
}

impl Actor for BusActor {
    type Context = Context<Self>;
}

impl<M: 'static> Handler<Subscription<M>> for BusActor
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    type Result = ();

    fn handle(&mut self, msg: Subscription<M>, _ctx: &mut Self::Context) -> Self::Result {
        self.bus.subscribe(msg.recipient)
    }
}

impl<M: 'static> Handler<Channel<M>> for BusActor
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    type Result = Result<mpsc::UnboundedReceiver<M>>;

    fn handle(&mut self, _msg: Channel<M>, _ctx: &mut Self::Context) -> Self::Result {
        Ok(self.bus.channel())
    }
}

impl<M: 'static> Handler<Oneshot<M>> for BusActor
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    type Result = Result<oneshot::Receiver<M>>;

    fn handle(&mut self, _msg: Oneshot<M>, _ctx: &mut Self::Context) -> Self::Result {
        Ok(self.bus.oneshot())
    }
}

impl<M: 'static> Handler<Broadcast<M>> for BusActor
where
    M: Message + Send + Clone + Debug,
    M::Result: Send,
{
    type Result = ();

    fn handle(&mut self, msg: Broadcast<M>, _ctx: &mut Self::Context) -> Self::Result {
        self.bus.broadcast(msg.msg)
    }
}

#[async_trait::async_trait]
impl Bus for Addr<BusActor> {
    async fn subscribe<M: 'static>(self, recipient: Recipient<M>) -> Result<()>
    where
        M: Message + Send + Clone + Debug,
        M::Result: Send,
    {
        self.send(Subscription { recipient })
            .await
            .map_err(|e| e.into())
    }

    async fn channel<M: 'static>(self) -> Result<mpsc::UnboundedReceiver<M>>
    where
        M: Message + Send + Clone + Debug,
        M::Result: Send,
    {
        self.send(Channel::<M>::new())
            .await
            .map_err(Into::<anyhow::Error>::into)?
    }

    async fn oneshot<M: 'static>(self) -> Result<M>
    where
        M: Message + Send + Clone + Debug,
        M::Result: Send,
    {
        self.send(Oneshot::<M>::new())
            .then(|result| async {
                match result {
                    Ok(receiver) => receiver?.await.map_err(Into::<anyhow::Error>::into),
                    Err(err) => Err(Into::<anyhow::Error>::into(err)),
                }
            })
            .await
    }

    async fn broadcast<M: 'static>(self, msg: M) -> Result<()>
    where
        M: Message + Send + Clone + Debug,
        M::Result: Send,
    {
        self.send(Broadcast { msg }).await.map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix::clock::delay_for;
    use futures::executor::block_on;
    use futures::StreamExt;
    use starcoin_logger::prelude::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[derive(Debug, Message, Clone)]
    #[rtype(result = "()")]
    struct MyMessage {}

    #[derive(Debug, Message, Clone)]
    #[rtype(result = "u64")]
    struct GetCounterMessage {}

    #[derive(Debug, Message, Clone)]
    #[rtype(result = "()")]
    struct DoBroadcast {}

    #[derive(Debug, Message, Clone)]
    #[rtype(result = "Result<()>")]
    struct DoBroadcast2 {}

    struct MyActor {
        counter: u64,
        bus: Addr<BusActor>,
    }

    impl Actor for MyActor {
        type Context = Context<Self>;
    }

    impl Handler<MyMessage> for MyActor {
        type Result = ();

        fn handle(&mut self, msg: MyMessage, _ctx: &mut Self::Context) {
            info!("handle MyMessage: {:?}", msg);
            self.counter += 1;
        }
    }

    impl Handler<GetCounterMessage> for MyActor {
        type Result = u64;

        fn handle(&mut self, _msg: GetCounterMessage, _ctx: &mut Self::Context) -> Self::Result {
            info!("handle GetCounterMessage: {:?}", self.counter);
            self.counter
        }
    }

    impl Handler<DoBroadcast> for MyActor {
        type Result = ();

        fn handle(&mut self, _msg: DoBroadcast, ctx: &mut Self::Context) {
            info!("handle DoBroadcast");
            self.bus
                .send(Broadcast { msg: MyMessage {} })
                .into_actor(self)
                //need convert act to static ActorFuture and call wait.
                .then(|_result, act, _ctx| async {}.into_actor(act))
                .wait(ctx);
        }
    }

    impl Handler<DoBroadcast2> for MyActor {
        type Result = ResponseActFuture<Self, Result<()>>;

        fn handle(&mut self, _msg: DoBroadcast2, _ctx: &mut Self::Context) -> Self::Result {
            let f = self.bus.clone().broadcast(MyMessage {});
            let f = actix::fut::wrap_future::<_, Self>(f);
            Box::new(f)
        }
    }

    #[stest::test]
    async fn test_bus_actor() {
        let bus_actor = BusActor::launch();
        let actor = MyActor {
            counter: 0,
            bus: bus_actor.clone(),
        };
        let addr = actor.start();
        let recipient = addr.clone().recipient::<MyMessage>();

        bus_actor.send(Subscription { recipient }).await.unwrap();
        bus_actor
            .send(Broadcast { msg: MyMessage {} })
            .await
            .unwrap();
        delay_for(Duration::from_millis(100)).await;
        let counter = addr.send(GetCounterMessage {}).await.unwrap();
        assert_eq!(counter, 1);
    }

    #[stest::test]
    async fn test_bus_actor_send_message_in_handle() {
        let bus_actor = BusActor::launch();
        let actor = MyActor {
            counter: 0,
            bus: bus_actor.clone(),
        };
        let addr = actor.start();
        let recipient = addr.clone().recipient::<MyMessage>();

        bus_actor.send(Subscription { recipient }).await.unwrap();
        addr.send(DoBroadcast {}).await.unwrap();
        delay_for(Duration::from_millis(100)).await;
        let counter = addr.send(GetCounterMessage {}).await.unwrap();
        assert_eq!(counter, 1);
    }

    #[stest::test]
    async fn test_bus_actor_async_trait() {
        let bus_actor = BusActor::launch();
        let actor = MyActor {
            counter: 0,
            bus: bus_actor.clone(),
        };
        let addr = actor.start();
        let recipient = addr.clone().recipient::<MyMessage>();

        bus_actor.subscribe(recipient).await.unwrap();
        addr.send(DoBroadcast2 {}).await.unwrap().unwrap();
        delay_for(Duration::from_millis(100)).await;
        let counter = addr.send(GetCounterMessage {}).await.unwrap();
        assert_eq!(counter, 1);
    }

    #[stest::test]
    async fn test_onshot() {
        let bus_actor = BusActor::launch();
        let bus_actor2 = bus_actor.clone();
        let arbiter = Arbiter::new();
        arbiter.exec_fn(move || loop {
            let result =
                block_on(async { bus_actor2.clone().broadcast(MyMessage {}).await.is_ok() });
            debug!("broadcast result: {}", result);
            sleep(Duration::from_millis(50));
        });
        let msg = bus_actor.clone().oneshot::<MyMessage>().await;
        assert!(msg.is_ok());
        let msg = bus_actor.clone().oneshot::<MyMessage>().await;
        assert!(msg.is_ok());
    }

    #[stest::test]
    async fn test_channel() {
        let bus_actor = BusActor::launch();
        let bus_actor2 = bus_actor.clone();
        let arbiter = Arbiter::new();
        arbiter.exec_fn(move || loop {
            let result =
                block_on(async { bus_actor2.clone().broadcast(MyMessage {}).await.is_ok() });
            debug!("broadcast result: {}", result);
            sleep(Duration::from_millis(50));
        });
        let result = bus_actor.clone().channel::<MyMessage>().await;
        assert!(result.is_ok());
        let receiver = result.unwrap();
        let msgs: Vec<MyMessage> = receiver.take(3).collect().await;
        assert_eq!(3, msgs.len());

        let receiver2 = bus_actor.clone().channel::<MyMessage>().await.unwrap();
        let msgs: Vec<MyMessage> = receiver2.take(3).collect().await;
        assert_eq!(3, msgs.len());
    }
}
