// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::event::Event;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, Value};
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockHeader;
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::event::{EventHandle, EventKey};
/// Subscription kind.
#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    /// New block headers subscription.
    NewHeads,
    /// Events subscription.
    Events,
    /// New Pending Transactions subscription.
    NewPendingTransactions,
}

/// Subscription result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Result {
    /// New block header.
    Header(Box<BlockHeader>),
    /// Transaction hash
    TransactionHash(Vec<HashValue>),
    Events(Box<Event>),
}

impl Serialize for Result {
    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Result::Header(ref header) => header.serialize(serializer),
            Result::Event(ref evt) => evt.serialize(serializer),
            Result::TransactionHash(ref hash) => hash.serialize(serializer),
            // Result::SyncState(ref sync) => sync.serialize(serializer),
        }
    }
}

/// Subscription kind.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Params {
    /// No parameters passed.
    None,
    /// Log parameters.
    Events(EventFilter),
}

impl Default for Params {
    fn default() -> Self {
        Params::None
    }
}

impl<'a> Deserialize<'a> for Params {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Params, D::Error>
    where
        D: Deserializer<'a>,
    {
        let v: Value = Deserialize::deserialize(deserializer)?;

        if v.is_null() {
            return Ok(Params::None);
        }
        // Err(D::Error::custom("Invalid Pub-Sub parameters"));
        from_value(v)
            .map(Params::Events)
            .map_err(|e| D::Error::custom(format!("Invalid Pub-Sub parameters: {}", e)))
    }
}

/// Filter
#[derive(Debug, PartialEq, Clone, Deserialize, Eq, Hash)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct EventFilter {
    /// From Block
    pub from_block: Option<u64>,
    /// To Block
    pub to_block: Option<u64>,
    /// Block hash
    pub block_hash: Option<HashValue>,
    /// Event key
    pub event_key: Option<EventKey>,
    /// Limit: from latest to oldest
    pub limit: Option<usize>,
}
