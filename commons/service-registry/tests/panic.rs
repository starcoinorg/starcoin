// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix_rt::System;
use common::panic_service::{PanicRequest, PanicService, PingRequest};
use futures_timer::Delay;
use starcoin_service_registry::{
    ActorService, RegistryAsyncService, RegistryService, ServiceStatus,
};
use std::time::Duration;

pub mod common;

#[stest::test]
fn test_service_panic() {
    //Fixme Expect the system stop on panic,but this feature not work as expect.
    let mut sys = System::builder()
        .stop_on_panic(true)
        .name("panic_test")
        .build();
    sys.block_on(async {
        let registry = RegistryService::launch();
        let service_ref = registry.register::<PanicService>().await.unwrap();
        service_ref.send(PingRequest).await.unwrap();
        let ping_count = service_ref.send(PingRequest).await.unwrap();
        assert_eq!(2, ping_count);

        let result = service_ref.send(PanicRequest).await;
        assert!(result.is_err());

        let status = registry
            .get_service_status(PanicService::service_name())
            .await
            .unwrap();
        assert_eq!(status, Some(ServiceStatus::Shutdown));
        //wait registry to remove Shutdown service.
        Delay::new(Duration::from_millis(4000)).await;

        let status = registry
            .get_service_status(PanicService::service_name())
            .await
            .unwrap();
        assert_eq!(status, None);
        registry.shutdown_system().await.unwrap();
        System::current().stop();
    });
}
