// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::PartialVMResult;
use move_vm_runtime::native_functions::NativeContext;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::{native_gas, NativeResult},
    pop_arg,
    values::Value,
};
use ripemd160::digest::Output;
use ripemd160::{Digest, Ripemd160};
use smallvec::smallvec;
use starcoin_vm_types::gas_schedule::NativeCostIndex;
use std::collections::VecDeque;

pub fn native_keccak_256(
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let input_arg = pop_arg!(arguments, Vec<u8>);

    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::KECCAK_256 as u8,
        input_arg.len(),
    );
    let output = crate::ecrecover::keccak(input_arg.as_slice());

    Ok(NativeResult::ok(cost, smallvec![Value::vector_u8(output)]))
}

pub fn native_ripemd160(
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let input_arg = pop_arg!(arguments, Vec<u8>);

    let cost = native_gas(
        context.cost_table(),
        NativeCostIndex::RIPEMD160 as u8,
        input_arg.len(),
    );

    let result = ripemd160(input_arg.as_slice());
    Ok(NativeResult::ok(cost, smallvec![Value::vector_u8(result)]))
}

fn ripemd160(input: &[u8]) -> Output<Ripemd160> {
    let mut hasher = Ripemd160::new();
    hasher.update(input);
    hasher.finalize()
}

#[cfg(test)]
mod test {
    use super::*;
    use hex::FromHex;

    #[test]
    fn test_keccak() {
        let input: Vec<u8> = FromHex::from_hex("616263").unwrap();
        let output = crate::ecrecover::keccak(input.as_slice());
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
