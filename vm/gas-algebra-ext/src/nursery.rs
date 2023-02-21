use crate::gas_meter::EXECUTION_GAS_MULTIPLIER as MUL;
use move_stdlib::natives::NurseryGasParameters;

// see starcoin/vm/types/src/on_chain_config/genesis_gas_schedule.rs
// convert from https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#native_schedule
crate::natives::define_gas_parameters_for_natives!(NurseryGasParameters, "nursery", [
    [.event.write_to_event_store.unit_cost, "event.write_to_event_store.unit_cost", (52 + 1) * MUL],
    [.debug.print.base_cost, optional "debug.print.base_cost", MUL],
    [.debug.print_stack_trace.base_cost, optional "debug.print_stack_trace.base_cost",  MUL],
]);
