// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_vm_runtime::native_functions::NativeContext;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeResult},
    pop_arg,
    values::Value,
};
use ripemd160::Digest;
use smallvec::smallvec;
use starcoin_vm_types::gas_schedule::NativeCostIndex;
use std::collections::VecDeque;
use base64;

pub fn native_base64_encode(
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let input_arg = pop_arg!(arguments, Vec<u8>);

    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::BASE64_ENCODE as u8,
        input_arg.len(),
    );

    let output = base64::encode(&input_arg);

    Ok(NativeResult::ok(cost, smallvec![Value::vector_u8(output.as_bytes().to_vec())]))
}

pub fn native_base64_decode(
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let input_arg = pop_arg!(arguments, Vec<u8>);

    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::BASE64_DECODE as u8,
        input_arg.len(),
    );

    let base64_decoded = base64::decode(&input_arg);
    if let Ok(base64_decoded) = base64_decoded {
        Ok(NativeResult::ok(
            cost,
            smallvec![Value::vector_u8(base64_decoded)],
        ))
    } else {
        return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
    }
}

