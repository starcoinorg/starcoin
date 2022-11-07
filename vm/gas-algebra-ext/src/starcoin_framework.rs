// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_meter::EXECUTION_GAS_MULTIPLIER as MUL;
use starcoin_natives::GasParameters;

crate::natives::define_gas_parameters_for_natives!(GasParameters, "starcoin_natives", [
    // XXX FIXME YSG
    [.account.create_signer.base, "account.create_signer.base", 300 * MUL],
    [.account.destroy_signer.base, "account.destroy_signer.base", 300* MUL],
        // XXX FIXME YSG
    [.hash.keccak256.base, optional "hash.keccak256.base", 4000 * MUL],
    [.hash.keccak256.per_byte, optional "hash.keccak256.per_byte", 45 * MUL],
    [.hash.ripemd160.base, optional "hash.ripemd160.base", 4000 * MUL],
    [.hash.ripemd160.per_byte, optional "hash.ripemd160.per_byte", 45 * MUL],
    [.signature.ed25519_validate_publickey.base, optional "signature.ed25519_validate_publickey.base", 4000 * MUL],
    [.signature.ed25519_validate_publickey.per_byte, optional "signature.ed25519_validate_publickey.per_byte", 45 * MUL],
    [.signature.ed25519_verify.base, optional "signature.ed25519_verify.base", 4000 * MUL],
    [.signature.ed25519_verify.per_byte, optional "signature.ed25519_verify.per_byte", 45 * MUL],
    [.signature.ec_recover.base, optional "signature.ec_recover.base", 4000 * MUL],
    [.token.address.base, optional "token.address.base", 300 * MUL],
    [.u256.add.base, optional "u256.add.base", 300 * MUL],
    [.u256.sub.base, optional "u256.sub.base", 300 * MUL],
    [.u256.mul.base, optional "u256.mul.base", 300 * MUL],
    [.u256.div.base, optional "u256.div.base", 300 * MUL],
    [.u256.rem.base, optional "u256.rem.base", 300 * MUL],
    [.u256.pow.base, optional "u256.pow.base", 300 * MUL],
    [.u256.from_bytes.base, optional "u256.from_bytes.base", 300 * MUL],
]);
