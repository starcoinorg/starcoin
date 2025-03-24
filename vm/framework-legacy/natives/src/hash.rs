// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
};
use ripemd160::digest::Output;
use ripemd160::{Digest, Ripemd160};
use smallvec::{smallvec, SmallVec};
use starcoin_gas_schedule::gas_params::natives::starcoin_framework::{
    HASH_KECCAK256_BASE, HASH_KECCAK256_PER_BYTE,
};
use starcoin_native_interface::{
    safely_pop_arg, RawSafeNative, SafeNativeBuilder, SafeNativeContext, SafeNativeResult,
};
use std::collections::VecDeque;
use tiny_keccak::Keccak;
/***************************************************************************************************
 * native fun native_keccak_256
 *
 *   gas cost: base_cost + per_byte * data_length
 *
 **************************************************************************************************/

pub fn native_keccak_256(
    context: &mut SafeNativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let bytes = safely_pop_arg!(args, Vec<u8>);

    let cost = HASH_KECCAK256_BASE + HASH_KECCAK256_PER_BYTE * NumBytes::new(bytes.len() as u64);
    context.charge(cost)?;

    let mut hasher = Keccak::v256();
    hasher.update(&bytes);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);

    Ok(smallvec![Value::vector_u8(output)])
}

/***************************************************************************************************
 * native fun native_ripemd160
 *
 *   gas cost: base_cost + per_byte * data_length
 *
 **************************************************************************************************/

pub fn native_ripemd160(
    context: &mut SafeNativeContext,
    ty_args: Vec<Type>,
    mut arguments: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(arguments.len() == 1);

    let input_arg = safely_pop_arg!(arguments, Vec<u8>);

    //let cost = gas_params.base + gas_params.per_byte * NumBytes::new(input_arg.len() as u64);
    let cost = HASH_KECCAK256_PER_BYTE * NumBytes::new(input_arg.len() as u64);
    context.charge(cost)?;

    let result = ripemd160(input_arg.as_slice());
    Ok(smallvec![Value::vector_u8(result)])
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
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct GasParameters {
//     pub keccak256: Keccak256HashGasParameters,
//     pub ripemd160: Ripemd160HashGasParameters,
// }

pub fn make_all(
    builder: &SafeNativeBuilder,
) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
    let natives = [
        ("keccak_256", native_keccak_256 as RawSafeNative),
        ("ripemd160", native_ripemd160 as RawSafeNative),
    ];

    builder.make_named_natives(natives)
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
