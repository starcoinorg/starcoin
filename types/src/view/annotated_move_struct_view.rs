// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{
    annotated_move_value_view::AnnotatedMoveValueView, function_arg_type_view::StructTagView,
    str_view::StrView,
};
use move_core_types::identifier::Identifier;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_resource_viewer::AnnotatedMoveStruct;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AnnotatedMoveStructView {
    pub abilities: u8,
    pub type_: StructTagView,
    #[schemars(with = "Vec<(String, AnnotatedMoveValueView)>")]
    pub value: Vec<(Identifier, AnnotatedMoveValueView)>,
}

impl From<AnnotatedMoveStruct> for AnnotatedMoveStructView {
    fn from(origin: AnnotatedMoveStruct) -> Self {
        Self {
            abilities: origin.abilities.into_u8(),
            type_: StrView(origin.type_),
            value: origin
                .value
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}
