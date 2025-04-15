// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::view::code_view::CodeView;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum WriteOpValueView {
    Code(CodeView),
    Resource(ResourceView),
}
