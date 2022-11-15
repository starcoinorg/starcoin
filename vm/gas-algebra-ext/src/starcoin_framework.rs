// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_meter::EXECUTION_GAS_MULTIPLIER as MUL;
use starcoin_natives::GasParameters;

// see starcoin/vm/types/src/on_chain_config/genesis_gas_schedule.rs
// convert from https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#native_schedule
// modify should with impl From<VMConfig> for GasSchedule
crate::natives::define_gas_parameters_for_natives!(GasParameters, "starcoin_natives", [
    [.signature.ed25519_verify.base,  "signature.ed25519_validate_key.base", 61 * MUL],
    [.signature.ed25519_verify.per_byte, optional "signature.ed25519_validate_key.per_byte", MUL],
    [.signature.ed25519_validate_key.base,  "signature.ed25519_verify.base", 61 * MUL],
    [.signature.ed25519_validate_key.per_byte, optional "signature.ed25519_verify.per_byte", MUL],

    [.account.create_signer.base, "account.create_signer.base", 24 * MUL],
    [.account.destroy_signer.base, "account.destroy_signer.base", 212* MUL],

    [.hash.ripemd160.base,  "hash.ripemd160.base", 64 * MUL],
    [.hash.ripemd160.per_byte, optional "hash.ripemd160.per_byte", MUL],
    [.hash.keccak256.base,  "hash.keccak256.base", 64 * MUL],
    [.hash.keccak256.per_byte, optional "hash.keccak256.per_byte",  MUL],
    [.signature.ec_recover.base,  "signature.ec_recover.base", 128 * MUL],
    [.signature.ec_recover.per_byte,  optional "signature.ec_recover.per_byte",  MUL],
    [.token.token_name_of.base,  "token.token_name_of.base", 2002 * MUL],

    [.u256.add.base,  "u256.add.base", 4 * MUL],
    [.u256.sub.base,  "u256.sub.base", 4 * MUL],
    [.u256.mul.base,  "u256.mul.base", 4 * MUL],
    [.u256.div.base,  "u256.div.base", 4 * MUL],
    [.u256.rem.base,  "u256.rem.base", 4 * MUL],
    [.u256.pow.base,  "u256.pow.base", 4 * MUL],
    [.u256.from_bytes.base,  "u256.from_bytes.base", 4 * MUL],
    [.u256.from_bytes.per_byte,  optional "u256.from_bytes.per_byte", MUL],
]);
