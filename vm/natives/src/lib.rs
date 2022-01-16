// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod account;
pub mod bcs;
pub mod debug;
pub mod hash;
pub mod signature;
pub mod token;
pub mod u256;
// for support evm compat and cross chain.
pub mod ecrecover;

pub mod vector;

use move_core_types::identifier::Identifier;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use move_vm_runtime::native_functions::{NativeFunction, NativeFunctionTable};

/// The function returns all native functions supported by Starcoin.
/// NOTICE:
/// - mostly re-use natives defined in move-stdlib.
/// - be careful with the native cost table index used in the implementation
pub fn starcoin_natives() -> NativeFunctionTable {
    const NATIVES: &[(&str, &str, NativeFunction)] = &[
        (
            "Hash",
            "sha2_256",
            move_stdlib::natives::hash::native_sha2_256,
        ),
        (
            "Hash",
            "sha3_256",
            move_stdlib::natives::hash::native_sha3_256,
        ),
        ("Hash", "keccak_256", hash::native_keccak_256),
        ("Hash", "ripemd160", hash::native_ripemd160),
        (
            "BCS",
            "to_bytes",
            move_stdlib::natives::bcs::native_to_bytes,
        ),
        ("BCS", "to_address", bcs::native_to_address),
        (
            "Signature",
            "ed25519_validate_pubkey",
            signature::native_ed25519_publickey_validation,
        ),
        (
            "Signature",
            "ed25519_verify",
            signature::native_ed25519_signature_verification,
        ),
        ("Signature", "native_ecrecover", ecrecover::native_ecrecover),
        (
            "Vector",
            "length",
            move_stdlib::natives::vector::native_length,
        ),
        (
            "Vector",
            "empty",
            move_stdlib::natives::vector::native_empty,
        ),
        (
            "Vector",
            "borrow",
            move_stdlib::natives::vector::native_borrow,
        ),
        (
            "Vector",
            "borrow_mut",
            move_stdlib::natives::vector::native_borrow,
        ),
        (
            "Vector",
            "push_back",
            move_stdlib::natives::vector::native_push_back,
        ),
        (
            "Vector",
            "pop_back",
            move_stdlib::natives::vector::native_pop,
        ),
        (
            "Vector",
            "destroy_empty",
            move_stdlib::natives::vector::native_destroy_empty,
        ),
        ("Vector", "swap", move_stdlib::natives::vector::native_swap),
        ("Vector", "native_append", vector::native_append),
        ("Vector", "native_remove", vector::native_remove),
        ("Vector", "native_reverse", vector::native_reverse),
        (
            "Event",
            "write_to_event_store",
            move_stdlib::natives::event::write_to_event_store,
        ),
        ("Account", "create_signer", account::native_create_signer),
        ("Account", "destroy_signer", account::native_destroy_signer),
        (
            "Signer",
            "borrow_address",
            move_stdlib::natives::signer::native_borrow_address,
        ),
        ("Token", "name_of", token::native_token_name_of),
        ("Debug", "print", debug::native_print),
        (
            "Debug",
            "print_stack_trace",
            debug::native_print_stack_trace,
        ),
        #[cfg(feature = "testing")]
        (
            "UnitTest",
            "create_signers_for_testing",
            move_stdlib::natives::unit_test::native_create_signers_for_testing,
        ),
        ("U256", "from_bytes", u256::native_u256_from_bytes),
        ("U256", "native_add", u256::native_u256_add),
        ("U256", "native_sub", u256::native_u256_sub),
        ("U256", "native_mul", u256::native_u256_mul),
        ("U256", "native_div", u256::native_u256_div),
        ("U256", "native_rem", u256::native_u256_rem),
        ("U256", "native_pow", u256::native_u256_pow),
    ];
    NATIVES
        .iter()
        .cloned()
        .map(|(module_name, func_name, func)| {
            (
                CORE_CODE_ADDRESS,
                Identifier::new(module_name).unwrap(),
                Identifier::new(func_name).unwrap(),
                func,
            )
        })
        .collect()
}
