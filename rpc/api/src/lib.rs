// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::{BoxFuture, Error};

pub type FutureResult<T> = BoxFuture<Result<T, Error>>;
//pub type FutureResult<T> = Pin<Box<dyn futures::Future<Output = Result<T, Error>> + Send>>;

pub mod account;
pub mod chain;
pub mod contract_api;
pub mod debug;
pub mod dev;
pub mod errors;
pub mod metadata;
pub mod miner;
pub mod network_manager;
pub mod node;
pub mod node_manager;
pub mod pubsub;
pub mod service;
pub mod state;
pub mod sync_manager;
pub mod txpool;
pub mod types;
