// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_service_registry::{
    ActorService, ServiceContext, ServiceFactory, ServiceHandler, ServiceRequest,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
#[allow(clippy::upper_case_acronyms)]
pub struct DB {
    data: Mutex<HashMap<String, String>>,
}

impl DB {
    pub fn insert(&self, key: String, value: String) {
        self.data.lock().unwrap().insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.data.lock().unwrap().get(key).cloned()
    }
}

pub struct GetService {
    db: Arc<DB>,
}

impl ActorService for GetService {}

impl ServiceFactory<Self> for GetService {
    fn create(ctx: &mut ServiceContext<GetService>) -> Result<GetService> {
        Ok(Self {
            db: ctx.get_shared::<Arc<DB>>()?,
        })
    }
}

#[derive(Debug)]
pub struct GetRequest {
    key: String,
}

impl GetRequest {
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

impl ServiceRequest for GetRequest {
    type Response = Option<String>;
}

impl ServiceHandler<Self, GetRequest> for GetService {
    fn handle(&mut self, msg: GetRequest, _ctx: &mut ServiceContext<GetService>) -> Option<String> {
        self.db.get(msg.key.as_str())
    }
}

pub struct PutService {
    db: Arc<DB>,
}

impl ActorService for PutService {}

impl ServiceFactory<Self> for PutService {
    fn create(ctx: &mut ServiceContext<PutService>) -> Result<PutService> {
        Ok(Self {
            db: ctx.get_shared::<Arc<DB>>()?,
        })
    }
}

#[derive(Debug)]
pub struct PutRequest {
    key: String,
    value: String,
}

impl PutRequest {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

impl ServiceRequest for PutRequest {
    type Response = ();
}

impl ServiceHandler<Self, PutRequest> for PutService {
    fn handle(&mut self, msg: PutRequest, _ctx: &mut ServiceContext<PutService>) {
        self.db.insert(msg.key, msg.value);
    }
}
