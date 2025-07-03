// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub use crate::event_info::{ContractEventInfo, StcContractEventInfo};
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_vm2_crypto::hash::PlainCryptoHash;
pub use starcoin_vm_types::contract_event::*;

use crate::event::StcEventKey;
use crate::language_storage::StcTypeTag;
pub(crate) use starcoin_vm2_vm_types::contract_event::ContractEvent as ContractEvent2;

#[derive(Debug, Hash, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum StcContractEvent {
    V1(ContractEvent),
    V2(ContractEvent2),
}

impl From<ContractEvent> for StcContractEvent {
    fn from(event: ContractEvent) -> Self {
        Self::V1(event)
    }
}

impl From<ContractEvent2> for StcContractEvent {
    fn from(event: ContractEvent2) -> Self {
        Self::V2(event)
    }
}

impl StcContractEvent {
    pub fn key(&self) -> StcEventKey {
        match self {
            Self::V1(event) => StcEventKey::V1(*event.key()),
            Self::V2(event) => StcEventKey::V2(event.event_key()),
        }
    }

    pub fn sequence_number(&self) -> u64 {
        match self {
            Self::V1(event) => event.sequence_number(),
            Self::V2(event) => event.sequence_number(),
        }
    }

    pub fn event_data(&self) -> &[u8] {
        match self {
            Self::V1(event) => event.event_data(),
            Self::V2(event) => event.event_data(),
        }
    }

    pub fn type_tag(&self) -> StcTypeTag {
        match self {
            Self::V1(event) => StcTypeTag::V1(event.type_tag().clone()),
            Self::V2(event) => StcTypeTag::V2(event.type_tag().clone()),
        }
    }

    pub fn hash(&self) -> HashValue {
        match self {
            Self::V1(event) => event.crypto_hash(),
            Self::V2(event) => event.crypto_hash(),
        }
    }

    pub fn to_v1(&self) -> Option<ContractEvent> {
        match self {
            Self::V1(event) => Some(event.clone()),
            Self::V2(_) => None,
        }
    }

    pub fn to_v2(&self) -> Option<ContractEvent2> {
        match self {
            Self::V1(_) => None,
            Self::V2(event) => Some(event.clone()),
        }
    }

    pub fn is_v1(&self) -> bool {
        matches!(self, Self::V1(_))
    }

    pub fn is_v2(&self) -> bool {
        matches!(self, Self::V2(_))
    }
}
