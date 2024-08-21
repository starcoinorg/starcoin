use crate::traits::EXECUTION_GAS_MULTIPLIER as MUL;

use move_core_types::gas_algebra::{ InternalGas};

// see starcoin/vm/types/src/on_chain_config/genesis_gas_schedule.rs
// convert from https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#native_schedule
crate::macros::define_gas_parameters!(NurseryGasParameters, "nursery",
    NativeGasParameters => .nursery,
    [
    [event_write_to_event_store_unit_cost : InternalGas, "event.write_to_event_store.unit_cost", (52 + 1) * MUL],
    [debug_print_base_cost: InternalGas,  "debug.print.base_cost", MUL],
    [debug_print_stack_trace_base_cost: InternalGas,  "debug.print_stack_trace.base_cost",  MUL],
]);
