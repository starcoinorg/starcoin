// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::mocker::MockHandler;
use crate::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest,
    ServiceStatus,
};
use anyhow::{bail, Result};
use log::{error, info, warn};
use std::any::{type_name, Any};
use std::fmt::Debug;
use std::marker::PhantomData;

pub trait HandlerProxy<S>
where
    S: ActorService + 'static,
{
    fn start(&mut self, ctx: &mut ServiceContext<S>) -> Result<()>;
    fn stop(&mut self, ctx: &mut ServiceContext<S>) -> Result<()>;
    fn restart(&mut self, ctx: &mut ServiceContext<S>) -> Result<()> {
        if self.status().is_started() {
            if let Err(e) = self.stop(ctx) {
                warn!("Stop service error on restart: {:?}", e);
            }
        }
        self.start(ctx)
    }
    fn status(&self) -> ServiceStatus;
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

type ServiceCreator<S> = Box<dyn Fn(&mut ServiceContext<S>) -> Result<S> + Send>;

pub(crate) struct ServiceHandlerProxy<S>
where
    S: ActorService + 'static,
{
    service_creator: ServiceCreator<S>,
    service: Option<S>,
}

impl<S> ServiceHandlerProxy<S>
where
    S: ActorService,
{
    pub fn new<F>() -> Self
    where
        F: ServiceFactory<S>,
    {
        Self {
            service_creator: Box::new(|ctx| F::create(ctx)),
            service: None,
        }
    }
}

impl<S> HandlerProxy<S> for ServiceHandlerProxy<S>
where
    S: ActorService,
{
    fn start(&mut self, ctx: &mut ServiceContext<S>) -> Result<()> {
        if self.status().is_started() {
            warn!("Service {} has bean started", S::service_name());
            return Ok(());
        }
        let mut service = match (self.service_creator)(ctx) {
            Err(e) => {
                error!("Create service {} error: {:?}", S::service_name(), e);
                return Err(e);
            }
            Ok(service) => service,
        };
        service.started(ctx)?;
        self.service.replace(service);
        info!("Service {} start.", S::service_name());
        Ok(())
    }

    fn stop(&mut self, ctx: &mut ServiceContext<S>) -> Result<()> {
        if self.status().is_stopped() {
            info!("Service {} has bean stopped", S::service_name());
            return Ok(());
        }
        let service = self.service.take();
        if let Some(mut service) = service {
            service.stopped(ctx)?;
        }
        info!("Service {} stop.", S::service_name());
        Ok(())
    }

    fn status(&self) -> ServiceStatus {
        if self.service.is_some() {
            ServiceStatus::Started
        } else {
            ServiceStatus::Stopped
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl<S, R> ServiceHandler<S, R> for ServiceHandlerProxy<S>
where
    R: ServiceRequest,
    S: ActorService + ServiceHandler<S, R>,
{
    fn handle(&mut self, r: R, ctx: &mut ServiceContext<S>) -> <R as ServiceRequest>::Response {
        self.service
            .as_mut()
            .expect("Service should exist at here")
            .handle(r, ctx)
    }
}

impl<S, M> EventHandler<S, M> for ServiceHandlerProxy<S>
where
    M: Debug + Send,
    S: ActorService + EventHandler<S, M>,
{
    fn handle_event(&mut self, msg: M, ctx: &mut ServiceContext<S>) {
        self.service
            .as_mut()
            .expect("Service should exist at here")
            .handle_event(msg, ctx)
    }
}

pub(crate) struct MockHandlerProxy<S>
where
    S: ActorService + 'static,
{
    service: PhantomData<S>,
    status: ServiceStatus,
    mocker: Box<dyn MockHandler<S>>,
}

impl<S> MockHandlerProxy<S>
where
    S: ActorService,
{
    pub fn new(mocker: Box<dyn MockHandler<S>>) -> Self {
        Self {
            service: PhantomData,
            status: ServiceStatus::Started,
            mocker,
        }
    }
}

impl<S> HandlerProxy<S> for MockHandlerProxy<S>
where
    S: ActorService,
{
    fn start(&mut self, _ctx: &mut ServiceContext<S>) -> Result<()> {
        if self.status().is_started() {
            bail!("Service {} has bean started", S::service_name())
        }
        info!("Mock {} handler do start.", S::service_name());
        self.status = ServiceStatus::Started;
        Ok(())
    }

    fn stop(&mut self, _ctx: &mut ServiceContext<S>) -> Result<()> {
        if self.status().is_stopped() {
            bail!("Service {} has bean stopped", S::service_name())
        }
        info!("Mock {} handler do stop.", S::service_name());
        self.status = ServiceStatus::Stopped;
        Ok(())
    }

    fn status(&self) -> ServiceStatus {
        self.status
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl<S, R> ServiceHandler<S, R> for MockHandlerProxy<S>
where
    R: ServiceRequest + 'static,
    S: ActorService + ServiceHandler<S, R>,
{
    fn handle(&mut self, r: R, ctx: &mut ServiceContext<S>) -> <R as ServiceRequest>::Response {
        let resp = self.mocker.as_mut().handle(Box::new(r), ctx);
        match resp.downcast::<<R as ServiceRequest>::Response>() {
            Ok(resp) => *resp,
            Err(resp) => panic!(
                "Expect response type: {}, but get: {:?}",
                type_name::<<R as ServiceRequest>::Response>(),
                resp
            ),
        }
    }
}

impl<S, M> EventHandler<S, M> for MockHandlerProxy<S>
where
    M: Debug + Send + 'static,
    S: ActorService + EventHandler<S, M>,
{
    fn handle_event(&mut self, msg: M, ctx: &mut ServiceContext<S>) {
        self.mocker.as_mut().handle_event(Box::new(msg), ctx);
    }
}
