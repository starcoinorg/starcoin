// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{function_arg_type_view::StructTagView, str_view::StrView};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_abi_decoder::DecodedMoveValue;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResourceView {
    pub raw: StrView<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<DecodedMoveValue>,
}

impl From<Vec<u8>> for ResourceView {
    fn from(v: Vec<u8>) -> Self {
        Self {
            raw: StrView(v),
            json: None,
        }
    }
}


#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListResourceView {
    pub resources: BTreeMap<StructTagView, ResourceView>,
}
