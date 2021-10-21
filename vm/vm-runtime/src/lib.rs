// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod data_cache;
pub mod metrics;
pub mod natives;
pub mod starcoin_vm;
pub use move_vm_runtime::move_vm;
mod access_path_cache;
mod errors;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::language_storage::StructTag;

/// Get the AccessPath to a resource stored under `address` with type name `tag`
fn create_access_path(address: AccountAddress, tag: StructTag) -> AccessPath {
    AccessPath::resource_access_path(address, tag)
}
