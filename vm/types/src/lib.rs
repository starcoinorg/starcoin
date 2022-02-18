// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
//TODO FIXME for fuzzing Arbitrary;
#![allow(clippy::unit_arg)]
mod language_storage_ext;

pub mod account_address;

pub mod gas_schedule {
    pub use move_core_types::gas_schedule::*;
    pub use move_vm_types::gas_schedule::{
        calculate_intrinsic_gas, new_from_instructions, zero_cost_schedule, GasStatus,
    };
    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
    #[repr(u8)]
    pub enum NativeCostIndex {
        SHA2_256 = 0,
        SHA3_256 = 1,
        ED25519_VERIFY = 2,
        ED25519_THRESHOLD_VERIFY = 3,
        BCS_TO_BYTES = 4,
        LENGTH = 5,
        EMPTY = 6,
        BORROW = 7,
        BORROW_MUT = 8,
        PUSH_BACK = 9,
        POP_BACK = 10,
        DESTROY_EMPTY = 11,
        SWAP = 12,
        ED25519_VALIDATE_KEY = 13,
        SIGNER_BORROW = 14,
        CREATE_SIGNER = 15,
        DESTROY_SIGNER = 16,
        EMIT_EVENT = 17,
        BCS_TO_ADDRESS = 18,
        TOKEN_NAME_OF = 19,
        KECCAK_256 = 20,
        RIPEMD160 = 21,
        ECRECOVER = 22,
        U256_FROM_BYTES = 23,
        U256_ADD = 24,
        U256_SUB = 25,
        U256_MUL = 26,
        U256_DIV = 27,
        U256_REM = 28,
        U256_POW = 29,
        VEC_APPEND = 30,
        VEC_REMOVE = 31,
        VEC_REVERSE = 32,
    }

    impl NativeCostIndex {
        //note: should change this value when add new native function.
        pub const NUMBER_OF_NATIVE_FUNCTIONS: usize = 33;
    }
}
pub mod location {
    pub use move_ir_types::location::Loc;
}

pub mod identifier {
    pub use move_core_types::identifier::{IdentStr, Identifier};
}

pub mod language_storage {
    pub use crate::language_storage_ext::parse_module_id;
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
    pub use move_bytecode_verifier::{
        dependencies, script_signature, verify_module, verify_script,
    };
}

pub mod access_path;
pub mod account_config;
pub mod block_metadata;
pub mod event;
pub mod genesis_config;
pub mod on_chain_config;
pub mod on_chain_resource;
pub mod serde_helper;
pub mod sign_message;
pub mod sips;
pub mod time;
pub mod token;
#[cfg(test)]
mod unit_tests;
