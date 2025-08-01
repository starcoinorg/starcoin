// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{code_view::CodeView, resource_view::ResourceView};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum WriteOpValueView {
    Code(CodeView),
    Resource(ResourceView),
}
