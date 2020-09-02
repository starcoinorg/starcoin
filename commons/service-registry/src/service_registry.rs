// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::bus::BusService;
use crate::mocker::MockHandler;
use crate::service::{ActorService, ServiceFactory};
use crate::service_actor::ServiceActor;
use crate::{
    EventHandler, ServiceCmd, ServiceContext, ServiceHandler, ServiceInfo, ServiceRef,
    ServiceRequest, ServiceStatus,
};
use actix::{Actor, AsyncContext, Supervisor};
use actix_rt::Arbiter;
use anyhow::{bail, format_err, Result};
use futures::executor::block_on;
use log::info;
use serde::export::PhantomData;
use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

struct ServiceHolder<S>
where
    S: ActorService + 'static,
{
    arbiter: Arbiter,
    service_ref: ServiceRef<S>,
}

impl<S> ServiceHolder<S>
where
    S: ActorService,
{
    pub fn new(arbiter: Arbiter, service_ref: ServiceRef<S>) -> Self {
        Self {
            arbiter,
            service_ref,
        }
    }
}

trait ServiceRefProxy: Send + Sync {
    fn service_name(&self) -> &'static str;
    fn service_info(&self) -> ServiceInfo;
    fn status(&self) -> ServiceStatus;
    fn exec_service_cmd(&self, service_cmd: ServiceCmd) -> Result<()>;
    fn shutdown(&self) -> Result<()>;
    fn as_any(&self) -> &dyn Any;
}

impl<S> ServiceRefProxy for ServiceHolder<S>
where
    S: ActorService,
{
    fn service_name(&self) -> &'static str {
        S::service_name()
    }

    fn service_info(&self) -> ServiceInfo {
        ServiceInfo {
            name: self.service_name().to_string(),
            status: self.status(),
        }
    }

    fn status(&self) -> ServiceStatus {
        self.service_ref.self_status()
    }

    fn exec_service_cmd(&self, service_cmd: ServiceCmd) -> Result<()> {
        self.service_ref.exec_service_cmd(service_cmd)
    }

    fn shutdown(&self) -> Result<()> {
        info!("Start to shutdown {}.", self.service_name());
        self.arbiter.stop();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Registry {
    service_ref: ServiceRef<RegistryService>,
    shared: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
    //use vec to keep service registry order.
    services: Vec<Box<dyn ServiceRefProxy>>,
}

impl Registry {
    pub(crate) fn new(service_ref: ServiceRef<RegistryService>) -> Self {
        let mut registry = Registry {
            service_ref,
            shared: HashMap::new(),
            services: vec![],
        };
        registry
            .registry::<BusService>()
            .expect("Registry BusService should success");
        registry
    }

    pub fn put_shared<T>(&mut self, t: T)
    where
        T: Send + Sync + 'static,
    {
        self.shared.insert(TypeId::of::<T>(), Arc::new(t));
    }

    pub fn get_shared<T>(&self) -> Result<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.get_shared_opt::<T>()
            .ok_or_else(|| format_err!("Can not find shared by type: {}", type_name::<T>()))
    }

    pub fn get_shared_opt<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.shared
            .get(&TypeId::of::<T>())
            .and_then(|t| t.clone().downcast::<T>().ok())
    }

    pub fn has_service(&self, service_name: &str) -> bool {
        self.services
            .iter()
            .any(|handle| handle.service_name() == service_name)
    }

    pub fn get_service_status(&self, service_name: &str) -> Option<ServiceStatus> {
        self.services
            .iter()
            .find(|handle| handle.service_name() == service_name)
            .map(|handle| handle.status())
    }

    fn do_registry<S, F>(&mut self, f: F) -> Result<ServiceRef<S>>
    where
        S: ActorService + 'static,
        F: FnOnce(ServiceRef<RegistryService>) -> ServiceActor<S> + Send + 'static,
    {
        let service_name = S::service_name();
        if self.has_service(service_name) {
            bail!("Service with name: {} exist.", service_name)
        }
        info!("Registry service: {}", service_name);

        let arbiter = Arbiter::new();
        let registry_ref = self.service_ref.clone();
        let addr = Supervisor::start_in_arbiter(&arbiter, move |_ctx| f(registry_ref));
        let service_ref: ServiceRef<S> = addr.into();
        let holder = ServiceHolder::new(arbiter, service_ref.clone());
        self.services.push(Box::new(holder));
        Ok(service_ref)
    }

    pub fn registry<S>(&mut self) -> Result<ServiceRef<S>>
    where
        S: ActorService + ServiceFactory<S> + 'static,
    {
        self.do_registry(ServiceActor::new::<S>)
    }

    pub fn registry_mocker<S>(&mut self, mocker: Box<dyn MockHandler<S>>) -> Result<ServiceRef<S>>
    where
        S: ActorService + 'static,
    {
        self.do_registry(|registry_ref| ServiceActor::new_mocker(registry_ref, mocker))
    }

    pub fn list(&self) -> Vec<ServiceInfo> {
        self.services
            .iter()
            .map(|proxy| proxy.service_info())
            .collect()
    }

    pub fn service_ref<S>(&self) -> Result<ServiceRef<S>>
    where
        S: ActorService,
    {
        self.do_with_proxy(S::service_name(), |proxy| {
            proxy
                .as_any()
                .downcast_ref::<ServiceHolder<S>>()
                .ok_or_else(|| format_err!("Downcast ServiceHandle fail."))
                .map(|holder| holder.service_ref.clone())
        })
    }

    fn do_with_proxy<T, F: FnOnce(&Box<dyn ServiceRefProxy>) -> Result<T>>(
        &self,
        service_name: &str,
        f: F,
    ) -> Result<T> {
        let handle = self
            .services
            .iter()
            .find(|proxy| proxy.service_name() == service_name);
        match handle {
            Some(handle) => f(handle),
            None => bail!("Can not find service {}", service_name),
        }
    }

    fn exec_service_cmd(&self, service_name: &str, service_cmd: ServiceCmd) -> Result<()> {
        self.do_with_proxy(service_name, |proxy| proxy.exec_service_cmd(service_cmd))
    }

    fn exec_system_cmd(&self, cmd: SystemCmd) -> Result<()> {
        match cmd {
            SystemCmd::Shutdown => {
                info!("Start to shutdown system");
                for service in self.services.iter().rev() {
                    service.shutdown()?;
                }
            }
        }
        Ok(())
    }

    // Check service status and removed shutdown status service ref.
    fn check_service(&mut self) {
        let has_shutdown = self
            .services
            .iter()
            .any(|service| service.status() == ServiceStatus::Shutdown);
        if has_shutdown {
            self.services.retain(|service| {
                if service.status() == ServiceStatus::Shutdown {
                    info!(
                        "{} status is shutdown, remove service registry ref.",
                        service.service_name()
                    );
                    false
                } else {
                    true
                }
            });
        }
    }
}

pub struct RegistryService {
    registry: Registry,
}

impl ServiceFactory<RegistryService> for RegistryService {
    fn create(ctx: &mut ServiceContext<RegistryService>) -> Result<RegistryService> {
        Ok(Self {
            registry: Registry::new(ctx.registry_ref().clone()),
        })
    }
}

impl RegistryService {
    pub fn launch() -> ServiceRef<Self> {
        let addr = ServiceActor::create(|ctx| {
            let service_ref: ServiceRef<RegistryService> = ctx.address().into();
            ServiceActor::new::<RegistryService>(service_ref)
        });
        addr.into()
    }
}

impl ActorService for RegistryService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) {
        ctx.run_interval(Duration::from_millis(2000), |ctx| {
            ctx.notify(CheckServiceEvent)
        });
    }
}

pub struct RegistryRequest<S>
where
    S: ActorService + ServiceFactory<S> + 'static,
{
    phantom: PhantomData<S>,
}

#[allow(clippy::new_without_default)]
impl<S> RegistryRequest<S>
where
    S: ActorService + ServiceFactory<S>,
{
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<S> ServiceRequest for RegistryRequest<S>
where
    S: ActorService + ServiceFactory<S>,
{
    type Response = Result<ServiceRef<S>>;
}

impl<S> ServiceHandler<Self, RegistryRequest<S>> for RegistryService
where
    S: ActorService + ServiceFactory<S>,
{
    fn handle(
        &mut self,
        _msg: RegistryRequest<S>,
        _ctx: &mut ServiceContext<RegistryService>,
    ) -> Result<ServiceRef<S>> {
        self.registry.registry::<S>()
    }
}

pub struct RegistryMockerRequest<S>
where
    S: ActorService + 'static,
{
    mocker: Box<dyn MockHandler<S>>,
}

impl<S> RegistryMockerRequest<S>
where
    S: ActorService,
{
    pub fn new(mocker: Box<dyn MockHandler<S>>) -> Self {
        Self { mocker }
    }
}

impl<S> ServiceRequest for RegistryMockerRequest<S>
where
    S: ActorService,
{
    type Response = Result<ServiceRef<S>>;
}

impl<S> ServiceHandler<Self, RegistryMockerRequest<S>> for RegistryService
where
    S: ActorService,
{
    fn handle(
        &mut self,
        msg: RegistryMockerRequest<S>,
        _ctx: &mut ServiceContext<RegistryService>,
    ) -> Result<ServiceRef<S>> {
        self.registry.registry_mocker::<S>(msg.mocker)
    }
}

pub struct ListRequest;

impl ServiceRequest for ListRequest {
    type Response = Vec<ServiceInfo>;
}

impl ServiceHandler<Self, ListRequest> for RegistryService {
    fn handle(
        &mut self,
        _msg: ListRequest,
        _ctx: &mut ServiceContext<RegistryService>,
    ) -> Vec<ServiceInfo> {
        self.registry.list()
    }
}

pub struct ServiceRefRequest<S>
where
    S: ActorService + 'static,
{
    phantom: PhantomData<S>,
}

impl<S> ServiceRefRequest<S>
where
    S: ActorService,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<S> ServiceRequest for ServiceRefRequest<S>
where
    S: ActorService,
{
    type Response = Result<ServiceRef<S>>;
}

impl<S> ServiceHandler<Self, ServiceRefRequest<S>> for RegistryService
where
    S: ActorService,
{
    fn handle(
        &mut self,
        _msg: ServiceRefRequest<S>,
        _ctx: &mut ServiceContext<RegistryService>,
    ) -> Result<ServiceRef<S>> {
        self.registry.service_ref::<S>()
    }
}

pub struct PutShardRequest<T>
where
    T: Send + Sync + 'static,
{
    value: T,
}

impl<T> PutShardRequest<T>
where
    T: Send + Sync,
{
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> ServiceRequest for PutShardRequest<T>
where
    T: Send + Sync + 'static,
{
    type Response = ();
}

impl<T> ServiceHandler<Self, PutShardRequest<T>> for RegistryService
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, msg: PutShardRequest<T>, _ctx: &mut ServiceContext<RegistryService>) {
        self.registry.put_shared(msg.value);
    }
}

pub struct GetShardRequest<T>
where
    T: Send + Sync + 'static,
{
    phantom: PhantomData<T>,
}

impl<T> GetShardRequest<T>
where
    T: Send + Sync,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<T> ServiceRequest for GetShardRequest<T>
where
    T: Send + Sync + 'static,
{
    type Response = Option<Arc<T>>;
}

impl<T> ServiceHandler<Self, GetShardRequest<T>> for RegistryService
where
    T: Send + Sync + 'static,
{
    fn handle(
        &mut self,
        _msg: GetShardRequest<T>,
        _ctx: &mut ServiceContext<RegistryService>,
    ) -> Option<Arc<T>> {
        self.registry.get_shared_opt::<T>()
    }
}

pub struct ServiceStatusRequest {
    service_name: String,
}

impl ServiceStatusRequest {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }
}

impl ServiceRequest for ServiceStatusRequest {
    type Response = Option<ServiceStatus>;
}

impl ServiceHandler<Self, ServiceStatusRequest> for RegistryService {
    fn handle(
        &mut self,
        msg: ServiceStatusRequest,
        _ctx: &mut ServiceContext<RegistryService>,
    ) -> Option<ServiceStatus> {
        self.registry.get_service_status(msg.service_name.as_str())
    }
}

pub struct ServiceCmdRequest {
    service_name: String,
    service_cmd: ServiceCmd,
}

impl ServiceCmdRequest {
    pub fn new(service_name: String, service_cmd: ServiceCmd) -> Self {
        Self {
            service_name,
            service_cmd,
        }
    }
}

impl ServiceRequest for ServiceCmdRequest {
    type Response = Result<()>;
}

impl ServiceHandler<Self, ServiceCmdRequest> for RegistryService {
    fn handle(
        &mut self,
        msg: ServiceCmdRequest,
        _ctx: &mut ServiceContext<RegistryService>,
    ) -> Result<()> {
        self.registry
            .exec_service_cmd(msg.service_name.as_str(), msg.service_cmd)
    }
}

pub enum SystemCmd {
    Shutdown,
}

pub struct SystemCmdRequest {
    cmd: SystemCmd,
}

impl ServiceRequest for SystemCmdRequest {
    type Response = Result<()>;
}

impl ServiceHandler<Self, SystemCmdRequest> for RegistryService {
    fn handle(
        &mut self,
        msg: SystemCmdRequest,
        ctx: &mut ServiceContext<RegistryService>,
    ) -> Result<()> {
        self.registry.exec_system_cmd(msg.cmd)?;
        ctx.stop_actor();
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct CheckServiceEvent;

impl EventHandler<Self, CheckServiceEvent> for RegistryService {
    fn handle_event(
        &mut self,
        _msg: CheckServiceEvent,
        _ctx: &mut ServiceContext<RegistryService>,
    ) {
        self.registry.check_service();
    }
}

#[async_trait::async_trait]
pub trait RegistryAsyncService {
    async fn registry<S>(&self) -> Result<ServiceRef<S>>
    where
        S: ActorService + ServiceFactory<S> + 'static;

    async fn registry_mocker<S, Mocker>(&self, mocker: Mocker) -> Result<ServiceRef<S>>
    where
        S: ActorService + 'static,
        Mocker: MockHandler<S> + 'static;

    fn registry_sync<S>(&self) -> Result<ServiceRef<S>>
    where
        S: ActorService + ServiceFactory<S> + 'static,
    {
        block_on(async { self.registry::<S>().await })
    }

    async fn service_ref<S>(&self) -> Result<ServiceRef<S>>
    where
        S: ActorService + 'static;

    fn service_ref_sync<S>(&self) -> Result<ServiceRef<S>>
    where
        S: ActorService + 'static,
    {
        block_on(async { self.service_ref::<S>().await })
    }

    async fn list_service(&self) -> Result<Vec<ServiceInfo>>;
    async fn stop_service(&self, service_name: &str) -> Result<()>;
    async fn start_service(&self, service_name: &str) -> Result<()>;
    async fn restart_service(&self, service_name: &str) -> Result<()>;
    async fn get_service_status(&self, service_name: &str) -> Result<Option<ServiceStatus>>;

    async fn put_shared<T>(&self, t: T) -> Result<()>
    where
        T: Send + Sync + 'static;
    async fn get_shared<T>(&self) -> Result<Option<Arc<T>>>
    where
        T: Send + Sync + 'static;

    async fn shutdown(&self) -> Result<()>;
}

#[async_trait::async_trait]
impl RegistryAsyncService for ServiceRef<RegistryService> {
    async fn registry<S>(&self) -> Result<ServiceRef<S>>
    where
        S: ActorService + ServiceFactory<S> + 'static,
    {
        self.send(RegistryRequest::new()).await?
    }

    async fn registry_mocker<S, Mocker>(&self, mocker: Mocker) -> Result<ServiceRef<S>>
    where
        S: ActorService + 'static,
        Mocker: MockHandler<S> + 'static,
    {
        let handler = Box::new(mocker);
        self.send(RegistryMockerRequest::new(handler)).await?
    }

    async fn service_ref<S>(&self) -> Result<ServiceRef<S>>
    where
        S: ActorService + 'static,
    {
        self.send(ServiceRefRequest::new()).await?
    }

    async fn list_service(&self) -> Result<Vec<ServiceInfo>> {
        self.send(ListRequest).await
    }

    async fn stop_service(&self, service_name: &str) -> Result<()> {
        self.send(ServiceCmdRequest::new(
            service_name.to_string(),
            ServiceCmd::Stop,
        ))
        .await?
    }

    async fn start_service(&self, service_name: &str) -> Result<()> {
        self.send(ServiceCmdRequest::new(
            service_name.to_string(),
            ServiceCmd::Start,
        ))
        .await?
    }

    async fn restart_service(&self, service_name: &str) -> Result<()> {
        self.send(ServiceCmdRequest::new(
            service_name.to_string(),
            ServiceCmd::Restart,
        ))
        .await?
    }

    async fn get_service_status(&self, service_name: &str) -> Result<Option<ServiceStatus>> {
        self.send(ServiceStatusRequest::new(service_name.to_string()))
            .await
    }

    async fn put_shared<T>(&self, value: T) -> Result<()>
    where
        T: Send + Sync + 'static,
    {
        self.send(PutShardRequest::new(value)).await
    }

    async fn get_shared<T>(&self) -> Result<Option<Arc<T>>>
    where
        T: Send + Sync + 'static,
    {
        self.send(GetShardRequest::new()).await
    }

    async fn shutdown(&self) -> Result<()> {
        self.send(SystemCmdRequest {
            cmd: SystemCmd::Shutdown,
        })
        .await?
    }
}
