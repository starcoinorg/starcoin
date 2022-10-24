// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeResult},
    pop_arg,
    values::Value,
};
use smallvec::smallvec;
use starcoin_crypto::{ed25519, traits::*};
use starcoin_vm_types::gas_schedule::NativeCostIndex;
use std::{collections::VecDeque, convert::TryFrom};
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};

/***************************************************************************************************
 * native fun Ed25519PublickeyValidation
 *
 *   gas cost: base_cost + unit_cost * data_length
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct Ed25519PublickeyValidationGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

pub fn native_ed25519_publickey_validation(
    gas_params: Ed25519PublickeyValidationGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let key = pop_arg!(arguments, Vec<u8>);

    let cost = gas_params.base + gas_params.per_byte * NumBytes::new(key.len() as u64);

    // This deserialization performs point-on-curve and small subgroup checks
    let valid = ed25519::Ed25519PublicKey::try_from(&key[..]).is_ok();
    Ok(NativeResult::ok(cost, smallvec![Value::bool(valid)]))
}

/***************************************************************************************************
 * native fun Ed25519PublickeyValidation
 *
 *   gas cost: base_cost + unit_cost * data_length
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct Ed25519SignatureVerificationGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

pub fn native_ed25519_signature_verification(
    gas_params: Ed25519SignatureVerificationGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 3);

    let msg = pop_arg!(arguments, Vec<u8>);
    let pubkey = pop_arg!(arguments, Vec<u8>);
    let signature = pop_arg!(arguments, Vec<u8>);

    let cost = gas_params.base + gas_params.per_byte * NumBytes::new(msg.len() as u64);

    let sig = match ed25519::Ed25519Signature::try_from(signature.as_slice()) {
        Ok(sig) => sig,
        Err(_) => {
            return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
        }
    };
    let pk = match ed25519::Ed25519PublicKey::try_from(pubkey.as_slice()) {
        Ok(pk) => pk,
        Err(_) => {
            return Ok(NativeResult::ok(cost, smallvec![Value::bool(false)]));
        }
    };

    let verify_result = sig.verify_arbitrary_msg(msg.as_slice(), &pk).is_ok();
    Ok(NativeResult::ok(
        cost,
        smallvec![Value::bool(verify_result)],
    ))
}


/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub ed25519_publickey_validation: Ed25519PublickeyValidationGasParameters,
    pub ed25519_signature_verification: Ed25519SignatureVerificationGasParameters,
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "ed25519_publickey_validation",
            make_native_from_func(gas_params.ed25519_publickey_validation, native_ed25519_publickey_validation),
        ),
        (
            "ed25519_signature_verification",
            make_native_from_func(gas_params.ed25519_signature_verification, native_ed25519_signature_verification),
        ),
    ];

    crate::natives::helpers::make_module_natives(natives)
}
