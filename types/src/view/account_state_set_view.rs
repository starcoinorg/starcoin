// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{
    annotated_move_struct_view::AnnotatedMoveStructView, function_arg_type_view::StructTagView,
    str_view::StrView, ByteCode,
};
use move_core_types::identifier::Identifier;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct AccountStateSetView {
    #[schemars(with = "String")] //TODO impl in schemars
    pub codes: BTreeMap<Identifier, StrView<ByteCode>>,
    pub resources: BTreeMap<StructTagView, AnnotatedMoveStructView>,
}
