// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_meter::EXECUTION_GAS_MULTIPLIER as MUL;
use move_stdlib::natives::GasParameters;

// see starcoin/vm/types/src/on_chain_config/genesis_gas_schedule.rs
// convert from https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#native_schedule
// modify should with impl From<VMConfig> for GasSchedule
crate::natives::define_gas_parameters_for_natives!(GasParameters, "move_stdlib", [


    [.hash.sha2_256.base, "hash.sha2_256.base", 21 * MUL],
    [.hash.sha2_256.per_byte, "hash.sha2_256.per_byte", MUL],
    [.hash.sha3_256.base, "hash.sha3_256.base", 64 * MUL],
    [.hash.sha3_256.per_byte, "hash.sha3_256.per_byte",  MUL],

    [.bcs.to_bytes.per_byte_serialized, "bcs.to_bytes.per_byte_serialized", 10 * MUL],
    [.bcs.to_bytes.failure, "bcs.to_bytes.failure", 1000 * MUL],

    [.vector.length.base, "vector.length.base", 98 * MUL],
    [.vector.empty.base, "vector.empty.base", 84 * MUL],
    [.vector.borrow.base, "vector.borrow.base", 1334 * MUL],
    [.vector.push_back.base, "vector.push_back.base", 53 * MUL],
    [.vector.pop_back.base, "vector.pop_back.base", 227 * MUL],
    [.vector.destroy_empty.base, "vector.destroy_empty.base", 572 * MUL],
    [.vector.swap.base, "vector.swap.base", 1436 * MUL],

    // Note(Gas): this initial value is guesswork.
    [.signer.borrow_address.base, "signer.borrow_address.base", 353 * MUL],

    [.vector.append.base, "vector.append.base", 40 * MUL],
    [.vector.remove.base, "vector.remove.base", 20 * MUL],
    [.vector.reverse.base, "vector.reverse.base", 10 * MUL],

    // Note(Gas): these initial values are guesswork.
    [.string.check_utf8.base, "string.check_utf8.base", 4 * MUL],
    [.string.check_utf8.per_byte, "string.check_utf8.per_byte",  MUL],
    [.string.is_char_boundary.base, "string.is_char_boundary.base", 4 * MUL],
    [.string.sub_string.base, "string.sub_string.base", 4 * MUL],
    [.string.sub_string.per_byte, "string.sub_string.per_byte",  MUL],
    [.string.index_of.base, "string.index_of.base", 4 * MUL],
    [.string.index_of.per_byte_pattern, "string.index_of.per_byte_pattern", MUL],
    [.string.index_of.per_byte_searched, "string.index_of.per_byte_searched", MUL],
]);
