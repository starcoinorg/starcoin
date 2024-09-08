// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_schedule::NativeGasParameters;
use crate::traits::EXECUTION_GAS_MULTIPLIER as MUL;
use move_core_types::gas_algebra::InternalGasPerArg;
use starcoin_gas_algebra::{InternalGas, InternalGasPerByte};

// see starcoin/vm/types/src/on_chain_config/genesis_gas_schedule.rs
// same order as from https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#native_schedule
// modify should with impl From<VMConfig> for GasSchedule
crate::gas_schedule::macros::define_gas_parameters!(StarcoinFrameworkGasParameters, "starcoin_framework",
    NativeGasParameters => .starcoin_framework,
    [
    [signature_ed25519_pubkey_base: InternalGas,  "signature.ed25519.pubkey.base", 0 * MUL],
    [signature_ed25519_pubkey_per_byte: InternalGasPerByte,  "signature.ed25519.pubkey.per_byte", (61 + 1)* MUL],
     [signature_ed25519_verify_base : InternalGas,  "signature.ed25519.verify.base", 0 * MUL],
    [signature_ed25519_verify_per_byte: InternalGasPerByte,  "signature.ed25519.verify.per_byte",(26 + 1) * MUL],

    [account_create_signer_base: InternalGas, "account.create_signer.base", (24 + 1) * MUL],
    [account_destroy_signer_base: InternalGas, "account.destroy_signer.base", (212 + 1)* MUL],

    [token_name_of_base: InternalGas,  "token.name_of.base", (2002 + 1) * MUL],

    [hash_keccak256_base: InternalGas, "hash.keccak256.base",  0 * MUL],
    [hash_keccak256_per_byte: InternalGasPerByte, "hash.keccak256.per_byte",  (64 + 1) *MUL],
        [hash_ripemd160_base:InternalGas ,   "hash.ripemd160.base", 0 * MUL],
    [hash_ripemd160_per_byte: InternalGasPerByte,  "hash.ripemd160.per_byte", (64 + 1) * MUL],
     [signature_ec_recover_base: InternalGas,   "signature.ec_recover.base",  0 * MUL],
    [signature_ec_recover_per_byte: InternalGasPerByte,   "signature.ec_recover.per_byte", (128 + 1) * MUL],

       // XXX FIXME YSG, need to remove?
    //[u256_from_bytes_base,   "u256.from_bytes.base",  0 * MUL],
    [u256_from_bytes_per_byte: InternalGasPerByte,   "u256.from_bytes.per_byte", (2 + 1) * MUL],
    [u256_add_base: InternalGas,   "u256.add.base", (4 + 1) * MUL],
    [u256_sub_base: InternalGas,   "u256.sub.base",  (4 + 1) * MUL],
    [u256_mul_base: InternalGas,   "u256.mul.base",  (4 + 1) * MUL],
    [u256_div_base: InternalGas,   "u256.div.base",  (10 + 1) * MUL],
    [u256_rem_base: InternalGas,   "u256.rem.base",  (4 + 1) * MUL],
    [u256_pow_base: InternalGas,   "u256.pow.base",  (8 + 1) * MUL],
        // XXX FIXME YSG, need to remove?

    [util_from_bytes_base: InternalGas,  "util.from_bytes.base", (4 + 1)  * MUL],
          [util_from_bytes_per_byte: InternalGasPerByte, "util.from_bytes.per_byte", 0],
    [secp256k1_base: InternalGas,  "secp256k1.base", (4 + 1)  * MUL],
                [secp256k1_ecdsa_recover: InternalGasPerArg, "secp256k1.ecdsa_recover", 5918360],

        [code_request_publish_base: InternalGas, "code.request_publish.base", 1838],
        [code_request_publish_per_byte: InternalGasPerByte, "code.request_publish.per_byte", 7],
]);
