// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::advance_cal_service::{AdvanceCalAsyncService, AdvanceCalService};
use crate::common::cal_service::{CalAsyncService, CalService};
use log::debug;
use starcoin_service_registry::mocker::{mock, MockHandler};
use starcoin_service_registry::{RegistryAsyncService, RegistryService, ServiceContext};
use std::any::Any;
use std::sync::{Arc, Mutex};

pub mod common;

#[stest::test]
async fn test_mock_service_by_fn() {
    let registry = RegistryService::launch();
    let mocker = mock(|request, _ctx| {
        debug!("Mocker receive request: {:?}", request);
        //always return zero.
        Box::new(0u64)
    });

    let cal_service_ref = registry.register_mocker(mocker).await.unwrap();
    let result = cal_service_ref.add(1).await.unwrap();
    assert_eq!(result, 0);
    let service_ref = registry.register::<AdvanceCalService>().await.unwrap();

    let values = vec![1, 2, 3, 4, 5];
    let result = service_ref.batch_add(values).await.unwrap();
    assert_eq!(result, 0);
    registry.shutdown_system().await.unwrap();
}

#[derive(Default, Clone)]
struct MockCalService {
    counter: Arc<Mutex<u64>>,
}

impl MockHandler<CalService> for MockCalService {
    fn handle(
        &mut self,
        request: Box<dyn Any>,
        _ctx: &mut ServiceContext<CalService>,
    ) -> Box<dyn Any> {
        debug!("Mocker receive request: {:?}", request);
        *self.counter.lock().unwrap() += 1;
        //always return zero.
        Box::new(0u64)
    }
}

#[stest::test]
async fn test_mock_service() {
    let registry = RegistryService::launch();
    let mocker = MockCalService::default();

    let cal_service_ref = registry.register_mocker(mocker.clone()).await.unwrap();
    let result = cal_service_ref.add(1).await.unwrap();
    assert_eq!(result, 0);
    assert_eq!(*mocker.counter.lock().unwrap(), 1);

    let service_ref = registry.register::<AdvanceCalService>().await.unwrap();

    let values = vec![1, 2, 3, 4, 5];
    let result = service_ref.batch_add(values).await.unwrap();
    assert_eq!(result, 0);
    registry.shutdown_system().await.unwrap();
}
