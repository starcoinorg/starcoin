// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_core_types::account_address::AccountAddress;
use move_core_types::gas_algebra::InternalGas;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
};
use smallvec::smallvec;
use std::collections::VecDeque;
use std::sync::Arc;

/***************************************************************************************************
 * native fun create_signer
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct CreateSignerGasParameters {
    pub base: InternalGas,
}

pub fn native_create_signer(
    gas_params: &CreateSignerGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let address = pop_arg!(arguments, AccountAddress);
    Ok(NativeResult::ok(
        gas_params.base,
        smallvec![Value::signer(address)],
    ))
}

pub fn make_native_create_signer(gas_params: CreateSignerGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_create_signer(&gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * native fun destroy_signer
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct DestroySignerGasParameters {
    pub base: InternalGas,
}

/// NOTE: this function will be deprecated after the Diem v3 release, but must
/// remain for replaying old transactions
pub fn native_destroy_signer(
    gas_params: &DestroySignerGasParameters,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    Ok(NativeResult::ok(gas_params.base, smallvec![]))
}

pub fn make_native_destroy_signer(gas_params: DestroySignerGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_destroy_signer(&gas_params, context, ty_args, args)
    })
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub create_signer: CreateSignerGasParameters,
    pub destroy_signer: DestroySignerGasParameters,
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "create_signer",
            make_native_create_signer(gas_params.create_signer),
        ),
        (
            "destroy_signer",
            make_native_destroy_signer(gas_params.destroy_signer),
        ),
    ];

    crate::helpers::make_module_natives(natives)
}
