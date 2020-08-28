// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use crate::service_registry::{ServiceRegistry, ServiceStatus, SystemService};
use actix::{Actor, ActorContext, Context, Handler};
use anyhow::Result;
use starcoin_bus::BusActor;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_types::system_events::ActorStop;
use std::sync::Arc;

struct MockService {}

impl MockService {
    pub fn new(_registry: &ServiceRegistry) -> Result<Self> {
        Ok(Self {})
    }
}

impl Actor for MockService {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("mock service started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("mock service stopped");
    }
}

impl SystemService for MockService {}

impl Handler<ActorStop> for MockService {
    type Result = ();

    fn handle(&mut self, _msg: ActorStop, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop()
    }
}

#[stest::test]
async fn test_start_and_stop() {
    let config = Arc::new(NodeConfig::random_for_test());
    let bus = BusActor::launch();
    let registry = ServiceRegistry::new(config, bus);
    let _address = registry.registry(MockService::new).unwrap();

    let services = registry.list();
    assert_eq!(1, services.len());

    let info = registry.service_info::<MockService>();
    assert!(info.is_some());
    assert_eq!(info.unwrap().status, ServiceStatus::Started);
    //registry.start_by_name(MockService::service_name()).unwrap();
    registry.stop::<MockService>().unwrap();

    let info = registry.service_info::<MockService>();
    assert!(info.is_some());
    assert_eq!(info.unwrap().status, ServiceStatus::Stopped);

    registry.start::<MockService>().unwrap();

    let info = registry.service_info::<MockService>();
    assert!(info.is_some());
    assert_eq!(info.unwrap().status, ServiceStatus::Started);
}
