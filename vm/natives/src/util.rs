// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
};
use std::{collections::VecDeque, sync::Arc};

/// Used to pass gas parameters into native functions.
pub fn make_native_from_func<T: std::marker::Send + std::marker::Sync + 'static>(
    gas_params: T,
    func: fn(&T, &mut NativeContext, Vec<Type>, VecDeque<Value>) -> PartialVMResult<NativeResult>,
) -> NativeFunction {
    Arc::new(move |context, ty_args, args| func(&gas_params, context, ty_args, args))
}
