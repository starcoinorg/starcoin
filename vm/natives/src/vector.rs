use move_binary_format::errors::PartialVMResult;
use move_vm_runtime::native_functions::NativeContext;
use move_vm_types::loaded_data::runtime_types::Type;
use move_vm_types::natives::function::{native_gas, NativeResult};
use move_vm_types::pop_arg;
use move_vm_types::values::{Value, VectorRef};
use starcoin_vm_types::gas_schedule::NativeCostIndex;
use std::collections::VecDeque;

pub fn native_append(
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 2);

    let other = args.pop_back().unwrap();
    let lhs = pop_arg!(args, VectorRef);
    let cost = native_gas(context.cost_table(), NativeCostIndex::VEC_APPEND as u8, 1);
    lhs.append(cost, other.value_as()?, &ty_args[0])
}

pub fn native_remove(
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 2);

    let idx: u64 = args.pop_back().unwrap().value_as()?;
    let lhs = pop_arg!(args, VectorRef);

    let cost = native_gas(context.cost_table(), NativeCostIndex::VEC_REMOVE as u8, 1);
    lhs.remove(idx as usize, cost, &ty_args[0])
}

pub fn native_reverse(
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);
    let lhs = pop_arg!(args, VectorRef);
    let cost = native_gas(context.cost_table(), NativeCostIndex::VEC_REVERSE as u8, 1);
    lhs.reverse(cost, &ty_args[0])
}
