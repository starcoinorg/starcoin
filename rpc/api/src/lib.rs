// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::Error;

pub type FutureResult<T> = Box<dyn jsonrpc_core::futures::Future<Item = T, Error = Error> + Send>;

pub mod account;
pub mod chain;
pub mod debug;
pub mod dev;
pub mod errors;
pub mod metadata;
pub mod miner;
pub mod node;
pub mod node_manager;
pub mod pubsub;
pub mod service;
pub mod state;
pub mod txpool;
pub mod types;
