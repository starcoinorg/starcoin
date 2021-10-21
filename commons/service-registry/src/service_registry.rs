// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::bus::BusService;
use crate::mocker::MockHandler;
use crate::service::{ActorService, ServiceFactory};
use crate::service_actor::ServiceActor;
use crate::{
    EventHandler, ServiceCmd, ServiceContext, ServiceHandler, ServiceInfo, ServicePing, ServiceRef,
    ServiceRequest, ServiceStatus,
};
use actix::prelude::SendError;
use actix::{Actor, AsyncContext};
use actix_rt::Arbiter;
use anyhow::{bail, format_err, Result};
use futures::executor::block_on;
use log::info;
use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::marker::PhantomData;

trait ServiceRefProxy: Send + Sync {
    fn service_name(&self) -> &'static str;
    fn service_info(&self) -> ServiceInfo;
    fn status(&self) -> ServiceStatus;
    fn check_status(&self) -> ServiceStatus;
    fn update_status(&mut self, status: ServiceStatus);
    fn exec_service_cmd(&self, service_cmd: ServiceCmd) -> Result<()>;
    fn shutdown(&self) -> Result<()>;
    fn as_any(&self) -> &dyn Any;
}

struct ServiceHolder<S>
where
    S: ActorService + 'static,
{
    arbiter: Arbiter,
    status: ServiceStatus,
    service_ref: ServiceRef<S>,
}

impl<S> ServiceHolder<S>
where
    S: ActorService,
{
    pub fn new(arbiter: Arbiter, service_ref: ServiceRef<S>) -> Self {
        Self {
            arbiter,
            status: ServiceStatus::Started,
            service_ref,
        }
    }
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
        if self.service_ref.addr.connected() {
            self.status
        } else {
            ServiceStatus::Shutdown
        }
    }

    fn check_status(&self) -> ServiceStatus {
        if self.status.is_started() {
            if let Err(e) = self.service_ref.addr.try_send(ServicePing) {
                match e {
                    SendError::Full(_) => ServiceStatus::Unavailable,
                    SendError::Closed(_) => ServiceStatus::Shutdown,
                }
            } else {
                ServiceStatus::Started
            }
        } else {
            self.status
        }
    }

    fn update_status(&mut self, status: ServiceStatus) {
        self.status = status;
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
    shared: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
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
            .register::<BusService, BusService>()
            .expect("Registry BusService should success");
        registry
    }

    pub fn put_shared<T>(&mut self, t: T)
    where
        T: Send + Sync + Clone + 'static,
    {
        info!("Put shared by type: {}", type_name::<T>());
        self.shared.insert(TypeId::of::<T>(), Box::new(t));
    }

    pub fn remove_shared<T>(&mut self)
    where
        T: Send + Sync + Clone + 'static,
    {
        info!("Remove shared by type: {}", type_name::<T>());
        self.shared.remove(&TypeId::of::<T>());
    }

    pub fn get_shared<T>(&self) -> Result<T>
    where
        T: Send + Sync + Clone + 'static,
    {
        self.get_shared_opt::<T>()
            .ok_or_else(|| format_err!("Can not find shared by type: {}", type_name::<T>()))
    }

    pub fn get_shared_opt<T>(&self) -> Option<T>
    where
        T: Send + Sync + Clone + 'static,
    {
        self.shared
            .get(&TypeId::of::<T>())
            .and_then(|t| t.downcast_ref::<T>().cloned())
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

    pub fn check_service_status(&self, service_name: &str) -> Option<ServiceStatus> {
        self.services
            .iter()
            .find(|handle| handle.service_name() == service_name)
            .map(|handle| handle.check_status())
    }

    fn do_register<S, F>(&mut self, f: F) -> Result<ServiceRef<S>>
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
        let addr = ServiceActor::start_in_arbiter(&arbiter, move |_ctx| f(registry_ref));
        let service_ref: ServiceRef<S> = addr.into();
        let holder = ServiceHolder::new(arbiter, service_ref.clone());
        self.services.push(Box::new(holder));
        Ok(service_ref)
    }

    pub fn register<S, F>(&mut self) -> Result<ServiceRef<S>>
    where
        S: ActorService + 'static,
        F: ServiceFactory<S> + 'static,
    {
        self.do_register(ServiceActor::new::<F>)
    }

    pub fn register_mocker<S>(&mut self, mocker: Box<dyn MockHandler<S>>) -> Result<ServiceRef<S>>
    where
        S: ActorService + 'static,
    {
        self.do_register(|registry_ref| ServiceActor::new_mocker(registry_ref, mocker))
    }

    /// Stop service thread and remove from registry.
    /// A service after shutdown, can not start again, must been registry again.
    pub fn shutdown_service(&mut self, service_name: &str) -> Result<()> {
        self.do_with_proxy(service_name, |proxy| proxy.shutdown())?;
        self.services
            .retain(|proxy| proxy.service_name() != service_name);
        Ok(())
    }

    pub fn list(&self) -> Vec<ServiceInfo> {
        self.services
            .iter()
            .map(|proxy| proxy.service_info())
            .collect()
    }

    pub fn service_ref<S>(&self) -> Option<ServiceRef<S>>
    where
        S: ActorService,
    {
        let service_name = S::service_name();
        self.services
            .iter()
            .find(|proxy| proxy.service_name() == service_name)
            .map(|proxy| {
                proxy
                    .as_any()
                    .downcast_ref::<ServiceHolder<S>>()
                    .expect("Downcast to ServiceHolder should success.")
            })
            .map(|holder| holder.service_ref.clone())
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

    fn exec_system_cmd(&mut self, cmd: SystemCmd) -> Result<()> {
        match cmd {
            SystemCmd::Shutdown => {
                info!("Start to shutdown system");
                for service in self.services.iter().rev() {
                    service.shutdown()?;
                }
            }
            SystemCmd::ShutdownService(service_name) => {
                info!("Start to shutdown service: {}", service_name);
                self.shutdown_service(service_name.as_str())?;
            }
        }
        Ok(())
    }

    fn update_service_status(&mut self, service_name: &str, status: ServiceStatus) {
        if let Some(handle) = self
            .services
            .iter_mut()
            .find(|proxy| proxy.service_name() == service_name)
        {
            handle.update_status(status)
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
        let arbiter = Arbiter::new();
        let addr = ServiceActor::start_in_arbiter(&arbiter, |ctx| {
            let service_ref: ServiceRef<RegistryService> = ctx.address().into();
            ServiceActor::new::<RegistryService>(service_ref)
        });
        addr.into()
    }
}

impl ActorService for RegistryService {
    fn started(&mut self, _ctx: &mut ServiceContext<Self>) -> Result<()> {
        Ok(())
    }
}

pub struct RegisterRequest<S, F>
where
    S: ActorService + 'static,
    F: ServiceFactory<S> + 'static,
{
    phantom_service: PhantomData<S>,
    phantom_factory: PhantomData<F>,
}

impl<S, F> Debug for RegisterRequest<S, F>
where
    S: ActorService,
    F: ServiceFactory<S>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", type_name::<Self>())
    }
}

#[allow(clippy::new_without_default)]
impl<S, F> RegisterRequest<S, F>
where
    S: ActorService,
    F: ServiceFactory<S>,
{
    pub fn new() -> Self {
        Self {
            phantom_service: PhantomData,
            phantom_factory: PhantomData,
        }
    }
}

impl<S, F> ServiceRequest for RegisterRequest<S, F>
where
    S: ActorService,
    F: ServiceFactory<S>,
{
    type Response = Result<ServiceRef<S>>;
}

impl<S, F> ServiceHandler<Self, RegisterRequest<S, F>> for RegistryService
where
    S: ActorService,
    F: ServiceFactory<S>,
{
    fn handle(
        &mut self,
        _msg: RegisterRequest<S, F>,
        _ctx: &mut ServiceContext<RegistryService>,
    ) -> Result<ServiceRef<S>> {
        self.registry.register::<S, F>()
    }
}

pub struct RegisterMockerRequest<S>
where
    S: ActorService + 'static,
{
    mocker: Box<dyn MockHandler<S>>,
}

impl<S> Debug for RegisterMockerRequest<S>
where
    S: ActorService,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", type_name::<Self>())
    }
}

impl<S> RegisterMockerRequest<S>
where
    S: ActorService,
{
    pub fn new(mocker: Box<dyn MockHandler<S>>) -> Self {
        Self { mocker }
    }
}

impl<S> ServiceRequest for RegisterMockerRequest<S>
where
    S: ActorService,
{
    type Response = Result<ServiceRef<S>>;
}

impl<S> ServiceHandler<Self, RegisterMockerRequest<S>> for RegistryService
where
    S: ActorService,
{
    fn handle(
        &mut self,
        msg: RegisterMockerRequest<S>,
        _ctx: &mut ServiceContext<RegistryService>,
    ) -> Result<ServiceRef<S>> {
        self.registry.register_mocker::<S>(msg.mocker)
    }
}

#[derive(Debug)]
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

impl<S> Debug for ServiceRefRequest<S>
where
    S: ActorService,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", type_name::<Self>())
    }
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
    type Response = Option<ServiceRef<S>>;
}

impl<S> ServiceHandler<Self, ServiceRefRequest<S>> for RegistryService
where
    S: ActorService,
{
    fn handle(
        &mut self,
        _msg: ServiceRefRequest<S>,
        _ctx: &mut ServiceContext<RegistryService>,
    ) -> Option<ServiceRef<S>> {
        self.registry.service_ref::<S>()
    }
}

pub struct PutShardRequest<T>
where
    T: Send + Sync + Clone + 'static,
{
    value: T,
}

impl<T> Debug for PutShardRequest<T>
where
    T: Send + Sync + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}<{}>", type_name::<Self>(), type_name::<T>())
    }
}

impl<T> PutShardRequest<T>
where
    T: Send + Sync + Clone,
{
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> ServiceRequest for PutShardRequest<T>
where
    T: Send + Sync + Clone + 'static,
{
    type Response = ();
}

impl<T> ServiceHandler<Self, PutShardRequest<T>> for RegistryService
where
    T: Send + Sync + Clone + 'static,
{
    fn handle(&mut self, msg: PutShardRequest<T>, _ctx: &mut ServiceContext<RegistryService>) {
        self.registry.put_shared(msg.value);
    }
}

pub struct GetShardRequest<T>
where
    T: Send + Sync + Clone + 'static,
{
    phantom: PhantomData<T>,
}

impl<T> Debug for GetShardRequest<T>
where
    T: Send + Sync + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}<{}>", type_name::<Self>(), type_name::<T>())
    }
}

impl<T> GetShardRequest<T>
where
    T: Send + Sync + Clone,
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
    T: Send + Sync + Clone + 'static,
{
    type Response = Option<T>;
}

impl<T> ServiceHandler<Self, GetShardRequest<T>> for RegistryService
where
    T: Send + Sync + Clone + 'static,
{
    fn handle(
        &mut self,
        _msg: GetShardRequest<T>,
        _ctx: &mut ServiceContext<RegistryService>,
    ) -> Option<T> {
        self.registry.get_shared_opt::<T>()
    }
}

pub struct RemoveShardRequest<T>
where
    T: Send + Sync + Clone + 'static,
{
    phantom: PhantomData<T>,
}

impl<T> Debug for RemoveShardRequest<T>
where
    T: Send + Sync + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}<{}>", type_name::<Self>(), type_name::<T>())
    }
}

impl<T> RemoveShardRequest<T>
where
    T: Send + Sync + Clone,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<T> ServiceRequest for RemoveShardRequest<T>
where
    T: Send + Sync + Clone + 'static,
{
    type Response = ();
}

impl<T> ServiceHandler<Self, RemoveShardRequest<T>> for RegistryService
where
    T: Send + Sync + Clone + 'static,
{
    fn handle(&mut self, _msg: RemoveShardRequest<T>, _ctx: &mut ServiceContext<RegistryService>) {
        self.registry.remove_shared::<T>();
    }
}

#[derive(Debug)]
pub struct CheckServiceStatusRequest {
    service_name: String,
}

impl CheckServiceStatusRequest {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }
}

impl ServiceRequest for CheckServiceStatusRequest {
    type Response = Result<ServiceStatus>;
}

impl ServiceHandler<Self, CheckServiceStatusRequest> for RegistryService {
    fn handle(
        &mut self,
        msg: CheckServiceStatusRequest,
        _ctx: &mut ServiceContext<RegistryService>,
    ) -> Result<ServiceStatus> {
        self.registry
            .get_service_status(msg.service_name.as_str())
            .ok_or_else(|| format_err!("Can not find service by name: {}", msg.service_name))
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum SystemCmd {
    ///Shutdown system
    Shutdown,
    ShutdownService(String),
}

#[derive(Debug)]
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
pub(crate) struct ServiceStatusChangeEvent {
    service_name: String,
    status: ServiceStatus,
}

impl ServiceStatusChangeEvent {
    pub fn new(service_name: String, status: ServiceStatus) -> Self {
        Self {
            service_name,
            status,
        }
    }
}

impl EventHandler<Self, ServiceStatusChangeEvent> for RegistryService {
    fn handle_event(
        &mut self,
        msg: ServiceStatusChangeEvent,
        _ctx: &mut ServiceContext<RegistryService>,
    ) {
        self.registry
            .update_service_status(msg.service_name.as_str(), msg.status);
    }
}

#[async_trait::async_trait]
pub trait RegistryAsyncService {
    async fn register<S>(&self) -> Result<ServiceRef<S>>
    where
        S: ActorService + ServiceFactory<S> + 'static;

    async fn register_by_factory<S, F>(&self) -> Result<ServiceRef<S>>
    where
        S: ActorService + 'static,
        F: ServiceFactory<S> + 'static;

    async fn register_mocker<S, Mocker>(&self, mocker: Mocker) -> Result<ServiceRef<S>>
    where
        S: ActorService + 'static,
        Mocker: MockHandler<S> + 'static;

    fn registry_sync<S>(&self) -> Result<ServiceRef<S>>
    where
        S: ActorService + ServiceFactory<S> + 'static,
    {
        block_on(async { self.register::<S>().await })
    }

    async fn service_ref<S>(&self) -> Result<ServiceRef<S>>
    where
        S: ActorService + 'static,
    {
        self.service_ref_opt()
            .await?
            .ok_or_else(|| format_err!("Can not find service: {}", S::service_name()))
    }

    async fn service_ref_opt<S>(&self) -> Result<Option<ServiceRef<S>>>
    where
        S: ActorService + 'static;

    async fn list_service(&self) -> Result<Vec<ServiceInfo>>;

    fn list_service_sync(&self) -> Result<Vec<ServiceInfo>> {
        block_on(async move { self.list_service().await })
    }
    async fn stop_service(&self, service_name: &str) -> Result<()>;

    fn stop_service_sync(&self, service_name: &str) -> Result<()> {
        block_on(async move { self.stop_service(service_name).await })
    }

    async fn start_service(&self, service_name: &str) -> Result<()>;

    fn start_service_sync(&self, service_name: &str) -> Result<()> {
        block_on(async move { self.start_service(service_name).await })
    }

    async fn restart_service(&self, service_name: &str) -> Result<()>;

    fn restart_service_sync(&self, service_name: &str) -> Result<()> {
        block_on(async move { self.restart_service(service_name).await })
    }
    async fn check_service_status(&self, service_name: &str) -> Result<ServiceStatus>;

    fn check_service_status_sync(&self, service_name: &str) -> Result<ServiceStatus> {
        block_on(async move { self.check_service_status(service_name).await })
    }

    async fn put_shared<T>(&self, t: T) -> Result<()>
    where
        T: Send + Sync + Clone + 'static;

    fn put_shared_sync<T>(&self, t: T) -> Result<()>
    where
        T: Send + Sync + Clone + 'static,
    {
        block_on(async move { self.put_shared(t).await })
    }

    async fn get_shared_opt<T>(&self) -> Result<Option<T>>
    where
        T: Send + Sync + Clone + 'static;

    async fn get_shared<T>(&self) -> Result<T>
    where
        T: Send + Sync + Clone + 'static,
    {
        self.get_shared_opt()
            .await?
            .ok_or_else(|| format_err!("Can not find shared data by type: {}", type_name::<T>()))
    }

    fn get_shared_sync<T>(&self) -> Result<T>
    where
        T: Send + Sync + Clone + 'static,
    {
        self.get_shared_opt_sync()?
            .ok_or_else(|| format_err!("Can not find shared data by type: {}", type_name::<T>()))
    }

    fn get_shared_opt_sync<T>(&self) -> Result<Option<T>>
    where
        T: Send + Sync + Clone + 'static,
    {
        block_on(async { self.get_shared_opt().await })
    }

    async fn remove_shared<T>(&self) -> Result<()>
    where
        T: Send + Sync + Clone + 'static;

    fn remove_shared_sync<T>(&self) -> Result<()>
    where
        T: Send + Sync + Clone + 'static,
    {
        block_on(async { self.remove_shared::<T>().await })
    }

    async fn shutdown_system(&self) -> Result<()>;

    fn shutdown_system_sync(&self) -> Result<()> {
        block_on(async { self.shutdown_system().await })
    }

    async fn shutdown_service<S>(&self) -> Result<()>
    where
        S: ActorService + 'static;

    fn shutdown_service_sync<S>(&self) -> Result<()>
    where
        S: ActorService + 'static,
    {
        block_on(async { self.shutdown_service::<S>().await })
    }
}

#[async_trait::async_trait]
impl RegistryAsyncService for ServiceRef<RegistryService> {
    async fn register<S>(&self) -> Result<ServiceRef<S>>
    where
        S: ActorService + ServiceFactory<S> + 'static,
    {
        self.send(RegisterRequest::<S, S>::new()).await?
    }

    async fn register_by_factory<S, F>(&self) -> Result<ServiceRef<S>>
    where
        S: ActorService + 'static,
        F: ServiceFactory<S> + 'static,
    {
        self.send(RegisterRequest::<S, F>::new()).await?
    }

    async fn register_mocker<S, Mocker>(&self, mocker: Mocker) -> Result<ServiceRef<S>>
    where
        S: ActorService + 'static,
        Mocker: MockHandler<S> + 'static,
    {
        let handler = Box::new(mocker);
        self.send(RegisterMockerRequest::new(handler)).await?
    }

    async fn service_ref_opt<S>(&self) -> Result<Option<ServiceRef<S>>>
    where
        S: ActorService + 'static,
    {
        self.send(ServiceRefRequest::new()).await
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

    async fn check_service_status(&self, service_name: &str) -> Result<ServiceStatus> {
        self.send(CheckServiceStatusRequest::new(service_name.to_string()))
            .await?
    }

    async fn put_shared<T>(&self, value: T) -> Result<()>
    where
        T: Send + Sync + Clone + 'static,
    {
        self.send(PutShardRequest::new(value)).await
    }

    async fn get_shared_opt<T>(&self) -> Result<Option<T>>
    where
        T: Send + Sync + Clone + 'static,
    {
        self.send(GetShardRequest::new()).await
    }

    async fn remove_shared<T>(&self) -> Result<()>
    where
        T: Send + Sync + Clone + 'static,
    {
        self.send(RemoveShardRequest::<T>::new()).await
    }

    async fn shutdown_system(&self) -> Result<()> {
        self.send(SystemCmdRequest {
            cmd: SystemCmd::Shutdown,
        })
        .await?
    }

    async fn shutdown_service<S>(&self) -> Result<()>
    where
        S: ActorService + 'static,
    {
        self.send(SystemCmdRequest {
            cmd: SystemCmd::ShutdownService(S::service_name().to_string()),
        })
        .await?
    }
}
