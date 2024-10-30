// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_schedule::NativeGasParameters;
use crate::traits::EXECUTION_GAS_MULTIPLIER as MUL;
use starcoin_gas_algebra::{InternalGas, InternalGasPerByte};

#[cfg(all(test, not(feature = "testing")))]
const UNIT_TEST_ENTRIES: usize = 0;

#[cfg(all(test, feature = "testing"))]
const UNIT_TEST_ENTRIES: usize = 2;

// see starcoin/vm/types/src/on_chain_config/genesis_gas_schedule.rs
// same order as https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#native_schedule
// modify should with impl From<VMConfig> for GasSchedule
crate::gas_schedule::macros::define_gas_parameters!(
    MoveStdlibGasParameters,
    "move_stdlib",
    NativeGasParameters => .move_stdlib,
    [
        [hash_sha2_256_base:InternalGas, "hash.sha2_256.base", 0 * MUL],
        [hash_sha2_256_per_byte: InternalGasPerByte, "hash.sha2_256.per_byte", (21 + 1) * MUL],
        [hash_sha2_256_legacy_min_input_len: InternalGas,  "hash.sha2_256.legacy_min_input_len", MUL],
        [hash_sha3_256_base:InternalGas, "hash.sha3_256.base", 0 * MUL],
        [hash_sha3_256_per_byte:InternalGasPerByte,  "hash.sha3_256.per_byte",  (64 + 1) * MUL],
        [hash_sha3_256_legacy_min_input_len: InternalGas,  "hash.sha3_256.legacy_min_input_len",  MUL],

        [bcs_to_bytes_per_byte_serialized:InternalGasPerByte, "bcs.to_bytes.per_byte_serialized", (181 + 1) * MUL],
        [bcs_to_bytes_failure:InternalGas, "bcs.to_bytes.failure", (181 + 1) * MUL],
        [bcs_to_bytes_legacy_min_output_size: InternalGas,  "bcs.to_bytes.legacy_min_output_size",  MUL],

        // Note(Gas): this initial value is guesswork.
        [signer_borrow_address_base:InternalGas, "signer.borrow_address.base", (353 + 1) * MUL],
        [bcs_to_address_base:InternalGas, "bcs.to_address.base", 0 * MUL],
        [bcs_to_address_per_byte:InternalGasPerByte, "bcs.to_address.per_byte", (26 + 1) *MUL],

        // Note(Gas): these initial values are guesswork.
        [string_check_utf8_base:InternalGas,  "string.check_utf8.base", 0 * MUL],
        [string_check_utf8_per_byte: InternalGasPerByte,  "string.check_utf8.per_byte", (4 + 1) *  MUL],
        [string_is_char_boundary_base: InternalGas,  "string.is_char_boundary.base", (4 + 1) * MUL],
        [string_sub_string_base:InternalGas,  "string.sub_string.base", 0 * MUL],
        [string_sub_string_per_byte:InternalGasPerByte,  "string.sub_string.per_byte", (4 + 1) *  MUL],
        [string_index_of_base:InternalGas,  "string.index_of.base", 0 * MUL],
        [string_index_of_per_byte_pattern: InternalGasPerByte, "string.index_of.per_byte_pattern", 73],
        [string_index_of_per_byte_searched:InternalGasPerByte,  "string.index_of.per_byte_searched", (4 + 1)  * MUL],
        [vector_spawn_from_base:InternalGas,  "vector.spawn_from.base", 0  * MUL],
    ]
);
