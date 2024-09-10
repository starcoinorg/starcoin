// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::gas_algebra::NumBytes;
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{loaded_data::runtime_types::Type, values::Value};
use ripemd160::Digest;
use ripemd160::Ripemd160;
use smallvec::{smallvec, SmallVec};
use starcoin_gas_schedule::gas_params::natives::starcoin_framework::*;
use starcoin_native_interface::{
    safely_pop_arg, RawSafeNative, SafeNativeBuilder, SafeNativeContext, SafeNativeResult,
};
use std::collections::VecDeque;
use tiny_keccak::{Hasher, Keccak};

fn native_keccak256(
    context: &mut SafeNativeContext,
    mut _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(_ty_args.is_empty());
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

pub fn native_ripemd160(
    context: &mut SafeNativeContext,
    mut _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(args.len() == 1);

    let bytes = safely_pop_arg!(args, Vec<u8>);

    let cost = HASH_RIPEMD160_BASE + HASH_RIPEMD160_PER_BYTE * NumBytes::new(bytes.len() as u64);
    context.charge(cost)?;

    let mut hasher = Ripemd160::new();
    hasher.update(&bytes);
    let output = hasher.finalize().to_vec();

    Ok(smallvec![Value::vector_u8(output)])
}

pub fn make_all(
    builder: &SafeNativeBuilder,
) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
    let natives = [
        ("keccak_256", native_keccak256 as RawSafeNative),
        ("ripemd160", native_ripemd160),
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
