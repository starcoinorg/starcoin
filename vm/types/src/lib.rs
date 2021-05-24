// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
//TODO FIXME for fuzzing Arbitrary;
#![allow(clippy::unit_arg)]
mod language_storage_ext;

pub mod account_address;

pub mod gas_schedule {
    pub use move_core_types::gas_schedule::*;
    pub use move_vm_types::gas_schedule::*;
}
pub mod location {
    pub use move_ir_types::location::Loc;
}

pub mod identifier {
    pub use move_core_types::identifier::{IdentStr, Identifier};
}

pub mod language_storage {
    pub use crate::language_storage_ext::FunctionId;
    pub use move_core_types::language_storage::{
        ModuleId, ResourceKey, StructTag, TypeTag, CODE_TAG, CORE_CODE_ADDRESS, RESOURCE_TAG,
    };
}

pub mod move_resource;

pub mod transaction_argument {
    pub use move_core_types::transaction_argument::*;
}

pub mod parser {
    use crate::language_storage::TypeTag;
    use anyhow::{bail, Result};
    use move_core_types::language_storage::StructTag;
    pub use move_core_types::parser::{
        parse_transaction_argument, parse_type_tag, parse_type_tags,
    };

    pub fn parse_struct_tag(s: &str) -> Result<StructTag> {
        let type_tag = parse_type_tag(s)?;
        match type_tag {
            TypeTag::Struct(st) => Ok(st),
            t => bail!("expect a struct tag, found: {:?}", t),
        }
    }
}

#[cfg(any(test, feature = "fuzzing"))]
pub mod proptest_types;

pub mod transaction_metadata;

pub mod value {
    pub use move_core_types::value::*;
}

pub mod values {
    pub use move_vm_types::values::*;
}

pub mod loaded_data {
    pub mod runtime_types {
        pub use move_vm_types::loaded_data::runtime_types::{StructType, Type};
    }
}

pub mod data_store {
    pub use move_vm_types::data_store::DataStore;
}

pub mod file_format {
    pub use vm::file_format::*;
}

pub mod normalized {
    pub use vm::normalized::*;
}

pub mod compatibility {
    pub use vm::compatibility::*;
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
    pub use vm::IndexKind;
}

pub mod write_set;

pub mod state_view;

pub mod transaction;

pub mod contract_event;

pub mod vm_status {
    pub use move_core_types::vm_status::*;
    pub mod sub_status {
        pub use move_core_types::vm_status::sub_status::*;
    }
}
pub mod effects {
    pub use move_core_types::effects::*;
}
pub mod bytecode_verifier {
    pub use bytecode_verifier::{dependencies, script_signature, verify_module, verify_script};
}

pub mod access_path;
pub mod account_config;
pub mod block_metadata;
pub mod event;
pub mod genesis_config;
pub mod on_chain_config;
pub mod on_chain_resource;
pub mod receipt_identifier;
pub mod serde_helper;
pub mod sign_message;
pub mod sips;
pub mod time;
pub mod token;

#[cfg(test)]
mod unit_tests;
