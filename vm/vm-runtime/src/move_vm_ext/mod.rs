// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! MoveVM and Session wrapped, to make sure Starcoin natives and extensions are always installed and
//! taken care of after session finish.
mod resolver;
mod session;
mod vm;

mod warm_vm_cache;

pub(crate) mod write_op_converter;

pub use crate::move_vm_ext::{
    resolver::{AsExecutorView, StarcoinMoveResolver},
    session::{SessionExt, SessionId},
    vm::{GenesisMoveVM, MoveVmExt},
};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{
    account_address::AccountAddress, language_storage::StructTag, vm_status::StatusCode,
};
use starcoin_vm_types::state_store::state_key::StateKey;

pub(crate) fn resource_state_key(
    address: &AccountAddress,
    tag: &StructTag,
) -> PartialVMResult<StateKey> {
    StateKey::resource(address, tag).map_err(|e| {
        PartialVMError::new(StatusCode::VALUE_SERIALIZATION_ERROR)
            .with_message(format!("Failed to serialize struct tag: {}", e))
    })
}
