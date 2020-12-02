// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::handler_proxy::{HandlerProxy, MockHandlerProxy, ServiceHandlerProxy};
use crate::mocker::MockHandler;
use crate::service::{ActorService, ServiceContext, ServiceFactory, ServiceHandler};
use crate::service_cache::ServiceCache;
use crate::service_registry::ServiceStatusChangeEvent;
use crate::{
    EventHandler, RegistryService, ServiceCmd, ServiceEventStream, ServicePing, ServiceQuery,
    ServiceQueryResult, ServiceRef, ServiceRequest,
};
use actix::{Actor, AsyncContext, Context, Handler, Message, MessageResult, Supervised};
use anyhow::{format_err, Result};
use futures::{Stream, StreamExt};
use log::{debug, error, info};
use std::fmt::Debug;

const DEFAULT_MAIL_BOX_CAP: usize = 128;

pub struct ServiceActor<S>
where
    S: ActorService + 'static,
{
    proxy: Box<dyn HandlerProxy<S> + Send>,
    pub(crate) cache: ServiceCache,
}

impl<S> ServiceActor<S>
where
    S: ActorService,
{
    pub fn new<F>(registry: ServiceRef<RegistryService>) -> Self
    where
        F: ServiceFactory<S>,
    {
        Self {
            proxy: Box::new(ServiceHandlerProxy::new::<F>()),
            cache: ServiceCache::new(registry),
        }
    }

    pub fn new_mocker(
        registry: ServiceRef<RegistryService>,
        mocker: Box<dyn MockHandler<S>>,
    ) -> Self {
        Self {
            proxy: Box::new(MockHandlerProxy::new(mocker)),
            cache: ServiceCache::new(registry),
        }
    }

    fn notify_status(&self) {
        if self.cache.registry_ref().connected() {
            if let Err(e) = self
                .cache
                .registry_ref()
                .notify(ServiceStatusChangeEvent::new(
                    S::service_name().to_string(),
                    self.proxy.status(),
                ))
            {
                error!("Report status to registry error: {:?}", e);
            }
        }
    }
}

impl<S> Actor for ServiceActor<S>
where
    S: ActorService,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(DEFAULT_MAIL_BOX_CAP);
        let mut service_ctx = ServiceContext::new(&mut self.cache, ctx);
        if let Err(e) = self.proxy.start(&mut service_ctx) {
            error!("{} service start fail: {:?}.", S::service_name(), e);
        } else {
            info!("{} service actor started", S::service_name());
        }
        self.notify_status();
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let mut service_ctx = ServiceContext::new(&mut self.cache, ctx);
        if let Err(e) = self.proxy.stop(&mut service_ctx) {
            error!("{} service stop fail: {:?}.", S::service_name(), e);
        } else {
            info!("{} service actor stopped", S::service_name());
        }
        self.notify_status();
    }
}

impl<S> Supervised for ServiceActor<S>
where
    S: ActorService,
{
    fn restarting(&mut self, ctx: &mut Self::Context) {
        let mut service_ctx = ServiceContext::new(&mut self.cache, ctx);
        if let Err(e) = self.proxy.restart(&mut service_ctx) {
            error!("Restarting service actor error: {:?}", e);
        }
        info!("{} service actor restarted", S::service_name());
    }
}

#[derive(Debug)]
pub struct ServiceMessage<R: ServiceRequest + 'static> {
    request: R,
}

impl<R: ServiceRequest> ServiceMessage<R> {
    pub fn new(request: R) -> Self {
        Self { request }
    }

    pub fn into_inner(self) -> R {
        self.request
    }
}

impl<R> From<R> for ServiceMessage<R>
where
    R: ServiceRequest,
{
    fn from(request: R) -> Self {
        ServiceMessage { request }
    }
}

impl<R: ServiceRequest> Message for ServiceMessage<R> {
    type Result = Result<R::Response>;
}

impl<S, R> Handler<ServiceMessage<R>> for ServiceActor<S>
where
    R: ServiceRequest,
    S: ActorService + ServiceHandler<S, R>,
{
    type Result = MessageResult<ServiceMessage<R>>;

    fn handle(&mut self, msg: ServiceMessage<R>, ctx: &mut Self::Context) -> Self::Result {
        debug!("{} handle request: {:?}", S::service_name(), &msg.request);
        if self.proxy.status().is_stopped() {
            return MessageResult(Err(format_err!("Service {} is stopped", S::service_name())));
        }
        let mut service_ctx = ServiceContext::new(&mut self.cache, ctx);
        let proxy_any = self.proxy.as_mut_any();
        let resp = if let Some(proxy) = proxy_any.downcast_mut::<ServiceHandlerProxy<S>>() {
            proxy.handle(msg.request, &mut service_ctx)
        } else if let Some(proxy) = proxy_any.downcast_mut::<MockHandlerProxy<S>>() {
            proxy.handle(msg.request, &mut service_ctx)
        } else {
            unreachable!("Unknown HandlerProxy type.")
        };
        MessageResult(Ok(resp))
    }
}

impl<S> Handler<ServiceCmd> for ServiceActor<S>
where
    S: ActorService,
{
    type Result = Result<()>;

    fn handle(&mut self, msg: ServiceCmd, ctx: &mut Self::Context) -> Self::Result {
        debug!("{} Actor handle ServiceCmd: {:?}", S::service_name(), msg);
        let mut service_ctx = ServiceContext::new(&mut self.cache, ctx);
        let result = match msg {
            ServiceCmd::Stop => self.proxy.stop(&mut service_ctx),
            ServiceCmd::Start => self.proxy.start(&mut service_ctx),
            ServiceCmd::Restart => self.proxy.restart(&mut service_ctx),
        };
        self.notify_status();
        result
    }
}

impl<S> Handler<ServiceQuery> for ServiceActor<S>
where
    S: ActorService,
{
    type Result = MessageResult<ServiceQuery>;

    fn handle(&mut self, msg: ServiceQuery, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ServiceQuery::Status => MessageResult(ServiceQueryResult::Status(self.proxy.status())),
        }
    }
}

impl<S> Handler<ServicePing> for ServiceActor<S>
where
    S: ActorService,
{
    type Result = ();

    fn handle(&mut self, _msg: ServicePing, _ctx: &mut Self::Context) -> Self::Result {
        //do nothing
    }
}

impl<S, Fut, M> Handler<ServiceEventStream<Fut>> for ServiceActor<S>
where
    S: ActorService,
    S: EventHandler<S, M>,
    Fut: Stream<Item = M>,
    M: Send + Debug + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: ServiceEventStream<Fut>, ctx: &mut Self::Context) {
        ctx.add_message_stream(msg.stream.map(EventMessage::new));
    }
}

#[derive(Clone, Debug)]
pub struct EventMessage<M>
where
    M: Debug + Send,
{
    msg: M,
}

impl<M> EventMessage<M>
where
    M: Debug + Send,
{
    pub fn new(msg: M) -> Self {
        Self { msg }
    }

    pub fn into_inner(self) -> M {
        self.msg
    }
}

impl<M> Message for EventMessage<M>
where
    M: Debug + Send,
{
    type Result = ();
}

impl<S, M> Handler<EventMessage<M>> for ServiceActor<S>
where
    M: Debug + Send + 'static,
    S: ActorService + EventHandler<S, M>,
{
    type Result = ();

    fn handle(&mut self, msg: EventMessage<M>, ctx: &mut Self::Context) -> Self::Result {
        debug!("{} handle event: {:?}", S::service_name(), &msg.msg);
        if self.proxy.status().is_stopped() {
            info!("Service {} is already stopped", S::service_name());
            return;
        }
        let mut service_ctx = ServiceContext::new(&mut self.cache, ctx);

        let proxy_any = self.proxy.as_mut_any();
        if let Some(proxy) = proxy_any.downcast_mut::<ServiceHandlerProxy<S>>() {
            proxy.handle_event(msg.msg, &mut service_ctx);
        } else if let Some(proxy) = proxy_any.downcast_mut::<MockHandlerProxy<S>>() {
            proxy.handle_event(msg.msg, &mut service_ctx);
        } else {
            unreachable!("Unknown HandlerProxy type.")
        };
    }
}
