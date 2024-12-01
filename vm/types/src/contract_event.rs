// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config::{block::NewBlockEvent, deposit::DepositEvent, withdraw::WithdrawEvent},
    event::EventKey,
    language_storage::TypeTag,
    move_resource::MoveResource,
    on_chain_config::new_epoch_event_key,
};
use anyhow::{bail, Error, Result};
use move_core_types::move_resource::MoveStructType;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, CryptoHasher};
use std::str::FromStr;

/// Support versioning of the data structure.
#[derive(Hash, Clone, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub enum ContractEvent {
    V1(ContractEventV1),
    V2(ContractEventV2),
}

impl ContractEvent {
    pub fn new_v1(
        key: EventKey,
        sequence_number: u64,
        type_tag: TypeTag,
        event_data: Vec<u8>,
    ) -> Self {
        ContractEvent::V1(ContractEventV1::new(
            key,
            sequence_number,
            type_tag,
            event_data,
        ))
    }

    pub fn new_v2(type_tag: TypeTag, event_data: Vec<u8>) -> Self {
        ContractEvent::V2(ContractEventV2::new(type_tag, event_data))
    }

    pub fn new_v2_with_type_tag_str(type_tag_str: &str, event_data: Vec<u8>) -> Self {
        ContractEvent::V2(ContractEventV2::new(
            TypeTag::from_str(type_tag_str).unwrap(),
            event_data,
        ))
    }

    pub fn event_key(&self) -> Option<&EventKey> {
        match self {
            ContractEvent::V1(event) => Some(event.key()),
            ContractEvent::V2(_event) => None,
        }
    }

    pub fn event_data(&self) -> &[u8] {
        match self {
            ContractEvent::V1(event) => event.event_data(),
            ContractEvent::V2(event) => event.event_data(),
        }
    }

    pub fn type_tag(&self) -> &TypeTag {
        match self {
            ContractEvent::V1(event) => &event.type_tag,
            ContractEvent::V2(event) => &event.type_tag,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            ContractEvent::V1(event) => event.size(),
            ContractEvent::V2(event) => event.size(),
        }
    }

    pub fn is_v1(&self) -> bool {
        matches!(self, ContractEvent::V1(_))
    }

    pub fn is_v2(&self) -> bool {
        matches!(self, ContractEvent::V2(_))
    }

    pub fn v1(&self) -> Result<&ContractEventV1> {
        Ok(match self {
            ContractEvent::V1(event) => event,
            ContractEvent::V2(_event) => bail!("This is a module event"),
        })
    }

    pub fn v2(&self) -> Result<&ContractEventV2> {
        Ok(match self {
            ContractEvent::V1(_event) => bail!("This is a instance event"),
            ContractEvent::V2(event) => event,
        })
    }

    pub fn try_v2(&self) -> Option<&ContractEventV2> {
        match self {
            ContractEvent::V1(_event) => None,
            ContractEvent::V2(event) => Some(event),
        }
    }

    pub fn try_v2_typed<T: DeserializeOwned>(&self, event_type: &TypeTag) -> Result<Option<T>> {
        if let Some(v2) = self.try_v2() {
            if &v2.type_tag == event_type {
                return Ok(Some(bcs::from_bytes(&v2.event_data)?));
            }
        }

        Ok(None)
    }

    pub fn is_new_epoch_event(&self) -> bool {
        match self {
            ContractEvent::V1(event) => *event.key() == new_epoch_event_key(),
            ContractEvent::V2(_event) => false,
        }
    }

    pub fn expect_new_block_event(&self) -> Result<NewBlockEvent> {
        NewBlockEvent::try_from_bytes(self.event_data())
    }
}

// TODO(BobOng): [framework-upgrade] To confirm usefulness of this implements
// // Temporary hack to avoid massive changes, it won't work when new variant comes and needs proper
// // dispatch at that time.
// impl Deref for ContractEvent {
//     type Target = ContractEventV1;
//
//     fn deref(&self) -> &Self::Target {
//         match self {
//             Self::V1(event) => event,
//             Self::V2(event) => event,
//         }
//     }
// }

/// Entry produced via a call to the `emit_event` builtin.
#[derive(Hash, Clone, Eq, PartialEq, Serialize, Deserialize, CryptoHasher)]
pub struct ContractEventV1 {
    /// The unique key that the event was emitted to
    key: EventKey,
    /// The number of messages that have been emitted to the path previously
    sequence_number: u64,
    /// The type of the data
    type_tag: TypeTag,
    /// The data payload of the event
    #[serde(with = "serde_bytes")]
    event_data: Vec<u8>,
}

impl ContractEventV1 {
    pub fn new(
        key: EventKey,
        sequence_number: u64,
        type_tag: TypeTag,
        event_data: Vec<u8>,
    ) -> Self {
        Self {
            key,
            sequence_number,
            type_tag,
            event_data,
        }
    }

    pub fn key(&self) -> &EventKey {
        &self.key
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn event_data(&self) -> &[u8] {
        &self.event_data
    }

    pub fn decode_event<EventType: MoveResource>(&self) -> Result<EventType> {
        bcs_ext::from_bytes(self.event_data.as_slice()).map_err(Into::into)
    }

    pub fn type_tag(&self) -> &TypeTag {
        &self.type_tag
    }

    pub fn is<EventType: MoveResource>(&self) -> bool {
        self.type_tag == TypeTag::Struct(Box::new(EventType::struct_tag()))
    }

    pub fn size(&self) -> usize {
        self.key.size() + 8 /* u64 */ + bcs::serialized_size(&self.type_tag).unwrap() + self.event_data.len()
    }
}

impl std::fmt::Debug for ContractEventV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ContractEvent {{ key: {:?}, index: {:?}, type: {:?}, event_data: {:?} }}",
            self.key,
            self.sequence_number,
            self.type_tag,
            hex::encode(&self.event_data)
        )
    }
}

/// Entry produced via a call to the `emit` builtin.
#[derive(Hash, Clone, Eq, PartialEq, Serialize, Deserialize, CryptoHasher)]
pub struct ContractEventV2 {
    /// The type of the data
    type_tag: TypeTag,
    /// The data payload of the event
    #[serde(with = "serde_bytes")]
    event_data: Vec<u8>,
}

impl ContractEventV2 {
    pub fn new(type_tag: TypeTag, event_data: Vec<u8>) -> Self {
        Self {
            type_tag,
            event_data,
        }
    }

    pub fn size(&self) -> usize {
        bcs::serialized_size(&self.type_tag).unwrap() + self.event_data.len()
    }

    pub fn type_tag(&self) -> &TypeTag {
        &self.type_tag
    }

    pub fn event_data(&self) -> &[u8] {
        &self.event_data
    }
}

impl std::fmt::Debug for ContractEventV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ModuleEvent {{ type: {:?}, event_data: {:?} }}",
            self.type_tag,
            hex::encode(&self.event_data)
        )
    }
}

impl TryFrom<&ContractEvent> for WithdrawEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        match event {
            ContractEvent::V1(event) => {
                if event.type_tag != TypeTag::Struct(Box::new(Self::struct_tag())) {
                    bail!("Expected Sent Payment")
                }
                Self::try_from_bytes(&event.event_data)
            }
            ContractEvent::V2(_) => bail!("This is a module event"),
        }
    }
}

impl TryFrom<&ContractEvent> for DepositEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        match event {
            ContractEvent::V1(event) => {
                if event.type_tag != TypeTag::Struct(Box::new(Self::struct_tag())) {
                    bail!("Expected Received Payment")
                }
                Self::try_from_bytes(&event.event_data)
            }
            ContractEvent::V2(_) => bail!("This is a module event"),
        }
    }
}

impl std::fmt::Debug for ContractEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractEvent::V1(event) => event.fmt(f),
            ContractEvent::V2(event) => event.fmt(f),
        }
    }
}

impl std::fmt::Display for ContractEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(payload) = WithdrawEvent::try_from(self) {
            let v1 = self.v1().unwrap();
            write!(
                f,
                "ContractEvent {{ key: {}, index: {:?}, type: {:?}, event_data: {:?} }}",
                v1.key, v1.sequence_number, v1.type_tag, payload,
            )
        } else if let Ok(payload) = DepositEvent::try_from(self) {
            let v1 = self.v1().unwrap();
            write!(
                f,
                "ContractEvent {{ key: {}, index: {:?}, type: {:?}, event_data: {:?} }}",
                v1.key, v1.sequence_number, v1.type_tag, payload,
            )
        } else {
            write!(f, "{:?}", self)
        }
    }
}
