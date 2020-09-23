// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::clock::Duration;
use common::cal_service::CalService;
use common::loop_task_service::{GetTaskStatus, LoopTaskService, StartTaskEvent};
use futures_timer::Delay;
use log::debug;
use starcoin_service_registry::{RegistryAsyncService, RegistryService};

pub mod common;

#[stest::test]
async fn test_loop_task() {
    let registry = RegistryService::launch();
    registry.register::<CalService>().await.unwrap();
    let service_ref = registry.register::<LoopTaskService>().await.unwrap();
    let task_status = service_ref.send(GetTaskStatus).await.unwrap();
    assert!(task_status.is_none());
    service_ref
        .notify(StartTaskEvent { target_number: 50 })
        .unwrap();
    let task_status = service_ref.send(GetTaskStatus).await.unwrap();
    assert!(task_status.is_some());
    debug!("task_status: {:?}", task_status);
    loop {
        Delay::new(Duration::from_millis(50)).await;
        let task_status = service_ref.send(GetTaskStatus).await.unwrap();
        debug!("task_status: {:?}", task_status);
        if task_status.is_none() {
            break;
        }
    }
    //start task again.
    service_ref
        .notify(StartTaskEvent { target_number: 10 })
        .unwrap();
    let task_status = service_ref.send(GetTaskStatus).await.unwrap();
    assert!(task_status.is_some());
    debug!("task_status: {:?}", task_status);
    loop {
        Delay::new(Duration::from_millis(50)).await;
        let task_status = service_ref.send(GetTaskStatus).await.unwrap();
        debug!("task_status: {:?}", task_status);
        if task_status.is_none() {
            break;
        }
    }
}
