// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::cal_service::{CalAsyncService, CalService};
use actix::clock::Duration;
use anyhow::Result;
use futures_timer::Delay;
use log::{error, info};
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler, ServiceRef,
    ServiceRequest,
};

#[derive(Debug, Clone)]
pub struct TaskStatus {
    target_number: u64,
    finished_number: Option<u64>,
}

pub struct LoopTaskService {
    cal_service: ServiceRef<CalService>,
    task_status: Option<TaskStatus>,
}

impl ServiceFactory<Self> for LoopTaskService {
    fn create(ctx: &mut ServiceContext<LoopTaskService>) -> Result<LoopTaskService> {
        Ok(Self {
            cal_service: ctx.service_ref::<CalService>()?.clone(),
            task_status: None,
        })
    }
}

impl ActorService for LoopTaskService {}

#[derive(Debug, Clone)]
pub struct StartTaskEvent {
    pub target_number: u64,
}

impl EventHandler<Self, StartTaskEvent> for LoopTaskService {
    fn handle_event(&mut self, msg: StartTaskEvent, ctx: &mut ServiceContext<LoopTaskService>) {
        if self.task_status.is_some() {
            error!("Exist a running task.");
            return;
        }
        self.task_status.replace(TaskStatus {
            target_number: msg.target_number,
            finished_number: None,
        });
        let cal_service = self.cal_service.clone();
        let self_ref = ctx.self_ref();
        ctx.spawn(async move {
            let mut current = 0;
            loop {
                current += 1;
                if current > msg.target_number {
                    break;
                }
                if let Err(e) = cal_service.add(current).await {
                    error!("Add error: {:?}", e);
                    Delay::new(Duration::from_millis(1000)).await;
                } else {
                    if let Err(e) = self_ref.notify(TaskStatusUpdateEvent {
                        finished_number: current,
                    }) {
                        error!("Notify error: {:?}", e);
                        Delay::new(Duration::from_millis(1000)).await;
                    }
                    Delay::new(Duration::from_millis(100)).await;
                }
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct TaskStatusUpdateEvent {
    finished_number: u64,
}

impl EventHandler<Self, TaskStatusUpdateEvent> for LoopTaskService {
    fn handle_event(
        &mut self,
        msg: TaskStatusUpdateEvent,
        _ctx: &mut ServiceContext<LoopTaskService>,
    ) {
        if let Some(task_status) = self.task_status.as_mut() {
            task_status.finished_number = Some(msg.finished_number);
            if task_status.target_number == msg.finished_number {
                info!("Task finish, clear task status.");
                self.task_status.take();
            }
        };
    }
}

#[derive(Debug, Clone)]
pub struct GetTaskStatus;

impl ServiceRequest for GetTaskStatus {
    type Response = Option<TaskStatus>;
}

impl ServiceHandler<Self, GetTaskStatus> for LoopTaskService {
    fn handle(
        &mut self,
        _msg: GetTaskStatus,
        _ctx: &mut ServiceContext<LoopTaskService>,
    ) -> Option<TaskStatus> {
        self.task_status.clone()
    }
}
