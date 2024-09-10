// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::natives::ecrecover::native_ecrecover;
use move_core_types::gas_algebra::NumBytes;
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{loaded_data::runtime_types::Type, values::Value};
use smallvec::{smallvec, SmallVec};
use starcoin_crypto::{ed25519, traits::*};
use starcoin_gas_schedule::gas_params::natives::starcoin_framework::*;
use starcoin_native_interface::{
    safely_pop_arg, RawSafeNative, SafeNativeBuilder, SafeNativeContext, SafeNativeResult,
};
use std::{collections::VecDeque, convert::TryFrom};

pub fn native_ed25519_validate_pubkey(
    context: &mut SafeNativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let key = safely_pop_arg!(arguments, Vec<u8>);

    let cost = SIGNATURE_ED25519_PUBKEY_BASE
        + SIGNATURE_ED25519_PUBKEY_PER_BYTE * NumBytes::new(key.len() as u64);
    context.charge(cost)?;

    // This deserialization performs point-on-curve and small subgroup checks
    let valid = ed25519::Ed25519PublicKey::try_from(&key[..]).is_ok();
    Ok(smallvec![Value::bool(valid)])
}

pub fn native_ed25519_verify(
    context: &mut SafeNativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 3);

    let msg = safely_pop_arg!(arguments, Vec<u8>);
    let pubkey = safely_pop_arg!(arguments, Vec<u8>);
    let signature = safely_pop_arg!(arguments, Vec<u8>);

    let cost = SIGNATURE_ED25519_VERIFY_BASE
        + SIGNATURE_ED25519_VERIFY_PER_BYTE * NumBytes::new(msg.len() as u64);

    context.charge(cost)?;

    let sig = match ed25519::Ed25519Signature::try_from(signature.as_slice()) {
        Ok(sig) => sig,
        Err(_) => return Ok(smallvec![Value::bool(false)]),
    };
    let pk = match ed25519::Ed25519PublicKey::try_from(pubkey.as_slice()) {
        Ok(pk) => pk,
        Err(_) => return Ok(smallvec![Value::bool(false)]),
    };

    let verify_result = sig.verify_arbitrary_msg(msg.as_slice(), &pk).is_ok();
    Ok(smallvec![Value::bool(verify_result)])
}

pub fn make_all(
    builder: &SafeNativeBuilder,
) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
    let natives = [
        (
            "ed25519_validate_pubkey",
            native_ed25519_validate_pubkey as RawSafeNative,
        ),
        ("ed25519_verify", native_ed25519_verify),
        ("native_ecrecover", native_ecrecover),
    ];

    builder.make_named_natives(natives)
}
