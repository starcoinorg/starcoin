// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::errors;
use jsonrpc_core::error::Error as JsonRpcError;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, Value};
use starcoin_crypto::HashValue;
use starcoin_types::block::{BlockHeader, BlockNumber};
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::event::EventKey;
use starcoin_types::filter::Filter;
use starcoin_types::language_storage::TypeTag;
use starcoin_types::U256;
use std::convert::{TryFrom, TryInto};

/// Subscription kind.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    /// New block subscription.
    NewHeads,
    /// Events subscription.
    Events,
    /// New Pending Transactions subscription.
    NewPendingTransactions,
    /// New block for minting
    NewMintBlock,
}

/// Subscription result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Result {
    /// New block.
    Block(Box<ThinHeadBlock>),
    /// Transaction hash
    TransactionHash(Vec<HashValue>),
    Event(Box<Event>),
    MintBlock(Box<MintBlock>),
}

impl Serialize for Result {
    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Result::Block(ref header) => header.serialize(serializer),
            Result::Event(ref evt) => evt.serialize(serializer),
            Result::TransactionHash(ref hash) => hash.serialize(serializer),
            Result::MintBlock(ref block) => block.serialize(serializer), // Result::SyncState(ref sync) => sync.serialize(serializer),
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
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, Hash)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct EventFilter {
    /// From Block
    #[serde(default)]
    pub from_block: Option<u64>,
    /// To Block
    #[serde(default)]
    pub to_block: Option<u64>,
    /// Event keys
    #[serde(default)]
    pub event_keys: Vec<EventKey>,
    /// Limit: from latest to oldest
    #[serde(default)]
    pub limit: Option<usize>,
}

impl TryInto<Filter> for EventFilter {
    type Error = JsonRpcError;

    fn try_into(self) -> std::result::Result<Filter, Self::Error> {
        match (self.from_block, self.to_block) {
            (Some(f), Some(t)) if f > t => {
                return Err(errors::invalid_params(
                    "fromBlock",
                    "fromBlock should not greater than toBlock",
                ));
            }
            _ => {}
        }
        Ok(Filter {
            from_block: self.from_block.unwrap_or(0),
            to_block: self.to_block.unwrap_or(std::u64::MAX),
            event_keys: self.event_keys,
            limit: self.limit,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub block_hash: Option<HashValue>,
    pub block_number: Option<BlockNumber>,
    pub transaction_hash: Option<HashValue>,
    // txn index in block
    pub transaction_index: Option<u64>,

    pub data: Vec<u8>,
    pub type_tags: TypeTag,
    #[serde(
        deserialize_with = "deserialize_event_key",
        serialize_with = "serialize_event_key"
    )]
    pub event_key: EventKey,
    pub event_seq_number: u64,
}

impl Event {
    pub fn new(
        block_hash: Option<HashValue>,
        block_number: Option<BlockNumber>,
        transaction_hash: Option<HashValue>,
        transaction_index: Option<u64>,
        contract_event: &ContractEvent,
    ) -> Self {
        Self {
            block_hash,
            block_number,
            transaction_hash,
            transaction_index,
            data: contract_event.event_data().to_vec(),
            type_tags: contract_event.type_tag().clone(),
            event_key: *contract_event.key(),
            event_seq_number: contract_event.sequence_number(),
        }
    }
}

pub fn serialize_event_key<S>(key: &EventKey, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(format!("{:#x}", key).as_str())
}

pub fn deserialize_event_key<'de, D>(d: D) -> std::result::Result<EventKey, D::Error>
where
    D: Deserializer<'de>,
{
    struct EventKeyVisitor;

    impl<'de> serde::de::Visitor<'de> for EventKeyVisitor {
        type Value = EventKey;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("EventKey in hex string")
        }
        fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let b = hex::decode(v.as_bytes()).map_err(E::custom)?;
            EventKey::try_from(b.as_slice()).map_err(E::custom)
        }
    }
    d.deserialize_str(EventKeyVisitor)
}

/// Block with only txn hashes.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct ThinHeadBlock {
    #[serde(flatten)]
    header: BlockHeader,
    #[serde(rename = "txn_hashes")]
    body: Vec<HashValue>,
}

impl ThinHeadBlock {
    pub fn new(header: BlockHeader, txn_hashes: Vec<HashValue>) -> Self {
        Self {
            header,
            body: txn_hashes,
        }
    }
    pub fn header(&self) -> &BlockHeader {
        &self.header
    }
    pub fn body(&self) -> &[HashValue] {
        &self.body
    }
}

/// Block for minting
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct MintBlock {
    pub header_hash: HashValue,
    pub difficulty: U256,
}
