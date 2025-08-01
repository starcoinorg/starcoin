// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::view::{annotated_move_struct_view::AnnotatedMoveStructView, str_view::StrView};
use move_core_types::{account_address::AccountAddress, u256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_resource_viewer::AnnotatedMoveValue;

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub enum AnnotatedMoveValueView {
    U8(u8),
    U64(StrView<u64>),
    U128(StrView<u128>),
    Bool(bool),
    Address(AccountAddress),
    Vector(Vec<AnnotatedMoveValueView>),
    Bytes(StrView<Vec<u8>>),
    Struct(AnnotatedMoveStructView),
    U16(StrView<u16>),
    U32(StrView<u32>),
    U256(StrView<u256::U256>),
}

impl From<AnnotatedMoveValue> for AnnotatedMoveValueView {
    fn from(origin: AnnotatedMoveValue) -> Self {
        match origin {
            AnnotatedMoveValue::U8(u) => Self::U8(u),
            AnnotatedMoveValue::U64(u) => Self::U64(StrView(u)),
            AnnotatedMoveValue::U128(u) => Self::U128(StrView(u)),
            AnnotatedMoveValue::Bool(b) => Self::Bool(b),
            AnnotatedMoveValue::Address(data) => Self::Address(data),
            AnnotatedMoveValue::Vector(data) => {
                Self::Vector(data.into_iter().map(Into::into).collect())
            }
            AnnotatedMoveValue::Bytes(data) => Self::Bytes(StrView(data)),
            AnnotatedMoveValue::Struct(data) => Self::Struct(data.into()),
            AnnotatedMoveValue::U16(u) => Self::U16(StrView(u)),
            AnnotatedMoveValue::U32(u) => Self::U32(StrView(u)),
            AnnotatedMoveValue::U256(u) => Self::U256(StrView(u)),
        }
    }
}
