// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::cal_service::{CalAddRequest, CalAsyncService, CalService};
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceStatus};

pub mod common;

#[stest::test]
async fn test_launch_and_shutdown() {
    let registry = RegistryService::launch();
    registry.shutdown_system().await.unwrap();
}

#[stest::test]
async fn test_basic() {
    let registry = RegistryService::launch();
    let service_ref = registry.register::<CalService>().await.unwrap();

    let service_ref2 = registry.service_ref::<CalService>().await;
    assert!(service_ref2.is_ok());

    let result = service_ref.send(CalAddRequest { value: 1 }).await.unwrap();

    assert_eq!(result, 1);
    let result = service_ref.add(1).await.unwrap();
    assert_eq!(result, 2);

    let services = registry.list_service().await.unwrap();
    // BusService + CounterService = 2
    assert_eq!(2, services.len());

    let status = service_ref.self_status().await;
    assert_eq!(status, ServiceStatus::Started);
    service_ref.stop_self().unwrap();

    let status = service_ref.self_status().await;
    assert_eq!(status, ServiceStatus::Stopped);

    service_ref.start_self().unwrap();

    let status = service_ref.self_status().await;
    assert_eq!(status, ServiceStatus::Started);

    let result = service_ref.add(1).await.unwrap();
    assert_eq!(result, 1);

    let result = service_ref.sub(1).await.unwrap();
    assert_eq!(result, 0);
    registry.shutdown_system().await.unwrap();
}
