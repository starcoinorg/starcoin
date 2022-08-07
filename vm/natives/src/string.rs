// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of native functions for utf8 strings.

use move_binary_format::errors::PartialVMResult;
use move_core_types::account_address::AccountAddress;
use move_vm_runtime::native_functions::NativeContext;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeResult},
    pop_arg,
    values::Value,
};
use smallvec::smallvec;
use starcoin_vm_types::gas_schedule::NativeCostIndex;
use std::collections::VecDeque;

// The implementation approach delegates all utf8 handling to Rust.
// This is possible without copying of bytes because (a) we can
// get a `std::cell::Ref<Vec<u8>>` from a `vector<u8>` and in turn a `&[u8]`
// from that (b) assuming that `vector<u8>` embedded in a string
// is already valid utf8, we can use `str::from_utf8_unchecked` to
// create a `&str` view on the bytes without a copy. Once we have this
// view, we can call ut8 functions like length, substring, etc.

/***************************************************************************************************
 * native fun internal_check_utf8
 *
 *   gas cost: base_cost + unit_cost * length_in_bytes
 *
 **************************************************************************************************/

pub fn make_native_check_utf8(
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 1);
    let s_arg = pop_arg!(arguments, VectorRef);
    let s_ref = s_arg.as_bytes_ref();
    let ok = std::str::from_utf8(s_ref.as_slice()).is_ok();

    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::CREATE_SIGNER as u8,
        0,
    );
    NativeResult::map_partial_vm_result_one(cost, Ok(Value::bool(ok)))
}

/***************************************************************************************************
 * native fun internal_is_char_boundary
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
pub fn make_native_is_char_boundary(
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 2);
    let i = pop_arg!(arguments, u64);
    let s_arg = pop_arg!(arguments, VectorRef);
    let s_ref = s_arg.as_bytes_ref();
    let ok = unsafe {
        // This is safe because we guarantee the bytes to be utf8.
        std::str::from_utf8_unchecked(s_ref.as_slice()).is_char_boundary(i as usize)
    };
    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::CREATE_SIGNER as u8,
        0,
    );
    NativeResult::map_partial_vm_result_one(cost, Ok(Value::bool(ok)))
}

/***************************************************************************************************
 * native fun internal_sub_string
 *
 *   gas cost: base_cost + unit_cost * sub_string_length_in_bytes
 *
 **************************************************************************************************/

pub fn make_native_sub_string(
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 3);
    let j = pop_arg!(arguments, u64) as usize;
    let i = pop_arg!(arguments, u64) as usize;

    if j < i {
        // TODO: what abort code should we use here?
        return Ok(NativeResult::err(1, 1));
    }

    let s_arg = pop_arg!(arguments, VectorRef);
    let s_ref = s_arg.as_bytes_ref();
    let s_str = unsafe {
        // This is safe because we guarantee the bytes to be utf8.
        std::str::from_utf8_unchecked(s_ref.as_slice())
    };
    let v = Value::vector_u8((&s_str[i..j]).as_bytes().iter().cloned());

    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::CREATE_SIGNER as u8,
        0,
    );
    NativeResult::map_partial_vm_result_one(cost, Ok(v))
}


