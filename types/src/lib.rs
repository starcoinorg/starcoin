// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![deny(clippy::arithmetic_side_effects)]

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

pub mod multi_state;

#[allow(clippy::too_many_arguments)]
pub mod block;
pub mod compact_block;

pub mod block_metadata {
    pub use starcoin_vm_types::block_metadata::BlockMetadata;
}

pub mod contract_event {
    pub use crate::event_info::ContractEventInfo;
    pub use starcoin_vm_types::contract_event::*;
}

// pub mod time {
//     pub use starcoin_vm_types::time::*;
// }

pub mod error;

pub mod event {
    pub use starcoin_vm_types::event::*;
}

pub mod filter;

#[cfg(any(test, feature = "fuzzing"))]
pub mod proptest_types;

pub mod sign_message {
    pub use starcoin_vm_types::sign_message::*;
}
pub mod startup_info;
pub mod state_set;
pub mod system_events;

pub mod multi_transaction;

pub mod transaction {
    use serde::{Deserialize, Serialize};
    use starcoin_crypto::HashValue;
    pub use starcoin_vm_types::transaction::*;
    use std::ops::Deref;

    /// try to parse_transaction_argument and auto convert no address 0x hex string to Move's vector<u8>
    pub fn parse_transaction_argument_advance(s: &str) -> anyhow::Result<TransactionArgument> {
        let arg = match parse_transaction_argument(s) {
            Ok(arg) => arg,
            Err(e) => {
                //auto convert 0xxx to vector<u8>
                match s.strip_prefix("0x") {
                    Some(stripped) => TransactionArgument::U8Vector(hex::decode(stripped)?),
                    None => return Err(e),
                }
            }
        };
        Ok(arg)
    }

    /// `RichTransactionInfo` is a wrapper of `TransactionInfo` with more info,
    /// such as `block_id`, `block_number` which is the block that include the txn producing the txn info.
    /// We cannot put the block_id into txn_info, because txn_info is accumulated into block header.
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct RichTransactionInfo {
        pub block_id: HashValue,
        pub block_number: u64,
        pub transaction_info: TransactionInfo,
        /// Transaction index in block
        pub transaction_index: u32,
        /// Transaction global index in chain, equivalent to transaction accumulator's leaf index
        pub transaction_global_index: u64,
    }

    impl Deref for RichTransactionInfo {
        type Target = TransactionInfo;

        fn deref(&self) -> &Self::Target {
            &self.transaction_info
        }
    }

    impl RichTransactionInfo {
        pub fn new(
            block_id: HashValue,
            block_number: u64,
            transaction_info: TransactionInfo,
            transaction_index: u32,
            transaction_global_index: u64,
        ) -> Self {
            Self {
                block_id,
                block_number,
                transaction_info,
                transaction_index,
                transaction_global_index,
            }
        }

        pub fn block_id(&self) -> HashValue {
            self.block_id
        }

        pub fn txn_info(&self) -> &TransactionInfo {
            &self.transaction_info
        }
    }
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
