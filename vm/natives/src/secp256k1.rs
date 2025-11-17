// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

/***************************************************************************************************
 * native fun secp256k1_recover
 *
 *   gas cost: base_cost +? ecdsa_recover
 *
 **************************************************************************************************/

use crate::util::make_native_from_func;
use libsecp256k1::PublicKey;
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::NativeResult;
use move_vm_types::pop_arg;
use move_vm_types::values::Value;
use smallvec::smallvec;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GasParameters {
    pub ecdsa_recover_internal: Secp256k1EcdsaRecoverGasParameters,
    pub decompress_pubkey: DecompressPubKeyGasParameters,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Secp256k1EcdsaRecoverGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompressPubKeyGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

/// Abort code when deserialization fails (0x01 == INVALID_ARGUMENT)
/// NOTE: This must match the code in the Move implementation
///
pub mod abort_codes {
    pub const NFE_DESERIALIZE: u64 = 0x01_0001;
}

fn native_ecdsa_recover(
    gas_params: &Secp256k1EcdsaRecoverGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 3);

    let signature = pop_arg!(arguments, Vec<u8>);
    let recovery_id = pop_arg!(arguments, u8);
    let msg = pop_arg!(arguments, Vec<u8>);

    let mut cost = gas_params.base;

    // NOTE(Gas): O(1) cost
    // (In reality, O(|msg|) deserialization cost, with |msg| < libsecp256k1_core::util::MESSAGE_SIZE
    // which seems to be 32 bytes, so O(1) cost for all intents and purposes.)
    let msg = match libsecp256k1::Message::parse_slice(&msg) {
        Ok(msg) => msg,
        Err(_) => {
            return Ok(NativeResult::err(cost, abort_codes::NFE_DESERIALIZE));
        }
    };

    // NOTE(Gas): O(1) cost
    let rid = match libsecp256k1::RecoveryId::parse(recovery_id) {
        Ok(rid) => rid,
        Err(_) => {
            return Ok(NativeResult::err(cost, abort_codes::NFE_DESERIALIZE));
        }
    };

    // NOTE(Gas): O(1) deserialization cost
    // which seems to be 64 bytes, so O(1) cost for all intents and purposes.
    let sig = match libsecp256k1::Signature::parse_standard_slice(&signature) {
        Ok(sig) => sig,
        Err(_) => {
            return Ok(NativeResult::err(cost, abort_codes::NFE_DESERIALIZE));
        }
    };

    cost += gas_params.per_byte * NumBytes::one();

    // NOTE(Gas): O(1) cost: a size-2 multi-scalar multiplication
    match libsecp256k1::recover(&msg, &sig, &rid) {
        Ok(pk) => Ok(NativeResult::ok(
            cost,
            smallvec![
                Value::vector_u8(pk.serialize()[1..].to_vec()),
                Value::bool(true)
            ],
        )),
        Err(_) => Ok(NativeResult::ok(
            cost,
            smallvec![Value::vector_u8([0u8; 0]), Value::bool(false)],
        )),
    }
}

fn native_decompress_pubkey(
    gas_params: &DecompressPubKeyGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let cost = gas_params.base + gas_params.per_byte * NumBytes::one();

    let compressed_key = pop_arg!(arguments, Vec<u8>);
    if compressed_key.len() != 33 {
        return Ok(NativeResult::err(cost, abort_codes::NFE_DESERIALIZE));
    }

    let fixed_size_key: [u8; 33] = match compressed_key.try_into() {
        Ok(arr) => arr,
        Err(_) => return Ok(NativeResult::err(cost, abort_codes::NFE_DESERIALIZE)),
    };

    match PublicKey::parse_compressed(&fixed_size_key) {
        Ok(pk) => Ok(NativeResult::ok(
            cost,
            smallvec![Value::vector_u8(pk.serialize()[1..].to_vec())],
        )),
        Err(_) => Ok(NativeResult::ok(
            cost,
            smallvec![Value::vector_u8([0u8; 0])],
        )),
    }
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "ecdsa_recover_internal",
            make_native_from_func(gas_params.ecdsa_recover_internal, native_ecdsa_recover),
        ),
        (
            "decompress_pubkey",
            make_native_from_func(gas_params.decompress_pubkey, native_decompress_pubkey),
        ),
    ];

    crate::helpers::make_module_natives(natives)
}
