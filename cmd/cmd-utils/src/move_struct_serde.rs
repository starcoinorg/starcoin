// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::ser::SerializeMap;
use serde::Serializer;
use starcoin_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue};

#[derive(Debug, Clone)]
pub struct MoveStruct(pub AnnotatedMoveStruct);

impl serde::Serialize for MoveStruct {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.value.len()))?;
        for (field, value) in &self.0.value {
            map.serialize_entry(field.as_str(), &MoveValue(value.clone()))?;
        }
        map.end()
    }
}

#[derive(Debug, Clone)]
struct MoveValue(AnnotatedMoveValue);

impl serde::Serialize for MoveValue {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            AnnotatedMoveValue::Bool(b) => serializer.serialize_bool(*b),
            AnnotatedMoveValue::U8(v) => serializer.serialize_u8(*v),
            AnnotatedMoveValue::U64(v) => serializer.serialize_u64(*v),
            AnnotatedMoveValue::U128(v) => serializer.serialize_u128(*v),
            AnnotatedMoveValue::Address(v) => v.serialize(serializer),
            AnnotatedMoveValue::Vector(v) => {
                let vs: Vec<_> = v.clone().into_iter().map(MoveValue).collect();
                vs.serialize(serializer)
            }
            AnnotatedMoveValue::Bytes(v) => hex::encode(v).serialize(serializer),
            AnnotatedMoveValue::Struct(v) => MoveStruct(v.clone()).serialize(serializer),
            AnnotatedMoveValue::U16(v) => serializer.serialize_u16(*v),
            AnnotatedMoveValue::U32(v) => serializer.serialize_u32(*v),
            AnnotatedMoveValue::U256(v) => v.serialize(serializer),
        }
    }
}
