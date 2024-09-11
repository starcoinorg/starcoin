// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
//TODO FIXME for fuzzing Arbitrary;
#![allow(clippy::unit_arg)]
mod language_storage_ext;

pub mod account_address;
pub mod gas_schedule;

pub mod move_any;
pub mod location {
    pub use move_ir_types::location::Loc;
}

pub mod identifier {
    pub use move_core_types::identifier::{IdentStr, Identifier};
}

pub mod language_storage {
    pub use crate::language_storage_ext::{parse_module_id, FunctionId};
    pub use move_core_types::language_storage::{
        ModuleId, ResourceKey, StructTag, TypeTag, CODE_TAG, CORE_CODE_ADDRESS, RESOURCE_TAG,
    };

    /// check the filter TypeTag is match with the Target, if the filter and target both are StructTag, call `struct_tag_match`, otherwise, same as `==`
    pub fn type_tag_match(filter: &TypeTag, target: &TypeTag) -> bool {
        if let (TypeTag::Struct(filter), TypeTag::Struct(target)) = (filter, target) {
            struct_tag_match(filter, target)
        } else {
            filter == target
        }
    }

    /// check the filter StructTag is match with the target.
    pub fn struct_tag_match(filter: &StructTag, target: &StructTag) -> bool {
        if filter == target {
            return true;
        }

        if filter.address != target.address
            || filter.module != target.module
            || filter.name != target.name
        {
            return false;
        }

        if filter.type_args.is_empty() {
            return true;
        }

        if filter.type_args.len() != target.type_args.len() {
            return false;
        }

        for (filter_type_tag, target_type_tag) in
            std::iter::zip(filter.type_args.clone(), target.type_args.clone())
        {
            if !type_tag_match(&filter_type_tag, &target_type_tag) {
                return false;
            }
        }
        true
    }

    #[cfg(test)]
    mod tests {
        use crate::parser::parse_struct_tag;
        #[test]
        fn test_struct_tag_match() {
            let test_cases = vec![
                (
                    "0x1::Account::Balance<0x1::STC::STC>",
                    "0x1::Account::Balance<0x1::STC::STC>",
                    true,
                ),
                (
                    "0x1::Account::Balance<0x1::STC::STC>",
                    "0x2::Account::Balance<0x1::STC::STC>",
                    false,
                ),
                (
                    "0x1::Account::Balance<0x1::STC::STC>",
                    "0x1::Account2::Balance<0x1::STC::STC>",
                    false,
                ),
                (
                    "0x1::Account::Balance<0x1::STC::STC>",
                    "0x1::Account::Balance2<0x1::STC::STC>",
                    false,
                ),
                (
                    "0x1::Account::Balance<0x1::STC::STC>",
                    "0x1::Account::Balance<0x1::XToken::XToken>",
                    false,
                ),
                (
                    "0x1::Account::Balance",
                    "0x1::Account::Balance<0x1::STC::STC>",
                    true,
                ),
                (
                    "0x1::Test::TokenPair",
                    "0x1::Test::TokenPair<0x1::STC::STC,0x1::USD::USD>",
                    true,
                ),
                (
                    "0x1::Test::TokenPair<0x1::STC::STC>",
                    "0x1::Test::TokenPair<0x1::STC::STC,0x1::USD::USD>",
                    false,
                ),
                (
                    "0x1::Test::TypeWrapper<0x1::Test::TypeNest>",
                    "0x1::Test::TypeWrapper<0x1::Test::TypeNest<0x1::X::X>>",
                    true,
                ),
                (
                    "0x1::Test::TypeWrapper<u64>",
                    "0x1::Test::TypeWrapper<u64>",
                    true,
                ),
                (
                    "0x1::Test::TypeWrapper",
                    "0x1::Test::TypeWrapper<u64>",
                    true,
                ),
                (
                    "0x1::Test::TypeWrapper<u64>",
                    "0x1::Test::TypeWrapper<bool>",
                    false,
                ),
            ];
            for case in test_cases {
                let filter = parse_struct_tag(case.0).unwrap();
                let target = parse_struct_tag(case.1).unwrap();
                assert_eq!(
                    crate::language_storage::struct_tag_match(&filter, &target),
                    case.2,
                    "{:?} failed",
                    case
                );
            }
        }
    }
}

pub mod move_resource;

pub mod transaction_argument {
    pub use move_core_types::transaction_argument::*;
}

pub mod parser {
    pub use move_core_types::parser::{
        parse_struct_tag, parse_transaction_argument, parse_type_tag, parse_type_tags,
    };
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

pub use account_address::AccountAddress as PeerId;

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
pub mod fee_statement;
pub mod genesis_config;
pub mod on_chain_config;
pub mod on_chain_resource;
pub mod serde_helper;
pub mod sign_message;
pub mod sips;
pub mod state_store;
pub mod time;
pub mod token;
pub mod utility_coin;
#[cfg(test)]
mod unit_tests;

pub mod sub_status {
    // Native Function Error sub-codes
    pub const NFE_VECTOR_ERROR_BASE: u64 = 0;
    // Failure in BCS deserialization
    pub const NFE_BCS_SERIALIZATION_FAILURE: u64 = 0x1C5;

    pub mod unknown_invariant_violation {
        // Paranoid Type checking returns an error
        pub const EPARANOID_FAILURE: u64 = 0x1;

        // Reference safety checks failure
        pub const EREFERENCE_COUNTING_FAILURE: u64 = 0x2;
    }

    pub mod type_resolution_failure {
        // User provided typetag failed to load.
        pub const EUSER_TYPE_LOADING_FAILURE: u64 = 0x1;
    }
}


