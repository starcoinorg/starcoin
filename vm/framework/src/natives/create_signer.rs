// Copyright © Starcoin Foundation
// SPDX-License-Identifier: Apache-2.0

use move_core_types::account_address::AccountAddress;
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{loaded_data::runtime_types::Type, values::Value};
use smallvec::{smallvec, SmallVec};
use starcoin_gas_schedule::gas_params::natives::starcoin_framework::*;
use starcoin_native_interface::{
    safely_pop_arg, RawSafeNative, SafeNativeBuilder, SafeNativeContext, SafeNativeResult,
};
use std::collections::VecDeque;

/***************************************************************************************************
 * native fun create_signer
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
pub(crate) fn native_create_signer(
    context: &mut SafeNativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    context.charge(ACCOUNT_CREATE_SIGNER_BASE)?;

    let address = safely_pop_arg!(arguments, AccountAddress);
    Ok(smallvec![Value::signer(address)])
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
pub fn make_all(
    builder: &SafeNativeBuilder,
) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
    let natives = [("create_signer", native_create_signer as RawSafeNative)];

    builder.make_named_natives(natives)
}
