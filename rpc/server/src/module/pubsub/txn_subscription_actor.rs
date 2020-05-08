use super::notify;
use super::pubsub;

use super::TxnSubscribers;

use actix;
use actix::{ActorContext, ActorFuture, AsyncContext, ContextFutureSpawner, WrapFuture};

use starcoin_crypto::HashValue;
use starcoin_txpool_api::TxPoolAsyncService;
use starcoin_types::transaction::TxStatus;
use std::sync::Arc;

pub struct TransactionSubscriptionActor<P> {
    txpool: P,
    subscribers: TxnSubscribers,
}

impl<P> TransactionSubscriptionActor<P> {
    pub fn new(subscribers: TxnSubscribers, txpool: P) -> Self {
        Self {
            subscribers,
            txpool,
        }
    }
}

impl<P> actix::Actor for TransactionSubscriptionActor<P>
where
    P: TxPoolAsyncService + 'static,
{
    type Context = actix::Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.txpool
            .clone()
            .subscribe_txns()
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(r) => {
                        ctx.add_stream(r);
                    }
                    Err(_e) => {
                        ctx.terminate();
                    }
                };
                async {}.into_actor(act)
            })
            .wait(ctx);
    }
}
type TxnEvent = Arc<Vec<(HashValue, TxStatus)>>;
impl<P> actix::StreamHandler<TxnEvent> for TransactionSubscriptionActor<P>
where
    P: TxPoolAsyncService + 'static,
{
    fn handle(&mut self, item: TxnEvent, _ctx: &mut Self::Context) {
        let hs = item.as_ref().iter().map(|(h, _)| *h).collect::<Vec<_>>();
        for subscriber in self.subscribers.read().values() {
            notify::notify(subscriber, pubsub::Result::TransactionHash(hs.clone()));
        }
    }
}
