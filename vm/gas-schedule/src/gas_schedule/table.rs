// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas_schedule::NativeGasParameters;
use crate::traits::EXECUTION_GAS_MULTIPLIER as MUL;

use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte};

// same order as from https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#native_schedule
// modify should with impl From<VMConfig> for GasSchedule
// XXX FIXME YSG, need to modified
crate::gas_schedule::macros::define_gas_parameters!(TableGasParameters, "table",     NativeGasParameters => .table,
    [
    [new_table_handle_base: InternalGas,  "new_table_handle.base", (4 + 1) * MUL],

    [add_box_per_byte_serialized: InternalGasPerByte,  "add_box.per_byte_serialized",  (4 + 1) * MUL],

    [borrow_box_per_byte_serialized: InternalGasPerByte,  "borrow_box.per_byte_serialized", (10 + 1) * MUL],

    [remove_box_per_byte_serialized: InternalGasPerByte,  "remove_box.per_byte_serialized", (8 + 1) * MUL],

    [contains_box_per_byte_serialized: InternalGasPerByte,  "contains_box.per_byte_serialized", (40 + 1) * MUL],


    [destroy_empty_box_base:InternalGas ,  "destroy_empty_box.base", (20 + 1) * MUL],

    [drop_unchecked_box_base: InternalGas,  "drop_unchecked_box.base", (73 + 1) * MUL],
]);
