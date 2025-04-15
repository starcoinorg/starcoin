// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::str_view::StrView;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
// use starcoin_abi_decoder::DecodedMoveValue;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResourceView {
    pub raw: StrView<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<DecodedMoveValue>,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListResourceView {
    pub resources: BTreeMap<StructTagView, ResourceView>,
}
