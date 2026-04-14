// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use futures::future::BoxFuture;
pub use starcoin_vm2_abi_decoder::DecodedMoveValue;
pub type FutureResult<T> = BoxFuture<'static, anyhow::Result<T>>;

pub mod account_api;
pub mod block_info_view2;
pub mod contract_api;
pub mod state_api;
pub mod transaction_view2;
