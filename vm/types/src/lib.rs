// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod account_address {
    pub use move_core_types::account_address::AccountAddress;

    use crate::transaction::authenticator::AuthenticationKey;
    use starcoin_crypto::ed25519::Ed25519PublicKey;

    pub fn from_public_key(public_key: &Ed25519PublicKey) -> AccountAddress {
        AuthenticationKey::ed25519(public_key).derived_address()
    }
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
    pub use move_core_types::parser::{parse_transaction_argument, parse_type_tags};

    pub fn parse_type_tag(s: &str) -> Result<TypeTag> {
        parse_type_tags(s)?
            .pop()
            .ok_or_else(|| format_err!("parse type fail from {}", s))
    }
}

pub mod transaction_metadata;

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

pub mod data_store {
    pub use move_vm_types::data_store::DataStore;
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

pub mod transaction;

pub mod contract_event {
    pub use libra_types::contract_event::{
        ContractEvent, ContractEventHasher, ContractEventV0, EventWithProof,
    };
}

pub mod vm_error {
    pub use libra_types::vm_error::*;

    pub mod sub_status {
        pub use libra_types::vm_error::sub_status::*;
    }
}

pub mod bytecode_verifier {
    pub use bytecode_verifier::{VerifiedModule, VerifiedScript};
}

pub mod access_path;
pub mod account_config;
pub mod block_metadata;
pub mod event;
pub mod on_chain_config;
