// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::transaction_event_view::TransactionEventView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_abi_decoder::DecodedMoveValue;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct TransactionEventResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decode_event_data: Option<DecodedMoveValue>,
    #[serde(flatten)]
    pub event: TransactionEventView,
}
