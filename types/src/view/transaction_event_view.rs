// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block::BlockNumber,
    contract_event::ContractEventInfo,
    view::{function_arg_type_view::TypeTagView, str_view::StrView},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_vm_types::{contract_event::ContractEvent, event::EventKey};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, JsonSchema)]
pub struct TransactionEventView {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<HashValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<StrView<BlockNumber>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<HashValue>,
    // txn index in block
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_index: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_global_index: Option<StrView<u64>>,
    pub data: StrView<Vec<u8>>,
    pub type_tag: TypeTagView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_index: Option<u32>,
    pub event_key: EventKey,
    pub event_seq_number: StrView<u64>,
}

impl From<ContractEventInfo> for TransactionEventView {
    fn from(info: ContractEventInfo) -> Self {
        Self {
            block_hash: Some(info.block_hash),
            block_number: Some(info.block_number.into()),
            transaction_hash: Some(info.transaction_hash),
            transaction_index: Some(info.transaction_index),
            transaction_global_index: Some(info.transaction_global_index.into()),
            data: StrView(info.event.event_data().to_vec()),
            type_tag: info.event.type_tag().clone().into(),
            event_index: Some(info.event_index),
            event_key: info.event.event_key(),
            event_seq_number: info.event.sequence_number().into(),
        }
    }
}

impl From<ContractEvent> for TransactionEventView {
    fn from(contract_event: ContractEvent) -> Self {
        Self {
            block_hash: None,
            block_number: None,
            transaction_hash: None,
            transaction_index: None,
            transaction_global_index: None,
            data: StrView(contract_event.event_data().to_vec()),
            type_tag: contract_event.type_tag().clone().into(),
            event_index: None,
            event_key: contract_event.event_key(),
            event_seq_number: contract_event.sequence_number().into(),
        }
    }
}

impl TransactionEventView {
    pub fn new(
        block_hash: Option<HashValue>,
        block_number: Option<BlockNumber>,
        transaction_hash: Option<HashValue>,
        transaction_index: Option<u32>,
        transaction_global_index: Option<u64>,
        event_index: Option<u32>,
        contract_event: &ContractEvent,
    ) -> Self {
        Self {
            block_hash,
            block_number: block_number.map(Into::into),
            transaction_hash,
            transaction_index,
            transaction_global_index: transaction_global_index.map(Into::into),
            data: StrView(contract_event.event_data().to_vec()),
            type_tag: contract_event.type_tag().clone().into(),
            event_index,
            event_key: contract_event.event_key(),
            event_seq_number: contract_event.sequence_number().into(),
        }
    }
}
