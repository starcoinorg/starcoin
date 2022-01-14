// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    fat_type::{FatStructType, FatType},
    resolver::Resolver,
};
use anyhow::{anyhow, Result};
use starcoin_vm_types::language_storage::TypeTag;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::value::MoveTypeLayout;
use starcoin_vm_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    contract_event::ContractEvent,
    errors::{Location, PartialVMError},
    file_format::{Ability, AbilitySet},
    identifier::Identifier,
    language_storage::StructTag,
    value::{MoveStruct, MoveValue},
};
use std::{
    collections::btree_map::BTreeMap,
    convert::TryInto,
    fmt::{Display, Formatter},
};

mod fat_type;
pub mod module_cache;
pub mod resolver;

#[derive(Debug)]
pub struct AnnotatedAccountStateBlob(BTreeMap<StructTag, AnnotatedMoveStruct>);

#[derive(Debug, Clone)]
pub struct AnnotatedMoveStruct {
    pub abilities: AbilitySet,
    pub type_: StructTag,
    pub value: Vec<(Identifier, AnnotatedMoveValue)>,
}

/// AnnotatedMoveValue is a fully expanded version of on chain move data. This should only be used
/// for debugging/client purpose right now and just for a better visualization of on chain data. In
/// the long run, we would like to transform this struct to a Json value so that we can have a cross
/// platform interpretation of the on chain data.
#[derive(Debug, Clone)]
pub enum AnnotatedMoveValue {
    U8(u8),
    U64(u64),
    U128(u128),
    Bool(bool),
    Address(AccountAddress),
    Vector(Vec<AnnotatedMoveValue>),
    Bytes(Vec<u8>),
    Struct(AnnotatedMoveStruct),
}

// impl Serialize for AnnotatedMoveValue {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         match self {
//             Self::U8(v) => serializer.serialize_u8(*v),
//             Self::U64(v) => serializer.serialize_u64(*v),
//             Self::U128(v) => serializer.serialize_u128(*v),
//             Self::Bool(v) => serializer.serialize_bool(*v),
//             AnnotatedMoveValue::Address(addr) => <AccountAddress>::serialize(addr, serializer),
//             AnnotatedMoveValue::Vector(values) => {
//                 let mut seq = serializer.serialize_seq(Some(values.len()))?;
//                 for v in values {
//                     seq.serialize_element(v)?;
//                 }
//                 seq.end()
//             }
//             AnnotatedMoveValue::Bytes(data) => serializer.serialize_str(hex::encode(data).as_str()),
//             AnnotatedMoveValue::Struct(s) => AnnotatedMoveStruct::serialize(s, serializer),
//         }
//     }
// }
//
// // TODO: better serialize as a real struct, instead of map.
// impl Serialize for AnnotatedMoveStruct {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut map = serializer.serialize_map(Some(self.value.len()))?;
//         for (f, v) in &self.value {
//             map.serialize_entry(f.as_str(), v)?;
//         }
//         map.end()
//     }
// }

pub struct MoveValueAnnotator<'a> {
    cache: Resolver<'a>,
    _data_view: &'a dyn StateView,
}

impl<'a> MoveValueAnnotator<'a> {
    pub fn new(view: &'a dyn StateView) -> Self {
        Self {
            cache: Resolver::new(view),
            _data_view: view,
        }
    }

    pub fn type_tag_to_type_layout(&self, ty: &TypeTag) -> Result<MoveTypeLayout> {
        let ty = self.cache.resolve_type(ty)?;
        let layout = (&ty)
            .try_into()
            .map_err(|e: PartialVMError| e.finish(Location::Undefined).into_vm_status())?;
        Ok(layout)
    }

    pub fn view_value(&self, type_tag: &TypeTag, blob: &[u8]) -> Result<AnnotatedMoveValue> {
        let ty = self.cache.resolve_type(type_tag)?;
        let move_ty = (&ty)
            .try_into()
            .map_err(|e: PartialVMError| e.finish(Location::Undefined).into_vm_status())?;

        let move_value = MoveValue::simple_deserialize(blob, &move_ty)?;
        self.annotate_value(&move_value, &ty)
    }

    pub fn view_struct(&self, struct_tag: StructTag, blob: &[u8]) -> Result<AnnotatedMoveStruct> {
        let ty = self.cache.resolve_struct(&struct_tag)?;
        let struct_def = (&ty)
            .try_into()
            .map_err(|e: PartialVMError| e.finish(Location::Undefined).into_vm_status())?;
        let move_struct = MoveStruct::simple_deserialize(blob, &struct_def)?;
        self.annotate_struct(&move_struct, &ty)
    }

    pub fn view_contract_event(&self, event: &ContractEvent) -> Result<AnnotatedMoveValue> {
        let ty = self.cache.resolve_type(event.type_tag())?;
        let move_ty = (&ty)
            .try_into()
            .map_err(|e: PartialVMError| e.finish(Location::Undefined).into_vm_status())?;

        let move_value = MoveValue::simple_deserialize(event.event_data(), &move_ty)?;
        self.annotate_value(&move_value, &ty)
    }

    fn annotate_struct(
        &self,
        move_struct: &MoveStruct,
        ty: &FatStructType,
    ) -> Result<AnnotatedMoveStruct> {
        let struct_tag = ty
            .struct_tag()
            .map_err(|e| e.finish(Location::Undefined).into_vm_status())?;
        let field_names = self.cache.get_field_names(ty)?;
        let mut annotated_fields = vec![];
        for (ty, v) in ty.layout.iter().zip(move_struct.fields().iter()) {
            annotated_fields.push(self.annotate_value(v, ty)?);
        }
        Ok(AnnotatedMoveStruct {
            abilities: ty.abilities.0,
            type_: struct_tag,
            value: field_names
                .into_iter()
                .zip(annotated_fields.into_iter())
                .collect(),
        })
    }

    fn annotate_value(&self, value: &MoveValue, ty: &FatType) -> Result<AnnotatedMoveValue> {
        Ok(match (value, ty) {
            (MoveValue::Bool(b), FatType::Bool) => AnnotatedMoveValue::Bool(*b),
            (MoveValue::U8(i), FatType::U8) => AnnotatedMoveValue::U8(*i),
            (MoveValue::U64(i), FatType::U64) => AnnotatedMoveValue::U64(*i),
            (MoveValue::U128(i), FatType::U128) => AnnotatedMoveValue::U128(*i),
            (MoveValue::Address(a), FatType::Address) => AnnotatedMoveValue::Address(*a),
            (MoveValue::Vector(a), FatType::Vector(ty)) => match ty.as_ref() {
                FatType::U8 => AnnotatedMoveValue::Bytes(
                    a.iter()
                        .map(|v| match v {
                            MoveValue::U8(i) => Ok(*i),
                            _ => Err(anyhow!("unexpected value type")),
                        })
                        .collect::<Result<_>>()?,
                ),
                _ => AnnotatedMoveValue::Vector(
                    a.iter()
                        .map(|v| self.annotate_value(v, ty.as_ref()))
                        .collect::<Result<_>>()?,
                ),
            },
            (MoveValue::Struct(s), FatType::Struct(ty)) => {
                AnnotatedMoveValue::Struct(self.annotate_struct(s, ty.as_ref())?)
            }
            _ => {
                return Err(anyhow!(
                    "Cannot annotate value {:?} with type {:?}",
                    value,
                    ty
                ))
            }
        })
    }
}

fn write_indent(f: &mut Formatter, indent: u64) -> std::fmt::Result {
    for _i in 0..indent {
        write!(f, " ")?;
    }
    Ok(())
}

fn pretty_print_value(
    f: &mut Formatter,
    value: &AnnotatedMoveValue,
    indent: u64,
) -> std::fmt::Result {
    match value {
        AnnotatedMoveValue::Bool(b) => write!(f, "{}", b),
        AnnotatedMoveValue::U8(v) => write!(f, "{}u8", v),
        AnnotatedMoveValue::U64(v) => write!(f, "{}", v),
        AnnotatedMoveValue::U128(v) => write!(f, "{}u128", v),
        AnnotatedMoveValue::Address(a) => write!(f, "0x{:#x}", a),
        AnnotatedMoveValue::Vector(v) => {
            writeln!(f, "[")?;
            for value in v.iter() {
                write_indent(f, indent + 4)?;
                pretty_print_value(f, value, indent + 4)?;
                writeln!(f, ",")?;
            }
            write_indent(f, indent)?;
            write!(f, "]")
        }
        AnnotatedMoveValue::Bytes(v) => write!(f, "{}", hex::encode(&v)),
        AnnotatedMoveValue::Struct(s) => pretty_print_struct(f, s, indent),
    }
}

fn pretty_print_struct(
    f: &mut Formatter,
    value: &AnnotatedMoveStruct,
    indent: u64,
) -> std::fmt::Result {
    pretty_print_ability_modifiers(f, value.abilities)?;
    writeln!(f, "{} {{", value.type_)?;
    for (field_name, v) in value.value.iter() {
        write_indent(f, indent + 4)?;
        write!(f, "{}: ", field_name)?;
        pretty_print_value(f, v, indent + 4)?;
        writeln!(f)?;
    }
    write_indent(f, indent)?;
    write!(f, "}}")
}

fn pretty_print_ability_modifiers(f: &mut Formatter, abilities: AbilitySet) -> std::fmt::Result {
    for ability in abilities {
        match ability {
            Ability::Copy => write!(f, "copy ")?,
            Ability::Drop => write!(f, "drop ")?,
            Ability::Store => write!(f, "store ")?,
            Ability::Key => write!(f, "key ")?,
        }
    }
    Ok(())
}

impl Display for AnnotatedMoveValue {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        pretty_print_value(f, self, 0)
    }
}

impl Display for AnnotatedMoveStruct {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        pretty_print_struct(f, self, 0)
    }
}

impl Display for AnnotatedAccountStateBlob {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for v in self.0.values() {
            write!(f, "{}", v)?;
            writeln!(f, ",")?;
        }
        writeln!(f, "}}")
    }
}

#[derive(Default)]
pub struct NullStateView;

impl StateView for NullStateView {
    fn get(&self, _access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    fn multi_get(&self, access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        Ok(access_paths.iter().map(|_| None).collect())
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
