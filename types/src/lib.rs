// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
// #![feature(vec_remove_item)]
pub mod access_path {
    pub use starcoin_vm_types::access_path::{
        into_inner, random_code, random_resource, AccessPath, DataType,
    };
}

pub mod account_address;

pub mod account_config {
    pub use starcoin_vm_types::account_config::*;
}

pub mod account_state;
pub mod accumulator_info;
pub mod block;

pub mod block_metadata {
    pub use starcoin_vm_types::block_metadata::BlockMetadata;
}

pub mod contract_event {
    pub use starcoin_vm_types::contract_event::*;
}

pub mod error;

pub mod event {
    pub use starcoin_vm_types::event::*;
}

pub mod filter;
pub mod peer_info;
pub mod proof;
pub mod startup_info;
pub mod state_set;
pub mod system_events;

pub mod transaction {
    pub use starcoin_vm_types::transaction::*;
}

pub mod vm_error {
    pub use starcoin_vm_types::vm_error::*;
}

pub mod language_storage {
    pub use starcoin_vm_types::language_storage::{
        ModuleId, ResourceKey, StructTag, TypeTag, CODE_TAG, CORE_CODE_ADDRESS, RESOURCE_TAG,
    };
}

pub mod write_set {
    pub use starcoin_vm_types::write_set::{WriteOp, WriteSet, WriteSetMut};
}

pub use ethereum_types::{H256, U256, U512};

pub const CHAIN_PROTOCOL_NAME: &[u8] = b"/starcoin/chain/1";
pub const TXN_PROTOCOL_NAME: &[u8] = b"/starcoin/txn/1";
pub const BLOCK_PROTOCOL_NAME: &[u8] = b"/starcoin/block/1";
