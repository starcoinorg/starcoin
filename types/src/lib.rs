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

pub mod contract_event;

// pub mod time {
//     pub use starcoin_vm_types::time::*;
// }

pub mod error;

pub mod event {
    pub use starcoin_vm_types::event::*;

    use serde::{Deserialize, Serialize};
    use starcoin_vm2_vm_types::event::EventKey as EventKey2;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum StcEventKey {
        V1(EventKey),
        V2(EventKey2),
    }
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

pub mod transaction;

//TODO rename or remove this mode.
pub mod vm_error {
    pub use starcoin_vm_types::vm_status::*;
}

pub mod language_storage {
    use serde::{Deserialize, Serialize};
    pub use starcoin_vm_types::language_storage::{
        ModuleId, ResourceKey, StructTag, TypeTag, CODE_TAG, CORE_CODE_ADDRESS, RESOURCE_TAG,
    };

    use starcoin_vm2_vm_types::language_storage::TypeTag as TypeTag2;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
    pub enum StcTypeTag {
        V1(TypeTag),
        V2(TypeTag2),
    }

    impl From<TypeTag> for StcTypeTag {
        fn from(tag: TypeTag) -> Self {
            StcTypeTag::V1(tag)
        }
    }

    impl From<TypeTag2> for StcTypeTag {
        fn from(tag: TypeTag2) -> Self {
            StcTypeTag::V2(tag)
        }
    }
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
