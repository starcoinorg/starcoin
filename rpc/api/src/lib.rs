// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::Error;

pub type FutureResult<T> = Box<dyn jsonrpc_core::futures::Future<Item = T, Error = Error> + Send>;

pub mod chain;
pub mod debug;
pub mod node;
pub mod state;
pub mod txpool;
pub mod wallet;
