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

pub mod transaction_argument {
    pub use move_core_types::transaction_argument::*;
}

pub mod parser {
    use crate::language_storage::TypeTag;
    use anyhow::{format_err, Result};
    pub use move_core_types::parser::parse_type_tags;

    pub fn parse_type_tag(s: &str) -> Result<TypeTag> {
        parse_type_tags(s)?
            .pop()
            .ok_or_else(|| format_err!("parse type fail from {}", s))
    }
}

pub mod transaction_metadata {
    pub use move_vm_types::transaction_metadata::TransactionMetadata;
}

pub mod values {
    pub use move_vm_types::values::*;
}

pub mod loaded_data {
    pub mod types {
        pub use move_vm_types::loaded_data::types::{FatStructType, FatType};
    }

    pub mod runtime_types {
        pub use move_vm_types::loaded_data::runtime_types::{StructType, Type, TypeConverter};
    }
}

pub mod chain_state {
    pub use move_vm_types::chain_state::ChainState;
}

pub mod file_format {
    pub use vm::file_format::*;
}

pub mod views {
    pub use vm::views::*;
}

pub mod data_cache {}

pub mod access {
    pub use vm::access::{ModuleAccess, ScriptAccess};
}

pub mod errors {
    pub use vm::errors::*;
}

pub mod write_set {
    pub use libra_types::write_set::{WriteOp, WriteSet, WriteSetMut};
}

pub mod state_view {
    pub use libra_state_view::StateView;
}

pub mod transaction {
    pub use libra_types::transaction::{ChangeSet, Module, Script};
}

pub mod contract_event {
    pub use libra_types::contract_event::{ContractEvent, ContractEventV0, EventWithProof};
}

pub mod access_path;
pub mod account_config;
pub mod event;
pub mod on_chain_config;
