// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_meter::EXECUTION_GAS_MULTIPLIER as MUL;
use move_stdlib::natives::GasParameters;

#[cfg(all(test, not(feature = "testing")))]
const UNIT_TEST_ENTRIES: usize = 0;

#[cfg(all(test, feature = "testing"))]
const UNIT_TEST_ENTRIES: usize = 2;

// see starcoin/vm/types/src/on_chain_config/genesis_gas_schedule.rs
// same order as https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#native_schedule
// modify should with impl From<VMConfig> for GasSchedule
crate::natives::define_gas_parameters_for_natives!(GasParameters, "move_stdlib", [


    // [.hash.sha2_256.base, "hash.sha2_256.base", 0 * MUL],
    [.hash.sha2_256.per_byte,  "hash.sha2_256.per_byte", (21 + 1) * MUL],
    [.hash.sha2_256.legacy_min_input_len,  "hash.sha2_256.legacy_min_input_len", MUL],
   // [.hash.sha3_256.base, "hash.sha3_256.base", 0 * MUL],
    [.hash.sha3_256.per_byte,  "hash.sha3_256.per_byte",  (64 + 1) * MUL],
    [.hash.sha3_256.legacy_min_input_len,  "hash.sha3_256.legacy_min_input_len",  MUL],

    [.bcs.to_bytes.per_byte_serialized, "bcs.to_bytes.per_byte_serialized", (181 + 1) * MUL],
    [.bcs.to_bytes.failure, "bcs.to_bytes.failure", (181 + 1) * MUL],
    [.bcs.to_bytes.legacy_min_output_size,  "bcs.to_bytes.legacy_min_output_size",  MUL],

    // Note(Gas): this initial value is guesswork.
    [.signer.borrow_address.base, "signer.borrow_address.base", (353 + 1) * MUL],
    // [.bcs.to_address.base, "bcs.to_address.base", 0 * MUL],
    [.bcs.to_address.per_byte, "bcs.to_address.per_byte", (26 + 1) *MUL],

    // Note(Gas): these initial values are guesswork.
   // [.string.check_utf8.base, optional "string.check_utf8.base", 0 * MUL],
    [.string.check_utf8.per_byte, optional "string.check_utf8.per_byte", (4 + 1) *  MUL],
    [.string.is_char_boundary.base, optional "string.is_char_boundary.base", (4 + 1) * MUL],
    // [.string.sub_string.base, optional "string.sub_string.base", 0 * MUL],
    [.string.sub_string.per_byte, optional "string.sub_string.per_byte", (4 + 1) *  MUL],
    // [.string.index_of.base, optional "string.index_of.base", 0 * MUL],
    [.string.index_of.per_byte_searched, optional "string.index_of.per_byte_searched", (4 + 1)  * MUL],
    // [.vector.spawn_from.base, optional "vector.spawn_from.base", 0  * MUL],
], allow_unmapped = 2 /* bcs */ + 2 /* hash */ + 5 /* vector */ + 3 /* string */ + 2 /* type_name */ + UNIT_TEST_ENTRIES);
