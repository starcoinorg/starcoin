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
pub mod blockhash;
pub mod compact_block;
pub mod consensus_header;

pub mod block_metadata {
    use crate::genesis_config::ChainId;
    pub use starcoin_vm2_vm_types::block_metadata::BlockMetadata;
    pub use starcoin_vm_types::block_metadata::BlockMetadata as BlockMetadataLegacy;

    /// Convert VM2 BlockMetadata to VM1 BlockMetadata
    pub fn from(meta: BlockMetadata) -> BlockMetadataLegacy {
        let author_bytes = meta.author().to_vec();
        let mut author_array = [0u8; 16];
        author_array.copy_from_slice(&author_bytes[..16]);
        let author_v1 = crate::account_address::AccountAddress::from(author_array);

        // Get parent_gas_used from VM2 metadata
        let (parent_hash, timestamp, _, _uncles, number, chain_id, parent_gas_used, _, _) =
            meta.into_inner();

        BlockMetadataLegacy::new(
            parent_hash,
            timestamp,
            author_v1,
            None, // author_auth_key
            0,    // Always set uncles to 0 for VM1
            number,
            ChainId::new(chain_id.id()),
            parent_gas_used,
        )
    }
}

pub mod contract_event;

// pub mod time {
//     pub use starcoin_vm_types::time::*;
// }

pub mod error;

pub mod event {
    pub use starcoin_vm_types::event::*;

    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use starcoin_vm2_vm_types::event::EventKey as EventKey2;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
    pub enum StcEventKey {
        V1(EventKey),
        V2(EventKey2),
    }

    impl TryFrom<StcEventKey> for EventKey {
        type Error = anyhow::Error;

        fn try_from(value: StcEventKey) -> Result<Self, Self::Error> {
            match value {
                StcEventKey::V1(key) => Ok(key),
                StcEventKey::V2(_key) => anyhow::bail!("V2 EventKey cannot be convert to V1"),
            }
        }
    }

    impl From<EventKey> for StcEventKey {
        fn from(key: EventKey) -> Self {
            StcEventKey::V1(key)
        }
    }

    impl From<EventKey2> for StcEventKey {
        fn from(key: EventKey2) -> Self {
            StcEventKey::V2(key)
        }
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
    use std::str::FromStr;

    pub use starcoin_vm2_vm_types::language_storage::{ModuleId as ModuleId2, TypeTag as TypeTag2};

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

    impl FromStr for StcTypeTag {
        type Err = anyhow::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if let Ok(tag) = TypeTag2::from_str(s) {
                Ok(StcTypeTag::V2(tag))
            } else if let Ok(tag) = TypeTag::from_str(s) {
                Ok(StcTypeTag::V1(tag))
            } else {
                Err(anyhow::anyhow!("Invalid TypeTag string: {}", s))
            }
        }
    }

    impl StcTypeTag {
        pub fn to_canonical_string(&self) -> String {
            match self {
                StcTypeTag::V1(tag) => tag.to_canonical_string(),
                StcTypeTag::V2(tag) => tag.to_canonical_string(),
            }
        }

        pub fn as_v1(&self) -> Option<&TypeTag> {
            match self {
                StcTypeTag::V1(tag) => Some(tag),
                StcTypeTag::V2(_) => None,
            }
        }

        pub fn as_v2(&self) -> Option<&TypeTag2> {
            match self {
                StcTypeTag::V1(_) => None,
                StcTypeTag::V2(tag) => Some(tag),
            }
        }
    }
}

pub mod identifier {
    pub use starcoin_vm2_vm_types::identifier::Identifier as Identifier2;
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
pub mod table;

pub mod proof {
    pub use forkable_jellyfish_merkle::proof::SparseMerkleProof;
}
