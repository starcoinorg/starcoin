// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::gas_algebra::InternalGasPerByte;
use starcoin_gas_algebra::InternalGas;

use crate::gas_schedule::NativeGasParameters;
use crate::traits::EXECUTION_GAS_MULTIPLIER as MUL;

// see starcoin/vm/types/src/on_chain_config/genesis_gas_schedule.rs
// same order as from https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#native_schedule
// modify should with impl From<VMConfig> for GasSchedule
crate::gas_schedule::macros::define_gas_parameters!(
  StarcoinFrameworkLegacyGasParameters,
  "starcoin_framework_legacy",
  NativeGasParameters => .starcoin_framework_legacy,
  [
    [signature_ed25519_verify_per_byte: InternalGasPerByte,  "signature.ed25519_verify.per_byte", (61 + 1)* MUL],
    [signature_ed25519_validate_key_per_byte: InternalGasPerByte, "signature.ed25519_validate_key_per_byte",(26 + 1) * MUL],
    [account_create_signer_base: InternalGas, "account.create_signer_base", (24 + 1) * MUL],
    [account_destroy_signer_base: InternalGas, "account.destroy_signer_base", (212 + 1)* MUL],
    [token_name_of_base:InternalGas,  "token.name_of_base", (2002 + 1) * MUL],
    [hash_keccak256_per_byte:InternalGas,  "hash.keccak256_per_byte",  (64 + 1) *MUL],
    [hash_ripemd160_per_byte:InternalGas ,  "hash.ripemd160_per_byte", (64 + 1) * MUL],
    [signature_ec_recover_per_byte:InternalGas,   "signature.ec_recover_per_byte", (128 + 1) * MUL],

    [u256_from_bytes_per_byte: InternalGasPerByte,   "u256.from_bytes.per_byte", (2 + 1) * MUL],
    [u256_add_base: InternalGas,   "u256.add.base", (4 + 1) * MUL],
    [u256_sub_base:InternalGas,   "u256.sub.base",  (4 + 1) * MUL],
    [u256_mul_base:InternalGas,   "u256.mul.base",  (4 + 1) * MUL],
    [u256_div_base:InternalGas,   "u256.div.base",  (10 + 1) * MUL],
    [u256_rem_base:InternalGas,   "u256.rem.base",  (4 + 1) * MUL],
    [u256_pow_base:InternalGas,   "u256_pow_base",  (8 + 1) * MUL],
    [from_bcs_base:InternalGas,  "from_bcs.base", (4 + 1)  * MUL],
    [secp256k1_base:InternalGas,  "secp256k1.base", (4 + 1)  * MUL],
  ]
);
