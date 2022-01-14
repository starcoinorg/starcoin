// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![deny(clippy::integer_arithmetic)]

mod event_info;

pub mod access_path {
    pub use starcoin_vm_types::access_path::{AccessPath, DataPath, DataType};
}

pub mod account_address;

pub use starcoin_uint::*;

pub mod account_config {
    pub use starcoin_vm_types::account_config::*;
}

pub mod account;

pub mod account_state;

#[allow(clippy::too_many_arguments)]
pub mod block;
pub mod cmpact_block;

pub mod block_metadata {
    pub use starcoin_vm_types::block_metadata::BlockMetadata;
}

pub mod contract_event {
    pub use crate::event_info::ContractEventInfo;
    pub use starcoin_vm_types::contract_event::*;
}

pub mod time {
    pub use starcoin_vm_types::time::*;
}

pub mod error;

pub mod event {
    pub use starcoin_vm_types::event::*;
}

pub mod filter;
pub mod peer_info;

#[cfg(any(test, feature = "fuzzing"))]
pub mod proptest_types;

pub mod sign_message {
    pub use starcoin_vm_types::sign_message::*;
}
pub mod startup_info;
pub mod state_set;
pub mod system_events;

pub mod transaction {
    pub use starcoin_vm_types::transaction::*;
}

//TODO rename or remove this mode.
pub mod vm_error {
    pub use starcoin_vm_types::vm_status::*;
}

pub mod language_storage {
    pub use starcoin_vm_types::language_storage::{
        ModuleId, ResourceKey, StructTag, TypeTag, CODE_TAG, CORE_CODE_ADDRESS, RESOURCE_TAG,
    };
}

pub mod identifier {
    pub use starcoin_vm_types::identifier::{IdentStr, Identifier};
}

pub mod write_set {
    pub use starcoin_vm_types::write_set::{WriteOp, WriteSet, WriteSetMut};
}

pub mod genesis_config {
    pub use starcoin_vm_types::genesis_config::*;
}

pub mod stress_test;
pub mod sync_status;

pub mod proof {
    pub use forkable_jellyfish_merkle::proof::SparseMerkleProof;
}
