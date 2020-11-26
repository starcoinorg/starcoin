// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::errors;
use crate::types::{BlockView, TransactionEventView};
use jsonrpc_core::error::Error as JsonRpcError;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, Value};
use starcoin_crypto::HashValue;
use starcoin_types::event::EventKey;
use starcoin_types::filter::Filter;
use starcoin_types::U256;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use std::convert::TryInto;

/// Subscription kind.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type_name")]
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
    Block(Box<BlockView>),
    /// Transaction hash
    TransactionHash(Vec<HashValue>),
    Event(Box<TransactionEventView>),
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
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize)]
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
            reverse: true,
        })
    }
}

/// Block for minting
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MintBlock {
    pub strategy: ConsensusStrategy,
    pub minting_blob: Vec<u8>,
    pub difficulty: U256,
}
