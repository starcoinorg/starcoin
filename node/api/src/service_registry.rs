// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::dev::ToEnvelope;
use actix::{Actor, Addr, Context, Handler};
use anyhow::{bail, format_err, Result};
use serde::{Deserialize, Serialize};
use starcoin_bus::BusActor;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_storage::Storage;
use starcoin_types::system_events::ActorStop;
use std::any::{type_name, Any};
use std::sync::{Arc, RwLock};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Stopped,
    Started,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub status: ServiceStatus,
}

pub trait SystemService
where
    Self: Actor<Context = Context<Self>>,
    Self: Handler<ActorStop>,
    Self::Context: ToEnvelope<Self, ActorStop>,
{
    fn service_name() -> &'static str {
        type_name::<Self>()
    }
}

pub trait ServiceHandle: Send {
    fn service_name(&self) -> &str;
    fn is_started(&self) -> bool;
    fn is_stopped(&self) -> bool;
    fn info(&self) -> ServiceInfo;
    fn status(&self) -> ServiceStatus;
    fn start(&mut self, registry: &ServiceRegistry) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn as_any(&self) -> &dyn Any;
}

struct SystemServiceHandle<S: SystemService> {
    creator: Box<dyn Fn(&ServiceRegistry) -> Result<S> + Send + 'static>,
    address: Option<Addr<S>>,
}

impl<S: SystemService> SystemServiceHandle<S> {
    pub fn new<F: 'static>(creator: F) -> Self
    where
        F: Fn(&ServiceRegistry) -> Result<S> + Send,
    {
        Self {
            creator: Box::new(creator),
            address: None,
        }
    }

    pub fn address(&self) -> Option<Addr<S>> {
        self.address.clone()
    }
}

impl<S: SystemService> ServiceHandle for SystemServiceHandle<S> {
    fn service_name(&self) -> &str {
        S::service_name()
    }

    fn is_started(&self) -> bool {
        match self.status() {
            ServiceStatus::Started => true,
            _ => false,
        }
    }

    fn is_stopped(&self) -> bool {
        match self.status() {
            ServiceStatus::Stopped => true,
            _ => false,
        }
    }

    fn info(&self) -> ServiceInfo {
        ServiceInfo {
            name: S::service_name().to_string(),
            status: self.status(),
        }
    }

    fn status(&self) -> ServiceStatus {
        //TODO do real service status check.
        if self.address.is_some() {
            ServiceStatus::Started
        } else {
            ServiceStatus::Stopped
        }
    }

    fn start(&mut self, registry: &ServiceRegistry) -> Result<()> {
        if self.is_started() {
            bail!("service has started");
        }
        let service = self.creator.as_ref()(registry)?;
        let addr = service.start();
        self.address = Some(addr);
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        let handle = self
            .address
            .take()
            .ok_or_else(|| format_err!("service has not started"))?;
        handle.try_send(ActorStop)?;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct ServiceRegistry {
    config: Arc<NodeConfig>,
    bus: Addr<BusActor>,
    storage: Option<Arc<Storage>>,
    //use vec to keep service registry order.
    services: RwLock<Vec<Box<dyn ServiceHandle>>>,
}

impl ServiceRegistry {
    pub fn new(config: Arc<NodeConfig>, bus: Addr<BusActor>) -> Self {
        ServiceRegistry {
            config,
            bus,
            storage: None,
            services: RwLock::new(vec![]),
        }
    }

    pub fn new_with_storage(
        config: Arc<NodeConfig>,
        bus: Addr<BusActor>,
        storage: Arc<Storage>,
    ) -> Self {
        ServiceRegistry {
            config,
            bus,
            storage: Some(storage),
            services: RwLock::new(vec![]),
        }
    }

    pub fn config(&self) -> Arc<NodeConfig> {
        self.config.clone()
    }

    pub fn storage(&self) -> Arc<Storage> {
        self.storage.clone().expect("Storage not init.")
    }

    pub fn bus(&self) -> Addr<BusActor> {
        self.bus.clone()
    }

    pub fn has_service(&self, service_name: &str) -> bool {
        self.services
            .read()
            .unwrap()
            .iter()
            .any(|handle| handle.service_name() == service_name)
    }

    pub fn registry<S, F>(&self, creator: F) -> Result<Addr<S>>
    where
        S: SystemService,
        F: Fn(&ServiceRegistry) -> Result<S> + Send + 'static,
    {
        let service_name = S::service_name();
        if self.has_service(service_name) {
            bail!("Service with name: {} exist.", service_name)
        }
        info!("Registry service: {}", service_name);
        let mut handle = SystemServiceHandle::new(creator);
        handle.start(self)?;
        let address = handle
            .address()
            .ok_or_else(|| format_err!("Service address "))?;
        self.services.write().unwrap().push(Box::new(handle));
        Ok(address)
    }

    fn do_with_handle_mut<T, F: FnOnce(&mut Box<dyn ServiceHandle>) -> Result<T>>(
        &self,
        service_name: &str,
        f: F,
    ) -> Result<T> {
        let mut services = self.services.write().unwrap();

        let handle = services
            .iter_mut()
            .find(|handle| handle.service_name() == service_name)
            .ok_or_else(|| format_err!("Can not find service by name:{}", service_name))?;
        f(handle)
    }

    fn do_with_handle<T, F: FnOnce(&Box<dyn ServiceHandle>) -> Option<T>>(
        &self,
        service_name: &str,
        f: F,
    ) -> Option<T> {
        let services = self.services.read().unwrap();

        let handle = services
            .iter()
            .find(|handle| handle.service_name() == service_name);
        match handle {
            Some(handle) => f(handle),
            None => None,
        }
    }

    pub fn list(&self) -> Vec<ServiceInfo> {
        self.services
            .read()
            .unwrap()
            .iter()
            .map(|handle| handle.info())
            .collect()
    }

    pub fn start<S: SystemService>(&self) -> Result<Addr<S>> {
        self.do_with_handle_mut(S::service_name(), |handle| handle.start(self))?;
        self.address::<S>()
            .ok_or_else(|| format_err!("Service start but get address fail."))
    }

    pub fn start_by_name(&self, service_name: &str) -> Result<()> {
        self.do_with_handle_mut(service_name, |handle| handle.start(self))
    }

    pub fn stop<S: SystemService>(&self) -> Result<()> {
        self.stop_by_name(S::service_name())
    }

    pub fn stop_by_name(&self, service_name: &str) -> Result<()> {
        self.do_with_handle_mut(service_name, |handle| handle.stop())
    }

    pub fn address<S: SystemService>(&self) -> Option<Addr<S>> {
        self.do_with_handle(S::service_name(), |handle| {
            handle
                .as_any()
                .downcast_ref::<SystemServiceHandle<S>>()
                .and_then(|handle| handle.address())
        })
    }

    pub fn service_info<S: SystemService>(&self) -> Option<ServiceInfo> {
        self.service_info_by_name(S::service_name())
    }

    pub fn service_info_by_name(&self, service_name: &str) -> Option<ServiceInfo> {
        self.do_with_handle(service_name, |handle| Some(handle.info()))
    }
}
