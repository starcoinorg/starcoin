// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_meter::EXECUTION_GAS_MULTIPLIER as MUL;
use starcoin_frameworks::natives::GasParameters;

// see starcoin/vm/types/src/on_chain_config/genesis_gas_schedule.rs
// same order as from https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#native_schedule
// modify should with impl From<VMConfig> for GasSchedule
crate::natives::define_gas_parameters_for_natives!(GasParameters, "starcoin_natives", [
    // [.signature.ed25519_verify.base,  "signature.ed25519_verify.base", 0 * MUL],
    [.signature.ed25519_verify.per_byte,  "signature.ed25519_verify.per_byte", (61 + 1)* MUL],
   // [.signature.ed25519_validate_key.base,  "signature.ed25519_validate_key.base", 0 * MUL],
    [.signature.ed25519_validate_key.per_byte, optional "signature.ed25519_validate_key.per_byte",(26 + 1) * MUL],

    [.account.create_signer.base, "account.create_signer.base", (24 + 1) * MUL],
    [.account.destroy_signer.base, "account.destroy_signer.base", (212 + 1)* MUL],

    [.token.name_of.base,  "token.name_of.base", (2002 + 1) * MUL],

   // [.hash.keccak256.base, optional "hash.keccak256.base",  0 * MUL],
    [.hash.keccak256.per_byte, optional "hash.keccak256.per_byte",  (64 + 1) *MUL],
  //  [.hash.ripemd160.base,  optional "hash.ripemd160.base", 0 * MUL],
    [.hash.ripemd160.per_byte, optional "hash.ripemd160.per_byte", (64 + 1) * MUL],
   // [.signature.ec_recover.base,  optional "signature.ec_recover.base",  0 * MUL],
    [.signature.ec_recover.per_byte,  optional "signature.ec_recover.per_byte", (128 + 1) * MUL],

    //[.u256.from_bytes.base,  optional "u256.from_bytes.base",  0 * MUL],
    [.u256.from_bytes.per_byte,  optional "u256.from_bytes.per_byte", (2 + 1) * MUL],
    [.u256.add.base,  optional "u256.add.base", (4 + 1) * MUL],
    [.u256.sub.base,  optional "u256.sub.base",  (4 + 1) * MUL],
    [.u256.mul.base,  optional "u256.mul.base",  (4 + 1) * MUL],
    [.u256.div.base,  optional "u256.div.base",  (10 + 1) * MUL],
    [.u256.rem.base,  optional "u256.rem.base",  (4 + 1) * MUL],
    [.u256.pow.base,  optional "u256.pow.base",  (8 + 1) * MUL],
    [.from_bcs.base, optional "frombcs.base", (4 + 1)  * MUL],
    [.secp256k1.base, optional "secp256k1.base", (4 + 1)  * MUL],
], allow_unmapped = 3 /* signature */ + 2 /* hash */ + 1 /* u256 */ + 1 /* from_bcs */ + 1 /* secp256k1 */);
