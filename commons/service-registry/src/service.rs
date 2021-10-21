// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::bus::{Bus, BusService};
use crate::service_actor::{EventMessage, ServiceActor};
use crate::service_cache::ServiceCache;
use crate::{RegistryAsyncService, RegistryService};
use crate::{ServiceRef, ServiceRequest};
use actix::fut::{wrap_future, IntoActorFuture};
use actix::{ActorContext, ActorFuture, AsyncContext, Context};
use anyhow::{format_err, Result};
use futures::channel::oneshot::{channel, Receiver};
use futures::executor::block_on;
use futures::{Future, Stream, StreamExt};
use log::error;
use std::any::type_name;
use std::fmt::Debug;
use std::time::Duration;

#[allow(unused_variables)]
pub trait ActorService: Send + Sized {
    fn service_name() -> &'static str {
        type_name::<Self>()
    }

    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        Ok(())
    }
    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        Ok(())
    }
}

pub struct ServiceContext<'a, S>
where
    S: ActorService + 'static,
{
    cache: &'a mut ServiceCache,
    ctx: &'a mut Context<ServiceActor<S>>,
}

impl<'a, S> ServiceContext<'a, S>
where
    S: ActorService,
{
    pub(crate) fn new(cache: &'a mut ServiceCache, ctx: &'a mut Context<ServiceActor<S>>) -> Self {
        Self { cache, ctx }
    }

    pub fn set_mailbox_capacity(&mut self, cap: usize) {
        self.ctx.set_mailbox_capacity(cap);
    }

    /// Get Self's ServiceRef
    pub fn self_ref(&self) -> ServiceRef<S> {
        self.ctx.address().into()
    }

    pub fn registry_ref(&self) -> &ServiceRef<RegistryService> {
        self.cache.registry_ref()
    }

    pub fn bus_ref(&mut self) -> &ServiceRef<BusService> {
        //TODO return error?
        self.cache.bus_ref().expect("Get bus ref should success.")
    }

    pub fn service_ref<DepS>(&mut self) -> Result<&ServiceRef<DepS>>
    where
        DepS: ActorService,
    {
        self.cache
            .service_ref::<DepS>()?
            .ok_or_else(|| format_err!("Can not find service :{:?}", DepS::service_name()))
    }

    pub fn service_ref_opt<DepS>(&mut self) -> Result<Option<&ServiceRef<DepS>>>
    where
        DepS: ActorService,
    {
        self.cache.service_ref::<DepS>()
    }

    pub fn get_shared<T>(&self) -> Result<T>
    where
        T: Send + Sync + Clone + 'static,
    {
        let registry_ref = self.registry_ref();
        registry_ref.get_shared_sync()
    }

    pub fn get_shared_opt<T>(&self) -> Result<Option<T>>
    where
        T: Send + Sync + Clone + 'static,
    {
        let registry_ref = self.registry_ref();
        registry_ref.get_shared_opt_sync()
    }

    pub fn get_shared_or_put<T, F>(&self, f: F) -> Result<T>
    where
        T: Send + Sync + Clone + 'static,
        F: FnOnce() -> Result<T>,
    {
        let registry_ref = self.registry_ref();
        block_on(async move {
            let result = registry_ref.get_shared_opt::<T>().await?;
            match result {
                Some(r) => Ok(r),
                None => {
                    let r = f()?;
                    registry_ref.put_shared(r.clone()).await?;
                    Ok(r)
                }
            }
        })
    }

    pub fn put_shared<T>(&self, shared: T) -> Result<()>
    where
        T: Send + Sync + Clone + 'static,
    {
        let registry_ref = self.registry_ref();
        registry_ref.put_shared_sync(shared)
    }

    pub fn subscribe<M>(&mut self)
    where
        M: Send + Clone + Debug + 'static,
        S: EventHandler<S, M>,
    {
        let notifier = self.self_ref().event_notifier();
        let bus = self.bus_ref().clone();
        let fut = wrap_future::<_, ServiceActor<S>>(async move { bus.subscribe(notifier).await })
            .map(|r, _act, _ctx| {
                if let Err(e) = r {
                    error!(
                        "Subscribe {} for service {} error: {:?}",
                        type_name::<M>(),
                        S::service_name(),
                        e
                    );
                }
            });
        self.ctx.wait(fut.into_future());
    }

    pub fn add_stream<M, MS>(&mut self, stream: MS)
    where
        M: Send + Clone + Debug + 'static,
        S: EventHandler<S, M>,
        MS: Stream<Item = M> + 'static,
    {
        self.ctx.add_message_stream(stream.map(EventMessage::new))
    }

    pub fn unsubscribe<M>(&mut self)
    where
        M: Send + Clone + Debug + 'static,
        S: EventHandler<S, M>,
    {
        let bus = self.bus_ref().clone();
        let fut = wrap_future::<_, ServiceActor<S>>(async move { bus.unsubscribe::<S, M>().await })
            .map(|r, _act, _ctx| {
                if let Err(e) = r {
                    error!(
                        "Unsubscribe {} for service {} error: {:?}",
                        type_name::<M>(),
                        S::service_name(),
                        e
                    );
                }
            });
        self.ctx.wait(fut.into_future());
    }

    pub fn broadcast<M>(&mut self, msg: M)
    where
        M: Send + Clone + Debug + 'static,
    {
        let bus = self.bus_ref();
        if let Err(e) = bus.broadcast(msg) {
            //TODO wait and retry?
            error!(
                "Broadcast {} for service {} error: {:?}",
                type_name::<M>(),
                S::service_name(),
                e
            );
        }
    }

    pub fn run_later<F>(&mut self, dur: Duration, mut f: F)
    where
        F: FnMut(&mut ServiceContext<S>) + 'static,
    {
        self.ctx.run_later(dur, move |this, ctx| {
            let mut service_ctx = ServiceContext::new(&mut this.cache, ctx);
            f(&mut service_ctx)
        });
    }

    pub fn run_interval<F>(&mut self, dur: Duration, mut f: F)
    where
        F: FnMut(&mut ServiceContext<S>) + 'static,
    {
        self.ctx.run_interval(dur, move |this, ctx| {
            let mut service_ctx = ServiceContext::new(&mut this.cache, ctx);
            f(&mut service_ctx)
        });
    }

    /// Exec a future and get result.
    pub fn exec<F, R>(&mut self, fut: F) -> Receiver<R>
    where
        F: Future<Output = R> + 'static,
        R: 'static,
    {
        let (sender, receiver) = channel();
        let fut = wrap_future::<_, ServiceActor<S>>(async move {
            let result = fut.await;
            if sender.send(result).is_err() {
                error!("ServiceContext exec future send result error.");
            }
        });
        self.ctx.spawn(fut);
        receiver
    }

    pub fn wait<F>(&mut self, fut: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.ctx.wait(wrap_future::<_, ServiceActor<S>>(fut))
    }

    pub fn spawn<F>(&mut self, fut: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.ctx.spawn(wrap_future::<_, ServiceActor<S>>(fut));
    }

    /// Notify self a event msg.
    pub fn notify<M>(&mut self, msg: M)
    where
        S: EventHandler<S, M>,
        M: Debug + Send + 'static,
    {
        self.ctx.notify(EventMessage::new(msg))
    }

    pub fn stop_actor(&mut self) {
        self.ctx.stop()
    }
}

pub trait ServiceHandler<S, R>
where
    S: ActorService,
    R: ServiceRequest,
{
    fn handle(&mut self, msg: R, ctx: &mut ServiceContext<S>) -> <R as ServiceRequest>::Response;
}

pub trait EventHandler<S, M>
where
    S: ActorService,
    M: Debug + Send,
{
    fn handle_event(&mut self, msg: M, ctx: &mut ServiceContext<S>);
}

pub trait ServiceFactory<S>: Send
where
    S: ActorService,
{
    fn create(ctx: &mut ServiceContext<S>) -> Result<S>;
}

impl<S> ServiceFactory<S> for S
where
    S: ActorService + Default,
{
    fn create(_ctx: &mut ServiceContext<S>) -> Result<Self> {
        Ok(S::default())
    }
}
