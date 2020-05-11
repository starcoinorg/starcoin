// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config::{
        received_payment_tag, sent_payment_tag, ReceivedPaymentEvent, SentPaymentEvent,
    },
    event::EventKey,
    language_storage::TypeTag,
};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, ops::Deref};

/// Support versioning of the data structure.
#[derive(Hash, Clone, Eq, PartialEq, Serialize, Deserialize)]
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
#[derive(Hash, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContractEventV0 {
    /// The unique key that the event was emitted to
    key: EventKey,
    /// The number of messages that have been emitted to the path previously
    sequence_number: u64,
    /// The type of the data
    type_tag: TypeTag,
    /// The data payload of the event
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

impl TryFrom<&ContractEvent> for SentPaymentEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(sent_payment_tag()) {
            anyhow::bail!("Expected Sent Payment")
        }
        Self::try_from_bytes(&event.event_data)
    }
}

impl TryFrom<&ContractEvent> for ReceivedPaymentEvent {
    type Error = Error;

    fn try_from(event: &ContractEvent) -> Result<Self> {
        if event.type_tag != TypeTag::Struct(received_payment_tag()) {
            anyhow::bail!("Expected Received Payment")
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
        if let Ok(payload) = SentPaymentEvent::try_from(self) {
            write!(
                f,
                "ContractEvent {{ key: {}, index: {:?}, type: {:?}, event_data: {:?} }}",
                self.key, self.sequence_number, self.type_tag, payload,
            )
        } else if let Ok(payload) = ReceivedPaymentEvent::try_from(self) {
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

impl From<&libra_types::contract_event::ContractEvent> for ContractEvent {
    fn from(event: &libra_types::contract_event::ContractEvent) -> Self {
        ContractEvent::new(
            event.key().clone().into(),
            event.sequence_number(),
            event.type_tag().clone().into(),
            event.event_data().to_vec(),
        )
    }
}
