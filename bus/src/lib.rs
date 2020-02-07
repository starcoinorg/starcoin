// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::bus::Bus;
use actix::prelude::*;

mod bus;

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Subscription<M: 'static>
where
    M: Message + Send + Clone,
    M::Result: Send,
{
    recipient: Recipient<M>,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Broadcast<M: 'static>
where
    M: Message + Send + Clone,
    M::Result: Send,
{
    message: M,
}

pub struct BusActor {
    bus: Bus,
}

impl BusActor {
    pub fn launch() -> Addr<BusActor> {
        let bus = BusActor { bus: Bus::new() };
        bus.start()
    }
}

impl Actor for BusActor {
    type Context = Context<BusActor>;
}

impl<M: 'static> Handler<Subscription<M>> for BusActor
where
    M: Message + Send + Clone,
    M::Result: Send,
{
    type Result = ();

    fn handle(&mut self, msg: Subscription<M>, _ctx: &mut Self::Context) -> Self::Result {
        self.bus.subscribe(msg.recipient)
    }
}

impl<M: 'static> Handler<Broadcast<M>> for BusActor
where
    M: Message + Send + Clone,
    M::Result: Send,
{
    type Result = ();

    fn handle(&mut self, msg: Broadcast<M>, _ctx: &mut Self::Context) -> Self::Result {
        self.bus.broadcast(msg.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{delay_for, Duration};

    #[derive(Debug, Message, Clone)]
    #[rtype(result = "()")]
    struct MyMessage {}

    #[derive(Debug, Message, Clone)]
    #[rtype(result = "u64")]
    struct GetCounterMessage {}

    struct MyActor {
        counter: u64,
    }

    impl Actor for MyActor {
        type Context = Context<Self>;
    }

    impl Handler<MyMessage> for MyActor {
        type Result = ();

        fn handle(&mut self, msg: MyMessage, _ctx: &mut Self::Context) -> Self::Result {
            println!("handle msg: {:?}", msg);
            self.counter += 1;
        }
    }

    impl Handler<GetCounterMessage> for MyActor {
        type Result = u64;

        fn handle(&mut self, _msg: GetCounterMessage, _ctx: &mut Self::Context) -> Self::Result {
            self.counter
        }
    }

    #[actix_rt::test]
    async fn test_bus_actor() {
        let bus_actor = BusActor::launch();
        let actor = MyActor { counter: 0 };
        let addr = actor.start();
        let recipient = addr.clone().recipient::<MyMessage>();

        bus_actor.send(Subscription { recipient }).await.unwrap();
        bus_actor
            .send(Broadcast {
                message: MyMessage {},
            })
            .await
            .unwrap();
        delay_for(Duration::from_millis(100)).await;
        let counter = addr.send(GetCounterMessage {}).await.unwrap();
        assert_eq!(counter, 1);
    }
}
