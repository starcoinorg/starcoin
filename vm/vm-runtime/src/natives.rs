// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use move_vm_runtime::native_functions::{
    make_table_from_iter, NativeFunction, NativeFunctionTable,
};
use starcoin_gas::NativeGasParameters;

/// The function returns all native functions supported by Starcoin.
/// NOTICE:
/// - mostly re-use natives defined in move-stdlib.
/// - be careful with the native cost table index used in the implementation
pub fn starcoin_natives(gas_params: NativeGasParameters) -> NativeFunctionTable {
    // XXX FIXME YSG
    let mut natives = vec![];

    macro_rules! add_natives_from_module {
        ($module_name: expr, $natives: expr) => {
            natives.extend(
                $natives.map(|(func_name, func)| ($module_name.to_string(), func_name, func)),
            );
        };
    }

    add_natives_from_module!(
        "hash",
        move_stdlib::natives::hash::make_all(gas_params.move_stdlib.hash)
    );
    add_natives_from_module!(
        "starcoin_hash",
        starcoin_natives::hash::make_all(gas_params.starcoin_natives.hash)
    );
    add_natives_from_module!(
        "bcs",
        move_stdlib::natives::bcs::make_all(gas_params.move_stdlib.bcs)
    );
    add_natives_from_module!(
        "signature",
        starcoin_natives::signature::make_all(gas_params.starcoin_natives.signature)
    );
    add_natives_from_module!(
        "vector",
        move_stdlib::natives::vector::make_all(gas_params.move_stdlib.vector)
    );
    add_natives_from_module!(
        "account",
        starcoin_natives::account::make_all(gas_params.starcoin_natives.account)
    );
    add_natives_from_module!(
        "signer",
        move_stdlib::natives::signer::make_all(gas_params.move_stdlib.signer)
    );
    add_natives_from_module!(
        "token",
        starcoin_natives::token::make_all(gas_params.starcoin_natives.token)
    );
    // XXX FIXME YSG
    //  add_natives_from_module!("event", move_stdlib::natives::event::make_all(gas_params.move_stdlib.event));
    //  add_natives_from_module!("debug", move_stdlib::natives::debug::make_all(gas_params.move_stdlib.debug));
    add_natives_from_module!(
        "u256",
        starcoin_natives::u256::make_all(gas_params.starcoin_natives.u256)
    );
    #[cfg(feature = "testing")]
    add_natives_from_module!(
        "unit_test",
        move_stdlib::natives::unit_test::make_all(gas_params.move_stdlib.unit_test)
    );
    add_natives_from_module!(
        "string",
        move_stdlib::natives::string::make_all(gas_params.move_stdlib.string)
    );
    let natives = make_table_from_iter(CORE_CODE_ADDRESS, natives);
    natives
        .into_iter()
        .chain(move_table_extension::table_natives(
            CORE_CODE_ADDRESS,
            gas_params.table,
        ))
        .collect()
}
