// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{block::Block, view::str_view::StrView};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RawBlockView {
    /// Raw block header that encoded in hex format.
    pub header: StrView<Vec<u8>>,

    /// Raw block body that encoded in hex format.
    pub body: StrView<Vec<u8>>,
}

impl TryFrom<&Block> for RawBlockView {
    type Error = anyhow::Error;

    fn try_from(value: &Block) -> Result<Self, Self::Error> {
        let header = StrView(bcs_ext::to_bytes(value.header())?);
        let body = StrView(bcs_ext::to_bytes(&value.body)?);
        Ok(Self { header, body })
    }
}
