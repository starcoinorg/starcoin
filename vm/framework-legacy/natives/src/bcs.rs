// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_core_types::account_address::AccountAddress;
use move_core_types::vm_status::sub_status::NFE_BCS_TO_ADDRESS_FAILURE;
use move_core_types::{
    vm_status::sub_status::NFE_BCS_SERIALIZATION_FAILURE,
};
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{values_impl::Reference, Value},
};
use smallvec::{smallvec, SmallVec};
use starcoin_native_interface::{
    safely_pop_arg, RawSafeNative,
    SafeNativeBuilder, SafeNativeContext, SafeNativeError, SafeNativeResult
};
use std::collections::VecDeque;
use starcoin_gas_schedule::gas_params::natives::move_stdlib::BCS_TO_BYTES_FAILURE;
/***************************************************************************************************
 * native fun to_bytes
 *
 *   gas cost: size_of(val_type) * input_unit_cost +        | get type layout
 *             size_of(val) * input_unit_cost +             | serialize value
 *             max(size_of(output), 1) * output_unit_cost
 *
 *             If any of the first two steps fails, a partial cost + an additional failure_cost
 *             will be charged.
 *
 **************************************************************************************************/

/// Rust implementation of Move's `native public fun to_bytes<T>(&T): vector<u8>`
#[inline]
fn native_to_bytes(
    context: &mut SafeNativeContext,
    mut ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);

    // let mut cost = 0.into();

    // pop type and value
    let ref_to_val = safely_pop_arg!(args, Reference);
    let arg_type = ty_args.pop().unwrap();

    // get type layout
    let layout = match context.type_to_type_layout(&arg_type)? {
        Some(layout) => layout,
        None => {
            // cost += gas_params.failure;
            context.charge(BCS_TO_BYTES_FAILURE)?;
            return Err(SafeNativeError::Abort {
                abort_code: NFE_BCS_SERIALIZATION_FAILURE,
            });
        }
    };
    // serialize value
    let val = ref_to_val.read_ref()?;
    let serialized_value = match val.simple_serialize(&layout) {
        Some(serialized_value) => serialized_value,
        None => {
            // cost += gas_params.failure;
            context.charge(BCS_TO_BYTES_FAILURE)?;
            return Err(SafeNativeError::Abort {
                abort_code: NFE_BCS_SERIALIZATION_FAILURE,
            });
        }
    };
    // cost += gas_params.per_byte_serialized
    //     * std::cmp::max(
    //         NumBytes::new(serialized_value.len() as u64),
    //         gas_params.legacy_min_output_size,
    //     );


    Ok(smallvec![Value::vector_u8(serialized_value)])
}

/***************************************************************************************************
 * native fun native_to_address
 *
 *   gas cost: base_cost + unit_cost * data_length
 *
 **************************************************************************************************/

/// Rust implementation of Move's `public fun from_public_key_vec(pub_key_vec: vector<u8>): address;`
pub fn native_to_address(
    context: &mut SafeNativeContext,
    mut ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let key = pop_arg!(args, Vec<u8>);
    // let cost = gas_params.base + gas_params.per_byte * NumBytes::new(key.len() as u64);
    if key.len() != AccountAddress::LENGTH {
        return Ok(NativeResult::err(cost, NFE_BCS_TO_ADDRESS_FAILURE));
    }

    let address = match AccountAddress::try_from(&key[..AccountAddress::LENGTH]) {
        Ok(addr) => addr,
        Err(_) => return Ok(NativeResult::err(cost, NFE_BCS_TO_ADDRESS_FAILURE)),
    };
    let return_values = smallvec![Value::address(address)];
    Ok(NativeResult::ok(cost, return_values))
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

pub fn make_all(builder: &SafeNativeBuilder) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        ("to_bytes", native_to_bytes as RawSafeNative),
        ("to_address", native_to_address as RawSafeNative),
    ];

    builder.make_named_natives(natives)
}
