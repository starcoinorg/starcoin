// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::str_view::StrView;
use move_core_types::identifier::Identifier;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CodeView {
    pub code: StrView<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abi: Option<ModuleABI>,
}

impl From<Vec<u8>> for CodeView {
    fn from(v: Vec<u8>) -> Self {
        Self {
            code: StrView(v),
            abi: None,
        }
    }
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ListCodeView {
    #[schemars(with = "String")]
    pub codes: BTreeMap<Identifier, CodeView>,
}
