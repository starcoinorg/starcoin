// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::shared_data_service::{GetRequest, GetService, PutRequest, PutService, DB};
use starcoin_service_registry::{RegistryAsyncService, RegistryService};

pub mod common;

#[stest::test]
async fn test_shared_data() {
    let registry = RegistryService::launch();
    let db = DB::default();
    registry.put_shared(db).await.unwrap();
    let put_ref = registry.registry::<PutService>().await.unwrap();
    let get_ref = registry.registry::<GetService>().await.unwrap();
    put_ref
        .send(PutRequest::new("k1".to_string(), "v1".to_string()))
        .await
        .unwrap();
    let value = get_ref
        .send(GetRequest::new("k1".to_string()))
        .await
        .unwrap();
    assert_eq!(value, Some("v1".to_string()));
    registry.shutdown().await.unwrap();
}
