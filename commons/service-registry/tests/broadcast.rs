// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::broadcast_process_service::{
    BMessage1, BMessage2, BroadcastProcessAsyncService, BroadcastProcessService,
};
use actix::clock::Duration;
use futures_timer::Delay;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::{RegistryAsyncService, RegistryService};

pub mod common;

#[stest::test]
async fn test_broadcast() {
    let registry = RegistryService::launch();
    let service_ref = registry
        .registry::<BroadcastProcessService>()
        .await
        .unwrap();
    let bus_ref = registry.service_ref::<BusService>().await.unwrap();

    //wait subscribe finished.
    Delay::new(Duration::from_millis(500)).await;

    bus_ref.broadcast(BMessage1 {}).await.unwrap();
    bus_ref.broadcast(BMessage1 {}).await.unwrap();
    bus_ref.broadcast(BMessage2 {}).await.unwrap();
    //wait broadcast message processed.
    Delay::new(Duration::from_millis(500)).await;

    let result = service_ref.get_msg_count().await.unwrap();
    assert_eq!(result.b1_count, 2);
    assert_eq!(result.b2_count, 1);

    service_ref.stop_service().unwrap();

    Delay::new(Duration::from_millis(500)).await;

    // broadcast message after service stop will process fail.
    bus_ref.broadcast(BMessage1 {}).await.unwrap();

    service_ref.start_service().unwrap();

    Delay::new(Duration::from_millis(500)).await;

    //broadcast service worked after restart

    bus_ref.broadcast(BMessage1 {}).await.unwrap();

    Delay::new(Duration::from_millis(500)).await;

    let result = service_ref.get_msg_count().await.unwrap();
    assert_eq!(result.b1_count, 1);
    assert_eq!(result.b2_count, 0);

    registry.shutdown().await.unwrap();
}
