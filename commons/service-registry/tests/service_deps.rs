// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::advance_cal_service::{AdvanceCalAsyncService, AdvanceCalService};
use crate::common::cal_service::CalService;
use starcoin_service_registry::{RegistryAsyncService, RegistryService};

pub mod common;

#[stest::test]
async fn test_service_deps() {
    let registry = RegistryService::launch();
    registry.registry::<CalService>().await.unwrap();
    let service_ref = registry.registry::<AdvanceCalService>().await.unwrap();

    let values = vec![1, 2, 3, 4, 5];
    let result = service_ref.batch_add(values).await.unwrap();
    assert_eq!(result, 15);
    registry.shutdown().await.unwrap();
}
