// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::Error;

pub type FutureResult<T> = Box<dyn jsonrpc_core::futures::Future<Item = T, Error = Error> + Send>;

//pub struct RpcMessage(pub jsonrpc_core::)

pub mod status;
pub mod txpool;
