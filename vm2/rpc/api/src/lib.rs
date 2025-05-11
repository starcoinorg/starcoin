// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use jsonrpc_core::{BoxFuture, Error};
pub use starcoin_vm2_abi_decoder::DecodedMoveValue;

pub type FutureResult<T> = BoxFuture<Result<T, Error>>;

pub mod account_api;
pub mod contract_api;
pub mod state_api;
