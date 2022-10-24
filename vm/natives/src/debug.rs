// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_schedule::ONE_GAS_UNIT;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
#[allow(unused_imports)]
use move_vm_types::values::{values_impl::debug::print_reference, Reference};
#[allow(unused_imports)]
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
};
use smallvec::smallvec;
use std::collections::VecDeque;
use move_core_types::gas_algebra::InternalGas;

/***************************************************************************************************
 * native fun print
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct PrintGasParameters {
    pub base: InternalGas,
}

pub fn native_print(
    gas_params: PrintGasParameters,
    _context: &mut NativeContext,
    mut ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);

    // No-op if the feature flag is not present.
    #[cfg(feature = "testing")]
    {
        let ty = ty_args.pop().unwrap();
        let r = pop_arg!(args, Reference);

        let mut buf = String::new();
        print_reference(&mut buf, &r)?;
        println!("[debug] {}", buf);
    }

    Ok(NativeResult::ok(gas_params.base, smallvec![]))
}

/***************************************************************************************************
 * native fun print_stack_trace
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct PrintStackTraceGasParameters {
    pub base: InternalGas,
}


#[allow(unused_variables)]
pub fn native_print_stack_trace(
    gas_params: PrintStackTraceGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.is_empty());

    #[cfg(feature = "testing")]
    {
        let mut s = String::new();
        context.print_stack_trace(&mut s)?;
        println!("{}", s);
    }

    Ok(NativeResult::ok(gas_params.base, smallvec![]))
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub print: PrintGasParameters,
    pub print_stack_trace: PrintStackTraceGasParameters,
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "print",
            make_native_from_func(gas_params.address, native_print),
        ),
        (
            "print_stack_trace",
            make_native_from_func(gas_params.address, native_print_stack_trace),
        ),
    ];

    crate::natives::helpers::make_module_natives(natives)
}