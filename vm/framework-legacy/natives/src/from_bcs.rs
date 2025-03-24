// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMError;
use move_core_types::{gas_algebra::NumBytes, vm_status::StatusCode};
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{loaded_data::runtime_types::Type, values::Value};
use smallvec::{smallvec, SmallVec};
use starcoin_gas_schedule::gas_params::natives::starcoin_framework_legacy::*;
use starcoin_native_interface::{
    safely_pop_arg, RawSafeNative, SafeNativeBuilder, SafeNativeContext, SafeNativeError,
    SafeNativeResult,
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

    //let cost = U256_FROM_BYTES_PER_BYTE * NumBytes::new(bytes.len() as u64);
    let cost = FROM_BCS_BASE + U256_FROM_BYTES_PER_BYTE * NumBytes::new(bytes.len() as u64);
    context.charge(cost)?;

    let layout = context.type_to_type_layout(&ty_args[0])?;

    let val = match Value::simple_deserialize(&bytes, &layout) {
        Some(val) => val,
        None => {
            return Err(SafeNativeError::Abort {
                abort_code: EFROM_BYTES,
            })
        }
    };
    Ok(smallvec![val])
}

pub fn make_all(
    builder: &SafeNativeBuilder,
) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
    let natives = [("from_bytes", native_from_bytes as RawSafeNative)];

    builder.make_named_natives(natives)
}
