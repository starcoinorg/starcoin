// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::natives::util::make_native_from_func;
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, pop_arg, values::Value,
};
use ripemd160::digest::Output;
use ripemd160::{Digest, Ripemd160};
use smallvec::smallvec;
use std::collections::VecDeque;

/***************************************************************************************************
 * native fun native_keccak_256
 *
 *   gas cost: base_cost + per_byte * data_length
 *
 **************************************************************************************************/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keccak256HashGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

pub fn native_keccak_256(
    gas_params: &Keccak256HashGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let input_arg = pop_arg!(arguments, Vec<u8>);

    let cost = gas_params.base + gas_params.per_byte * NumBytes::new(input_arg.len() as u64);

    let output = crate::natives::ecrecover::keccak(input_arg.as_slice());

    Ok(NativeResult::ok(cost, smallvec![Value::vector_u8(output)]))
}

/***************************************************************************************************
 * native fun native_ripemd160
 *
 *   gas cost: base_cost + per_byte * data_length
 *
 **************************************************************************************************/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ripemd160HashGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

pub fn native_ripemd160(
    gas_params: &Ripemd160HashGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let input_arg = pop_arg!(arguments, Vec<u8>);

    let cost = gas_params.base + gas_params.per_byte * NumBytes::new(input_arg.len() as u64);

    let result = ripemd160(input_arg.as_slice());
    Ok(NativeResult::ok(cost, smallvec![Value::vector_u8(result)]))
}

fn ripemd160(input: &[u8]) -> Output<Ripemd160> {
    let mut hasher = Ripemd160::new();
    hasher.update(input);
    hasher.finalize()
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GasParameters {
    pub keccak256: Keccak256HashGasParameters,
    pub ripemd160: Ripemd160HashGasParameters,
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "keccak_256",
            make_native_from_func(gas_params.keccak256, native_keccak_256),
        ),
        (
            "ripemd160",
            make_native_from_func(gas_params.ripemd160, native_ripemd160),
        ),
    ];

    crate::natives::helpers::make_module_natives(natives)
}

#[cfg(test)]
mod test {
    use super::*;
    use hex::FromHex;

    #[test]
    fn test_keccak() {
        let input: Vec<u8> = FromHex::from_hex("616263").unwrap();
        let output = crate::natives::ecrecover::keccak(input.as_slice());
        let expect_output: Vec<u8> =
            FromHex::from_hex("4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45")
                .unwrap();
        assert_eq!(expect_output, output);
    }

    #[test]
    fn test_ripemd160() {
        let input: Vec<u8> = FromHex::from_hex("616263").unwrap();
        let output = ripemd160(input.as_slice()).to_vec();
        let expect_output: Vec<u8> =
            FromHex::from_hex("8eb208f7e05d987a9b044a8e98c6b087f15a0bfc").unwrap();
        assert_eq!(expect_output, output);
    }
}
