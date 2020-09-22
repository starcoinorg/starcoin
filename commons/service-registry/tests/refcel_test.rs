// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::refcell_service::{CalAsyncService, CalService};
use starcoin_service_registry::{RegistryAsyncService, RegistryService};

pub mod common;

#[stest::test]
async fn test_refcell_cal() {
    let registry = RegistryService::launch();
    let service_ref = registry.register::<CalService>().await.unwrap();

    let result = service_ref.add(1).await.unwrap();
    assert_eq!(result, 1);

    let result = service_ref.add(1).await.unwrap();
    assert_eq!(result, 2);

    let result = service_ref.sub(1).await.unwrap();
    assert_eq!(result, 1);
    registry.shutdown_system().await.unwrap();
}
