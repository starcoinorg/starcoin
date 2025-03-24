// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// use crate::natives::create_signer;
use move_core_types::account_address::AccountAddress;
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{loaded_data::runtime_types::Type, values::Value};
use smallvec::{smallvec, SmallVec};
use starcoin_gas_schedule::gas_params::natives::starcoin_framework_legacy::*;
use starcoin_native_interface::{
    safely_pop_arg, SafeNativeBuilder, SafeNativeContext, SafeNativeResult,
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
    let natives = [
        // Despite that this is no longer present in account.move, we must keep this around for
        // replays.
        ("create_signer", native_create_signer),
    ];

    builder.make_named_natives(natives)
}