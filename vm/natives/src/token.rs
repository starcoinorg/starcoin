// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_core_types::language_storage::TypeTag;
use move_core_types::vm_status::sub_status::NFE_TOKEN_INVALID_TYPE_ARG_FAILURE;
use move_vm_runtime::native_functions::NativeContext;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeResult},
    values::Value,
};
use smallvec::smallvec;
use starcoin_vm_types::gas_schedule::NativeCostIndex;
use std::collections::VecDeque;

/// Return Token types ModuleAddress, ModuleName and StructName
pub fn native_token_name_of(
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(arguments.is_empty());
    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::TOKEN_NAME_OF as u8,
        1,
    );
    let type_tag = context.type_to_type_tag(&ty_args[0])?;
    if let TypeTag::Struct(struct_tag) = type_tag {
        let mut name = struct_tag.name.as_bytes().to_vec();
        let type_args_info =
            format_type_params(&struct_tag.type_params).expect("format should never fail");
        name.append(&mut type_args_info.into_bytes());
        Ok(NativeResult::ok(
            cost,
            smallvec![
                Value::address(struct_tag.address),
                Value::vector_u8(struct_tag.module.as_bytes().to_vec()),
                Value::vector_u8(name),
            ],
        ))
    } else {
        Ok(NativeResult::err(cost, NFE_TOKEN_INVALID_TYPE_ARG_FAILURE))
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

#[test]
fn test_type_params_formatting() {
    use move_core_types::account_address::AccountAddress;
    use move_core_types::identifier::Identifier;
    use move_core_types::language_storage::StructTag;
    let a_struct = StructTag {
        address: AccountAddress::ZERO,
        module: Identifier::new("TestModule").unwrap(),
        name: Identifier::new("TestStruct").unwrap(),
        type_params: vec![TypeTag::Address],
    };
    let cases = vec![
        (vec![TypeTag::Address], "<address>"),
        (
            vec![TypeTag::Vector(Box::new(TypeTag::U8)), TypeTag::U64],
            "<vector<u8>, u64>",
        ),
        (
            vec![TypeTag::U64, TypeTag::Struct(a_struct)],
            "<u64, 0x00000000000000000000000000000000::TestModule::TestStruct<address>>",
        ),
    ];

    for (ts, expected) in cases {
        let actual = format_type_params(&ts).unwrap();
        assert_eq!(&actual, expected);
    }
}
