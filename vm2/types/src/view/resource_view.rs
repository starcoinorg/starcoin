// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{function_arg_type_view::StructTagView, str_view::StrView};
use move_core_types::move_resource::MoveResource;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use starcoin_abi_decoder::DecodedMoveValue;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResourceView {
    pub raw: StrView<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<DecodedMoveValue>,
}

impl ResourceView {
    /// # Parameters
    /// - Generic parameter `R`: The target deserialization type, which must satisfy:
    ///   - Implements `MoveResource` (indicating it's a standard Move resource)
    ///   - Implements `DeserializeOwned` (ensuring complete ownership of deserialized data)
    ///
    /// # Returns
    /// - `Ok(R)`: Successfully deserialized instance of type R
    /// - `Err(anyhow::Error)`: Contains error details if deserialization fails, which could occur because:
    ///   - Byte data doesn't match BCS structure of type R
    ///   - Byte data is incomplete or malformed
    ///
    pub fn decode<R: MoveResource + DeserializeOwned>(&self) -> anyhow::Result<R> {
        bcs_ext::from_bytes(self.raw.0.as_slice())
    }
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
