// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod account_address {
    pub use move_core_types::account_address::AccountAddress;
}

pub mod gas_schedule {
    pub use move_core_types::gas_schedule::*;
    pub use move_vm_types::gas_schedule::*;
}

pub mod identifier {
    pub use move_core_types::identifier::{IdentStr, Identifier};
}

pub mod language_storage {
    pub use move_core_types::language_storage::{
        ModuleId, ResourceKey, StructTag, TypeTag, CODE_TAG, CORE_CODE_ADDRESS, RESOURCE_TAG,
    };
}

pub mod move_resource {
    pub use move_core_types::move_resource::MoveResource;
}

pub mod transaction_metadata {
    pub use move_vm_types::transaction_metadata::TransactionMetadata;
}

pub mod values {
    pub use move_vm_types::values::*;
}

pub mod chain_state {
    pub use move_vm_types::chain_state::ChainState;
}

pub mod file_format {
    pub use vm::file_format::*;
}

pub mod access {
    pub use vm::access::{ModuleAccess, ScriptAccess};
}

pub mod errors {
    pub use vm::errors::*;
}
