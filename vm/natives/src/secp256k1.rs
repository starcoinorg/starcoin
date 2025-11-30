// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

/***************************************************************************************************
 * native fun secp256k1_recover
 *
 *   gas cost: base_cost +? ecdsa_recover
 *
 **************************************************************************************************/

use crate::ecrecover::{keccak, pubkey_to_address};
use crate::util::make_native_from_func;
use arrayref::array_ref;
use libsecp256k1::{Message, PublicKey, SecretKey};
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
    pub secp256k1_sign: Secp256k1SignGasParameters,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Secp256k1SignGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

/// Abort code when deserialization fails (0x01 == INVALID_ARGUMENT)
/// NOTE: This must match the code in the Move implementation
///
pub mod abort_codes {
    pub const NFE_DESERIALIZE: u64 = 0x01_0001;
    pub const INVALID_PRIVKEY: u64 = 0x01_0002;
    pub const INVALID_HASH_FUNCTION: u64 = 0x01_0003;
}

// Hash function constants (must match Move implementation)
const HASH_KECCAK256: u8 = 0;
const HASH_SHA256: u8 = 1;

fn native_ecdsa_recover_internal(
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
    let compressed_key_bytes = pop_arg!(arguments, Vec<u8>);
    // Convert to fixed-size array
    let fixed_size_key: [u8; 33] = match compressed_key_bytes.try_into() {
        Ok(bytes) => bytes,
        Err(_) => return Ok(NativeResult::err(cost, abort_codes::NFE_DESERIALIZE)),
    };

    // Decompress the public key
    let pubkey = match PublicKey::parse_compressed(&fixed_size_key) {
        Ok(pubkey) => pubkey,
        Err(_) => return Ok(NativeResult::err(cost, abort_codes::NFE_DESERIALIZE)),
    };

    // Return the full 65-byte uncompressed public key (0x04 prefix + 64 bytes x,y coordinates)
    // This matches the expected format in Move contract committee.move
    Ok(NativeResult::ok(cost, smallvec![Value::vector_u8(pubkey.serialize().to_vec())]))
}

pub fn native_secp256k1_sign(
    gas_params: &Secp256k1SignGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 4);

    // The corresponding Move function is only used for testing, so we don't need to charge any gas.
    let cost = gas_params.base + gas_params.per_byte * NumBytes::one();

    // Parameters are popped in reverse order (last parameter first)
    let private_key_bytes = pop_arg!(args, Vec<u8>);
    let msg = pop_arg!(args, Vec<u8>);
    let hash = pop_arg!(args, u8);
    let recoverable = pop_arg!(args, bool);

    // Parse private key (must be 32 bytes)
    if private_key_bytes.len() != 32 {
        return Ok(NativeResult::err(cost, abort_codes::INVALID_PRIVKEY));
    }

    let seckey = match SecretKey::parse(array_ref![private_key_bytes, 0, 32]) {
        Ok(sk) => sk,
        Err(_) => return Ok(NativeResult::err(cost, abort_codes::INVALID_PRIVKEY)),
    };

    // Hash the message based on hash function type
    let msg_hash: [u8; 32] = match hash {
        HASH_KECCAK256 => keccak(&msg),
        HASH_SHA256 => {
            // Use sha3 for SHA3-256 hashing (project standard)
            use sha3::{Digest, Sha3_256};
            let mut hasher = Sha3_256::new();
            hasher.update(&msg);
            let result = hasher.finalize();
            result.into()
        }
        _ => return Ok(NativeResult::err(cost, abort_codes::INVALID_HASH_FUNCTION)),
    };

    // Parse the hashed message
    let message = match Message::parse_slice(&msg_hash) {
        Ok(msg) => msg,
        Err(_) => return Ok(NativeResult::err(cost, abort_codes::NFE_DESERIALIZE)),
    };

    // Sign the message
    let (sig, rec_id) = libsecp256k1::sign(&message, &seckey);
    let mut signature = sig.serialize().to_vec();

    // If recoverable, append recovery ID (in Ethereum format: rec_id + 27)
    if recoverable {
        signature.push(rec_id.serialize() + 27);
    }

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(signature)],
    ))
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "ecdsa_recover_internal",
            make_native_from_func(
                gas_params.ecdsa_recover_internal,
                native_ecdsa_recover_internal,
            ),
        ),
        (
            "decompress_pubkey",
            make_native_from_func(gas_params.decompress_pubkey, native_decompress_pubkey),
        ),
        (
            "secp256k1_sign",
            make_native_from_func(gas_params.secp256k1_sign, native_secp256k1_sign),
        ),
    ];

    crate::helpers::make_module_natives(natives)
}

#[test]
fn test_decompress() -> anyhow::Result<()> {
    use hex::FromHex;
    use libsecp256k1::PublicKey;

    let validator_pub_key = "029bef8d556d80e43ae7e0becb3a7e6838b95defe45896ed6075bb9035d06c9964";

    // Convert hex string to bytes
    let compressed_key_bytes: Vec<u8> = FromHex::from_hex(validator_pub_key)?;
    assert_eq!(
        compressed_key_bytes.len(),
        33,
        "Compressed public key must be 33 bytes"
    );

    // Convert to fixed-size array
    let fixed_size_key: [u8; 33] = compressed_key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to convert to fixed-size array"))?;

    // Decompress the public key
    let pubkey = PublicKey::parse_compressed(&fixed_size_key)
        .map_err(|e| anyhow::anyhow!("Failed to parse compressed public key: {:?}", e))?;

    // Get the full 65-byte uncompressed public key
    let uncompressed = pubkey.serialize();
    
    // Verify the uncompressed key is 65 bytes (0x04 prefix + 64 bytes)
    assert_eq!(uncompressed.len(), 65, "Uncompressed public key must be 65 bytes");
    assert_eq!(uncompressed[0], 0x04, "Uncompressed public key must start with 0x04");

    Ok(())
}
