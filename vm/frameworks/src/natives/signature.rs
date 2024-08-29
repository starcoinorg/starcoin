// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::natives::ecrecover::{make_native_ecrecover, EcrecoverGasParameters};
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
};
use smallvec::smallvec;
use starcoin_crypto::{ed25519, traits::*};
use std::sync::Arc;
use std::{collections::VecDeque, convert::TryFrom};

/***************************************************************************************************
 * native fun Ed25519PublickeyValidation
 *
 *   gas cost: base_cost + unit_cost * data_length
 *
 **************************************************************************************************/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ed25519ValidateKeyGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

pub fn native_ed25519_publickey_validation(
    gas_params: &Ed25519ValidateKeyGasParameters,
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

pub fn make_native_ed25519_validate_pubkey(
    gas_params: Ed25519ValidateKeyGasParameters,
) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_ed25519_publickey_validation(&gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * native fun Ed25519PublickeyValidation
 *
 *   gas cost: base_cost + unit_cost * data_length
 *
 **************************************************************************************************/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ed25519VerifyGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

pub fn native_ed25519_signature_verification(
    gas_params: &Ed25519VerifyGasParameters,
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

pub fn make_native_ed25519_verify(gas_params: Ed25519VerifyGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_ed25519_signature_verification(&gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GasParameters {
    pub ed25519_validate_key: Ed25519ValidateKeyGasParameters,
    pub ed25519_verify: Ed25519VerifyGasParameters,
    pub ec_recover: EcrecoverGasParameters,
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "ed25519_validate_pubkey",
            make_native_ed25519_validate_pubkey(gas_params.ed25519_validate_key),
        ),
        (
            "ed25519_verify",
            make_native_ed25519_verify(gas_params.ed25519_verify),
        ),
        (
            "native_ecrecover",
            make_native_ecrecover(gas_params.ec_recover),
        ),
    ];

    crate::natives::helpers::make_module_natives(natives)
}
