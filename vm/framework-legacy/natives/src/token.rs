// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    language_storage::TypeTag, vm_status::sub_status::NFE_TOKEN_INVALID_TYPE_ARG_FAILURE,
};
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{loaded_data::runtime_types::Type, values::Value};
use smallvec::{smallvec, SmallVec};
use starcoin_gas_schedule::gas_params::natives::starcoin_framework_legacy::*;
use starcoin_native_interface::{
    RawSafeNative, SafeNativeBuilder, SafeNativeContext, SafeNativeError, SafeNativeResult,
};
use starcoin_types::error;
use std::collections::VecDeque;

/***************************************************************************************************
 * native fun token_name_of
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/

/// Return Token types ModuleAddress, ModuleName and StructName
pub fn native_token_name_of(
    context: &mut SafeNativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(arguments.is_empty());

    context.charge(TOKEN_NAME_OF_BASE)?;

    let type_tag = context.type_to_type_tag(&ty_args[0])?;

    if let TypeTag::Struct(struct_tag) = type_tag {
        let mut name = struct_tag.name.as_bytes().to_vec();
        let type_args_info =
            format_type_params(&struct_tag.type_args).expect("format should never fail");
        name.append(&mut type_args_info.into_bytes());
        Ok(smallvec![
            Value::address(struct_tag.address),
            Value::vector_u8(struct_tag.module.as_bytes().to_vec()),
            Value::vector_u8(name),
        ])
    } else {
        Err(SafeNativeError::Abort {
            abort_code: error::invalid_state(NFE_TOKEN_INVALID_TYPE_ARG_FAILURE),
        })
    }
}

/// Copy from StructTag's display impl.
fn format_type_params(type_params: &[TypeTag]) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
    let mut f = String::new();
    if let Some(first_ty) = type_params.first() {
        write!(f, "<")?;
        write!(f, "{}", first_ty)?;
        for ty in type_params.iter().skip(1) {
            write!(f, ", {}", ty)?;
        }
        write!(f, ">")?;
    }
    Ok(f)
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/

pub fn make_all(
    builder: &SafeNativeBuilder,
) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
    let natives = [("name_of", native_token_name_of as RawSafeNative)];
    builder.make_named_natives(natives)
}

#[test]
fn test_type_params_formatting() {
    use move_core_types::account_address::AccountAddress;
    use move_core_types::identifier::Identifier;
    use move_core_types::language_storage::StructTag;
    let a_struct = StructTag {
        address: AccountAddress::ZERO,
        module: Identifier::new("TestModule").unwrap(),
        name: Identifier::new("TestStruct").unwrap(),
        type_args: vec![TypeTag::Address],
    };
    let cases = vec![
        (vec![TypeTag::Address], "<address>"),
        (
            vec![TypeTag::Vector(Box::new(TypeTag::U8)), TypeTag::U64],
            "<vector<u8>, u64>",
        ),
        (
            vec![TypeTag::U64, TypeTag::Struct(Box::new(a_struct))],
            "<u64, 0x00000000000000000000000000000000::TestModule::TestStruct<address>>",
        ),
    ];

    for (ts, expected) in cases {
        let actual = format_type_params(&ts).unwrap();
        assert_eq!(&actual, expected);
    }
}
