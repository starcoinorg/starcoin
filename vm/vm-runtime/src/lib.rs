// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#[macro_use]
extern crate prometheus;

pub mod data_cache;
pub mod metrics;
pub mod starcoin_vm;
pub use move_vm_runtime::move_vm;
