// Copyright © Starcoin Foundation
// SPDX-License-Identifier: Apache-2.0
use move_core_types::account_address::AccountAddress;
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{loaded_data::runtime_types::Type, values::Value};
use smallvec::smallvec;
use smallvec::SmallVec;
use starcoin_native_interface::{safely_assert_eq, safely_pop_arg};
use starcoin_native_interface::{
    RawSafeNative, SafeNativeBuilder, SafeNativeContext, SafeNativeResult,
};
use std::collections::VecDeque;
/***************************************************************************************************
 * native fun address_to_u128(addr: address): u128;
 **************************************************************************************************/
fn native_address_to_u128(
    _context: &mut SafeNativeContext,
    mut _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    safely_assert_eq!(_ty_args.len(), 0);
    safely_assert_eq!(args.len(), 1);
    let addr = safely_pop_arg!(args, AccountAddress);
    let raw = addr.into_bytes();
    let mut acc: u128 = 0;
    for b in raw.iter() {
        acc = (acc << 8) | (*b as u128);
    }
    Ok(smallvec![Value::u128(acc)])
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
pub fn make_all(
    builder: &SafeNativeBuilder,
) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
    let natives = [("address_to_u128", native_address_to_u128 as RawSafeNative)];

    builder.make_named_natives(natives)
}
