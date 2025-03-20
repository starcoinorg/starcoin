// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_meter::EXECUTION_GAS_MULTIPLIER as MUL;
use move_table_extension::GasParameters;

// same order as from https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#native_schedule
// modify should with impl From<VMConfig> for GasSchedule
crate::natives::define_gas_parameters_for_natives!(GasParameters, "table", [
    // Note(Gas): These are legacy parameters for loading from storage so they do not
    //            need to be multiplied.

    [.new_table_handle.base, optional "new_table_handle.base", (4 + 1) * MUL],

    [.add_box.per_byte_serialized, optional "add_box.per_byte_serialized",  (4 + 1) * MUL],

    [.borrow_box.per_byte_serialized, optional "borrow_box.per_byte_serialized", (10 + 1) * MUL],

    [.remove_box.per_byte_serialized, optional "remove_box.per_byte_serialized", (8 + 1) * MUL],

    [.contains_box.per_byte_serialized, optional "contains_box.per_byte_serialized", (40 + 1) * MUL],


    [.destroy_empty_box.base, optional "destroy_empty_box.base", (20 + 1) * MUL],

    [.drop_unchecked_box.base, optional "drop_unchecked_box.base", (73 + 1) * MUL],
], allow_unmapped = 4 /* table */ + 3 /* common */);
