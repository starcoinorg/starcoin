// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::advance_cal_service::{AdvanceCalAsyncService, AdvanceCalService};
use crate::common::cal_service::CalService;
use futures_timer::Delay;
use starcoin_service_registry::{ActorService, RegistryAsyncService, RegistryService};
use std::time::Duration;

pub mod common;

#[stest::test]
async fn test_service_deps() {
    let registry = RegistryService::launch();
    registry.register::<CalService>().await.unwrap();
    let service_ref = registry.register::<AdvanceCalService>().await.unwrap();

    let values = vec![1, 2, 3, 4, 5];
    let result = service_ref.batch_add(values).await.unwrap();
    assert_eq!(result, 15);

    // restart CalService and ensure  AdvanceCalService is work.
    registry
        .stop_service(CalService::service_name())
        .await
        .unwrap();
    registry
        .start_service(CalService::service_name())
        .await
        .unwrap();

    Delay::new(Duration::from_millis(500)).await;
    let values = vec![1, 2, 3, 4, 5];
    let result = service_ref.batch_add(values).await.unwrap();
    assert_eq!(result, 15);

    //restart AdvanceCalService, Cal state is keep in CalService
    registry
        .stop_service(AdvanceCalService::service_name())
        .await
        .unwrap();
    registry
        .start_service(AdvanceCalService::service_name())
        .await
        .unwrap();

    Delay::new(Duration::from_millis(500)).await;

    let values = vec![1];
    let result = service_ref.batch_add(values).await.unwrap();
    assert_eq!(result, 16);

    registry.shutdown_system().await.unwrap();
}
