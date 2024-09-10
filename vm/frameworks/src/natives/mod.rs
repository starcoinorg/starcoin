// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

// ref aptos-move/framework/natives/src/lib.rs
// XXX FIXME YSG, refactor

use move_core_types::account_address::AccountAddress;
use move_vm_runtime::native_functions::{make_table_from_iter, NativeFunctionTable};
use starcoin_native_interface::SafeNativeBuilder;

mod account;
mod hash;
mod signature;
mod token;
// for support evm compat and cross chain.
mod code;
mod ecrecover;
mod from_bcs;
mod secp256k1;

// mod u256;

pub mod status {
    // Failure in parsing a struct type tag
    pub const NFE_EXPECTED_STRUCT_TYPE_TAG: u64 = 0x1;
    // Failure in address parsing (likely no correct length)
    pub const NFE_UNABLE_TO_PARSE_ADDRESS: u64 = 0x2;
}

pub fn all_natives(
    framework_addr: AccountAddress,
    builder: &SafeNativeBuilder,
) -> NativeFunctionTable {
    let mut natives = vec![];

    macro_rules! add_natives_from_module {
        ($module_name:expr, $natives:expr) => {
            natives.extend(
                $natives.map(|(func_name, func)| ($module_name.to_string(), func_name, func)),
            );
        };
    }
    add_natives_from_module!("account", account::make_all(builder));
    add_natives_from_module!("secp256k1", secp256k1::make_all(builder));
    add_natives_from_module!("hash", hash::make_all(builder));
    add_natives_from_module!("signature", signature::make_all(builder));
    add_natives_from_module!("token", token::make_all(builder));
    add_natives_from_module!("code", code::make_all(builder));
    add_natives_from_module!("from_bcs", from_bcs::make_all(builder));
    //   add_natives_from_module!("u256", u256::make_all(builder));
    make_table_from_iter(framework_addr, natives)
}
