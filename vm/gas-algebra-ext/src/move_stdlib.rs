// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_meter::EXECUTION_GAS_MULTIPLIER as MUL;
use move_stdlib::natives::GasParameters;

// see starcoin/vm/types/src/on_chain_config/genesis_gas_schedule.rs
// same order as https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#native_schedule
// modify should with impl From<VMConfig> for GasSchedule
crate::natives::define_gas_parameters_for_natives!(GasParameters, "move_stdlib", [


    // [.hash.sha2_256.base, "hash.sha2_256.base", 0 * MUL],
    [.hash.sha2_256.per_byte,  "hash.sha2_256.per_byte", (21 + 1) * MUL],
    [.hash.sha2_256.legacy_min_input_len, optional "hash.sha2_256.legacy_min_input_len", MUL],
   // [.hash.sha3_256.base, "hash.sha3_256.base", 0 * MUL],
    [.hash.sha3_256.per_byte,  "hash.sha3_256.per_byte",  (64 + 1) * MUL],
    [.hash.sha3_256.legacy_min_input_len, optional "hash.sha3_256.legacy_min_input_len",  MUL],

    [.bcs.to_bytes.per_byte_serialized, "bcs.to_bytes.per_byte_serialized", (181 + 1) * MUL],
    [.bcs.to_bytes.failure, optional "bcs.to_bytes.failure", (181 + 1) * MUL],
    //[.bcs.to_bytes.legacy_min_output_size, optional "bcs.to_bytes.legacy_min_output_size", 1000 * MUL],

    [.vector.length.base, "vector.length.base", (98 + 1) * MUL],
    [.vector.empty.base, "vector.empty.base", (84 + 1) * MUL],
    [.vector.borrow.base, "vector.borrow.base", (1334 + 1) * MUL],
    // [.vector.push_back.base, "vector.push_back.base", 0 * MUL],
    [.vector.push_back.legacy_per_abstract_memory_unit, "vector.push_back.legacy_per_abstract_memory_unit", (53 + 1) * MUL],
    [.vector.pop_back.base, "vector.pop_back.base", (227 + 1) * MUL],
    [.vector.destroy_empty.base, "vector.destroy_empty.base", (572 + 1) * MUL],
    [.vector.swap.base, "vector.swap.base", (1436 + 1) * MUL],

    // Note(Gas): this initial value is guesswork.
    [.signer.borrow_address.base, "signer.borrow_address.base", (353 + 1) * MUL],
    // [.bcs.to_address.base, "bcs.to_address.base", 0 * MUL],
    [.bcs.to_address.per_byte, "bcs.to_address.per_byte", (26 + 1) *MUL],

    //[.vector.append.base, optional "vector.append.base", 0 * MUL],
    [.vector.append.legacy_per_abstract_memory_unit, optional "vector.append.legacy_per_abstract_memory_unit", (40 + 1) * MUL],
    //[.vector.remove.base, optional "vector.remove.base", 0 * MUL],
    [.vector.remove.legacy_per_abstract_memory_unit, optional "vector.remove.legacy_per_abstract_memory_unit", (20 + 1) * MUL],
    //[.vector.reverse.base, optional "vector.reverse.base", 0 * MUL],
     [.vector.reverse.legacy_per_abstract_memory_unit, optional "vector.reverse.legacy_per_abstract_memory_unit", (10 + 1) * MUL],
    // Note(Gas): these initial values are guesswork.
   // [.string.check_utf8.base, optional "string.check_utf8.base", 0 * MUL],
    [.string.check_utf8.per_byte, optional "string.check_utf8.per_byte", (4 + 1) *  MUL],
    [.string.is_char_boundary.base, optional "string.is_char_boundary.base", (4 + 1) * MUL],
    // [.string.sub_string.base, optional "string.sub_string.base", 0 * MUL],
    [.string.sub_string.per_byte, optional "string.sub_string.per_byte", (4 + 1) *  MUL],
    // [.string.index_of.base, optional "string.index_of.base", 0 * MUL],
    [.string.index_of.per_byte_pattern, optional "string.index_of.per_byte_pattern",(4 + 1) * MUL],
    [.string.index_of.per_byte_searched, optional "string.index_of.per_byte_searched", (4 + 1)  * MUL],
], allow_unmapped = 2 /* bcs */ + 2 /* hash */ + 4 /* vector */ + 3 /* string*/ +  2 /* XXX FIXME YSG for nextest*/);
