// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{account_address::AccountAddress, language_storage::CORE_CODE_ADDRESS};
use move_vm_runtime::native_functions::{make_table_from_iter, NativeFunctionTable};
use starcoin_native_interface::SafeNativeBuilder;

pub mod account;
pub mod hash;
pub mod signature;
pub mod token;
pub mod u256;
// for support evm compat and cross chain.
pub mod ecrecover;
pub mod from_bcs;
pub mod secp256k1;

pub mod bcs;
pub mod debug;
mod event;

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

    add_natives_from_module!("Hash", hash::make_all(builder));
    // add_natives_from_module!("Hash", move_stdlib::natives::hash::make_all(builder));
    add_natives_from_module!("FromBCS", from_bcs::make_all(builder));
    add_natives_from_module!("BCS", bcs::make_all(builder));
    add_natives_from_module!("Signature", signature::make_all(builder));
    add_natives_from_module!("Account", account::make_all(builder));
    add_natives_from_module!("Token", token::make_all(builder));
    add_natives_from_module!("U256", u256::make_all(builder));
    // add_natives_from_module!("Event", event::make_all(builder));
    // add_natives_from_module!("Vector", vector::make_all(builder));
    add_natives_from_module!("Debug", debug::make_all(builder, CORE_CODE_ADDRESS));

    #[cfg(feature = "testing")]
    add_natives_from_module!("unit_test", unit_test::make_all(builder));

    make_table_from_iter(framework_addr, natives);
}
