// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::shared_data_service::{GetRequest, GetService, PutRequest, PutService, DB};
use futures_timer::Delay;
use starcoin_service_registry::{ActorService, RegistryAsyncService, RegistryService};
use std::sync::Arc;
use std::time::Duration;

pub mod common;

#[stest::test]
async fn test_shared_data() {
    let registry = RegistryService::launch();
    let db = DB::default();
    registry.put_shared(Arc::new(db)).await.unwrap();
    let put_ref = registry.register::<PutService>().await.unwrap();
    let get_ref = registry.register::<GetService>().await.unwrap();
    put_ref
        .send(PutRequest::new("k1".to_string(), "v1".to_string()))
        .await
        .unwrap();
    let value = get_ref
        .send(GetRequest::new("k1".to_string()))
        .await
        .unwrap();
    assert_eq!(value, Some("v1".to_string()));

    //restart Service do not lost data.
    registry
        .stop_service(GetService::service_name())
        .await
        .unwrap();
    registry
        .start_service(GetService::service_name())
        .await
        .unwrap();

    Delay::new(Duration::from_millis(500)).await;

    let value = get_ref
        .send(GetRequest::new("k1".to_string()))
        .await
        .unwrap();
    assert_eq!(value, Some("v1".to_string()));

    registry.shutdown_system().await.unwrap();
}
