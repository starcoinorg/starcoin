use actix::prelude::*;
use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub trait SyncTaskAction: Send + Sync {
    fn activate(&self) {}
}

pub enum SyncTaskRequest {
    ACTIVATE(),
}

impl Message for SyncTaskRequest {
    type Result = Result<SyncTaskResponse>;
}

pub enum SyncTaskResponse {
    None,
}

#[derive(Message, Clone, Debug, PartialEq, Eq, Hash)]
#[rtype(result = "Result<()>")]
pub enum SyncTaskType {
    BLOCK,
    STATE,
}

#[derive(Clone)]
pub struct SyncTask {
    tasks: Arc<RwLock<HashMap<SyncTaskType, Box<dyn SyncTaskAction>>>>,
}

impl SyncTask {
    pub fn new_empty() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn push_task(&self, task_type: SyncTaskType, task: Box<dyn SyncTaskAction>) -> bool {
        let mut write = self.tasks.write();
        if !write.contains_key(&task_type) {
            write.insert(task_type, task);
            return true;
        }
        false
    }

    pub fn is_finish(&self) -> bool {
        self.tasks.read().is_empty()
    }

    pub fn drop_task(&self, task_type: &SyncTaskType) {
        self.tasks.write().remove(task_type);
    }

    pub fn activate_tasks(&self) {
        let read = self.tasks.read();
        for task in read.values() {
            task.activate();
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncTaskState {
    NotReady,
    Ready,
    Syncing,
    Failed,
    Finish,
}

impl SyncTaskState {
    pub fn is_ready(&self) -> bool {
        self != &SyncTaskState::NotReady
    }

    pub fn is_failed(&self) -> bool {
        self == &SyncTaskState::Failed
    }

    pub fn is_finish(&self) -> bool {
        self == &SyncTaskState::Finish
    }
}
