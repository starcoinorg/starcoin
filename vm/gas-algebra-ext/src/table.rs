// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_meter::EXECUTION_GAS_MULTIPLIER as MUL;
use move_table_extension::GasParameters;

// modify should with impl From<VMConfig> for GasSchedule
crate::natives::define_gas_parameters_for_natives!(GasParameters, "table", [
    // Note(Gas): These are legacy parameters for loading from storage so they do not
    //            need to be multiplied.

    [.new_table_handle.base, "new_table_handle.base", 4 * MUL],

    [.add_box.base, "add_box.base", 4 * MUL],
    [.add_box.per_byte_serialized, optional "add_box.per_byte_serialized",  MUL],

    [.borrow_box.base, "borrow_box.base", 10 * MUL],
    [.borrow_box.per_byte_serialized, optional "borrow_box.per_byte_serialized", MUL],

    [.remove_box.base, "remove_box.base", 8 * MUL],
    [.remove_box.per_byte_serialized, optional "remove_box.per_byte_serialized", MUL],

    [.contains_box.base, "contains_box.base", 40 * MUL],
    [.contains_box.per_byte_serialized, optional "contains_box.per_byte_serialized", MUL],


    [.destroy_empty_box.base, "destroy_empty_box.base", 20 * MUL],

    [.drop_unchecked_box.base, "drop_unchecked_box.base", 73 * MUL],
]);
