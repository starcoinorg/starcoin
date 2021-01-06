// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config::{BurnEvent, DepositEvent, MintEvent, WithdrawEvent},
    event::EventKey,
};
use anyhow::{Error, Result};
use move_core_types::{language_storage::TypeTag, move_resource::MoveResource};
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, CryptoHasher};
use std::{convert::TryFrom, ops::Deref};

/// Support versioning of the data structure.
#[derive(Hash, Clone, Eq, PartialEq, Serialize, Deserialize, CryptoHasher, CryptoHash)]
pub enum ContractEvent {
    V0(ContractEventV0),
}

impl ContractEvent {
    pub fn new(
        key: EventKey,
        sequence_number: u64,
        type_tag: TypeTag,
        event_data: Vec<u8>,
    ) -> Self {
        ContractEvent::V0(ContractEventV0::new(
            key,
            sequence_number,
            type_tag,
            event_data,
        ))
    }
}

// Temporary hack to avoid massive changes, it won't work when new variant comes and needs proper
// dispatch at that time.
impl Deref for ContractEvent {
    type Target = ContractEventV0;

    fn deref(&self) -> &Self::Target {
        match self {
            ContractEvent::V0(event) => event,
        }
    }
}

/// Entry produced via a call to the `emit_event` builtin.
#[derive(Hash, Clone, Eq, PartialEq, Serialize, Deserialize, CryptoHasher)]
pub struct ContractEventV0 {
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

impl ContractEventV0 {
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

    pub fn type_tag(&self) -> &TypeTag {
        &self.type_tag
    }
}

impl TryFrom<&ContractEvent> for WithdrawEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(WithdrawEvent::struct_tag()) {
            anyhow::bail!("Expected Withdraw Event")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for DepositEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(DepositEvent::struct_tag()) {
            anyhow::bail!("Expected Deposit Event")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for BurnEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(BurnEvent::struct_tag()) {
            anyhow::bail!("Expected BurnEvent")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for MintEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(MintEvent::struct_tag()) {
            anyhow::bail!("Expected BurnEvent")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl std::fmt::Debug for ContractEvent {
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

impl std::fmt::Display for ContractEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(payload) = WithdrawEvent::try_from(self) {
            write!(
                f,
                "ContractEvent {{ key: {}, index: {:?}, type: {:?}, event_data: {:?} }}",
                self.key, self.sequence_number, self.type_tag, payload,
            )
        } else if let Ok(payload) = DepositEvent::try_from(self) {
            write!(
                f,
                "ContractEvent {{ key: {}, index: {:?}, type: {:?}, event_data: {:?} }}",
                self.key, self.sequence_number, self.type_tag, payload,
            )
        } else {
            write!(f, "{:?}", self)
        }
    }
}

//TODO implement EventWithProof
pub struct EventWithProof {}
