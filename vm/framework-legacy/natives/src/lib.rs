// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::account_address::AccountAddress;
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

// mod helpers;
pub mod util;

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

    make_table_from_iter(framework_addr, natives);

    // add_natives_from_module!("Hash", move_stdlib::natives::hash::make_all(gas_params.move_stdlib.hash));
    // add_natives_from_module!("Hash",hash::make_all(gas_params.starcoin_natives.hash));
    // add_natives_from_module!(
    //     "BCS",
    //     bcs::make_all(gas_params.move_stdlib.bcs)
    // );
    // add_natives_from_module!(
    //     "FromBCS",
    //     starcoin_natives::from_bcs::make_all(gas_params.starcoin_natives.from_bcs)
    // );
    // add_natives_from_module!(
    //     "Signature",
    //     starcoin_natives::signature::make_all(gas_params.starcoin_natives.signature)
    // );
    // add_natives_from_module!(
    //     "Vector",
    //     move_stdlib::natives::vector::make_all(gas_params.move_stdlib.vector)
    // );
    // add_natives_from_module!(
    //     "Event",
    //     move_stdlib::natives::event::make_all(gas_params.nursery.clone().event)
    // );
    // add_natives_from_module!(
    //     "Account",
    //     starcoin_natives::account::make_all(gas_params.starcoin_natives.account)
    // );
    // add_natives_from_module!(
    //     "Signer",
    //     move_stdlib::natives::signer::make_all(gas_params.move_stdlib.signer)
    // );
    // add_natives_from_module!(
    //     "Token",
    //     starcoin_natives::token::make_all(gas_params.starcoin_natives.token)
    // );
    // add_natives_from_module!(
    //     "U256",
    //     starcoin_natives::u256::make_all(gas_params.starcoin_natives.u256)
    // );
    // #[cfg(feature = "testing")]
    // add_natives_from_module!(
    //     "unit_test",
    //     move_stdlib::natives::unit_test::make_all(gas_params.move_stdlib.unit_test)
    // );
    // add_natives_from_module!(
    //     "String",
    //     move_stdlib::natives::string::make_all(gas_params.move_stdlib.string)
    // );
    // add_natives_from_module!(
    //     "Debug",
    //     move_stdlib::natives::debug::make_all(gas_params.nursery.debug, CORE_CODE_ADDRESS)
    // );
    // add_natives_from_module!(
    //     "Secp256k1",
    //     starcoin_natives::secp256k1::make_all(gas_params.starcoin_natives.secp256k1)
    // );
}
