// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMError;
use move_core_types::gas_algebra::NumBytes;
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
};
use smallvec::{smallvec, SmallVec};
use starcoin_gas_schedule::gas_params::natives::starcoin_framework_legacy::*;
use starcoin_native_interface::{
    safely_pop_arg, RawSafeNative, SafeNativeBuilder, SafeNativeContext, SafeNativeResult,
};
use std::collections::VecDeque;

// !!!! NOTE !!!!
// This file is intended for natives from the util module in the framework.
// DO NOT PUT HELPER FUNCTIONS HERE!

/// Abort code when from_bytes fails (0x01 == INVALID_ARGUMENT)
const EFROM_BYTES: u64 = 0x01_0001;

/***************************************************************************************************
 * native fun from_bytes
 *
 *   gas cost: base_cost + unit_cost * bytes_len
 *
 **************************************************************************************************/

fn native_from_bytes(
    context: &mut SafeNativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(arguments.len(), 1);

    let bytes = safely_pop_arg!(arguments, Vec<u8>);

    let cost = FROM_BCS_BASE + FROM_BCS_BASE * NumBytes::new(bytes.len() as u64);
    context.charge(cost)?;

    // TODO(Gas): charge for getting the layout
    // let layout = context.type_to_type_layout(&ty_args[0])?.ok_or_else(|| {
    //     PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(format!(
    //         "Failed to get layout of type {:?} -- this should not happen",
    //         ty_args[0]
    //     ))
    // })?;
    let layout = context.type_to_type_layout(&ty_args[0])?;

    // context.charge(gas_params.base + gas_params.per_byte * NumBytes::new(bytes.len() as u64))?;
    // cost += gas_params.per_byte * NumBytes::new(bytes.len() as u64);
    let val = match Value::simple_deserialize(&bytes, &layout) {
        Some(val) => val,
        None => {
            return Ok(NativeResult::err(cost, EFROM_BYTES));
        }
    };
    Ok(smallvec![val])
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct GasParameters {
//     pub base: InternalGas,
//     pub per_byte: InternalGasPerByte,
// }

pub fn make_all(
    builder: &SafeNativeBuilder,
) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
    // let natives = [(
    //     "from_bytes",
    //     make_native_from_func(gas_params, native_from_bytes),
    // )];
    // crate::helpers::make_module_natives(natives)
    let natives = [("from_bytes", native_from_bytes as RawSafeNative)];
}

// pub(crate) fn native_create_signer(
//     context: &mut SafeNativeContext,
//     ty_args: Vec<Type>,
//     mut arguments: VecDeque<Value>,
// ) -> SafeNativeResult<SmallVec<[Value; 1]>> {
//     debug_assert!(ty_args.is_empty());
//     debug_assert!(arguments.len() == 1);
//
//     context.charge(ACCOUNT_CREATE_SIGNER_BASE)?;
//
//     let address = safely_pop_arg!(arguments, AccountAddress);
//     Ok(smallvec![Value::signer(address)])
// }
//
// /***************************************************************************************************
//  * module
//  *
//  **************************************************************************************************/
// pub fn make_all(
//     builder: &SafeNativeBuilder,
// ) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
//     let natives = [("create_signer", native_create_signer as RawSafeNative)];
//
//     builder.make_named_natives(natives)
// }
