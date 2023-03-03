use move_core_types::account_address::AccountAddress;
// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use move_vm_runtime::native_functions;
use move_vm_runtime::native_functions::{
    make_table_from_iter, NativeFunction, NativeFunctionTable,
};
use starcoin_gas::NativeGasParameters;

/// The function returns all native functions supported by Starcoin.
/// NOTICE:
/// - mostly re-use natives defined in move-stdlib.
/// - be careful with the native cost table index used in the implementation
pub fn starcoin_natives(gas_params: NativeGasParameters) -> NativeFunctionTable {
    let mut natives = vec![];

    macro_rules! add_natives_from_module {
        ($module_name: expr, $natives: expr) => {
            natives.extend(
                $natives.map(|(func_name, func)| ($module_name.to_string(), func_name, func)),
            );
        };
    }

    add_natives_from_module!(
        "Hash",
        move_stdlib::natives::hash::make_all(gas_params.move_stdlib.hash)
    );
    add_natives_from_module!(
        "Hash",
        starcoin_natives::hash::make_all(gas_params.starcoin_natives.hash)
    );
    add_natives_from_module!(
        "BCS",
        move_stdlib::natives::bcs::make_all(gas_params.move_stdlib.bcs)
    );
    add_natives_from_module!(
        "Signature",
        starcoin_natives::signature::make_all(gas_params.starcoin_natives.signature)
    );
    add_natives_from_module!(
        "Vector",
        move_stdlib::natives::vector::make_all(gas_params.move_stdlib.vector)
    );
    add_natives_from_module!(
        "Event",
        move_stdlib::natives::event::make_all(gas_params.nursery.clone().event)
    );
    add_natives_from_module!(
        "Account",
        starcoin_natives::account::make_all(gas_params.starcoin_natives.account)
    );
    add_natives_from_module!(
        "Signer",
        move_stdlib::natives::signer::make_all(gas_params.move_stdlib.signer)
    );
    add_natives_from_module!(
        "Token",
        starcoin_natives::token::make_all(gas_params.starcoin_natives.token)
    );
    add_natives_from_module!(
        "U256",
        starcoin_natives::u256::make_all(gas_params.starcoin_natives.u256)
    );
    #[cfg(feature = "testing")]
    add_natives_from_module!(
        "unit_test",
        move_stdlib::natives::unit_test::make_all(gas_params.move_stdlib.unit_test)
    );
    add_natives_from_module!(
        "String",
        move_stdlib::natives::string::make_all(gas_params.move_stdlib.string)
    );
    add_natives_from_module!(
        "Debug",
        move_stdlib::natives::debug::make_all(gas_params.nursery.debug, CORE_CODE_ADDRESS)
    );
    let natives = make_table_from_iter(CORE_CODE_ADDRESS, natives);
    natives
        .into_iter()
        .chain(table_natives(CORE_CODE_ADDRESS, gas_params.table))
        .collect()
}

fn table_natives(
    table_addr: AccountAddress,
    gas_params: move_table_extension::GasParameters,
) -> NativeFunctionTable {
    let natives: [(&str, &str, NativeFunction); 8] = [
        (
            "Table",
            "new_table_handle",
            move_table_extension::make_native_new_table_handle(gas_params.new_table_handle),
        ),
        (
            "Table",
            "add_box",
            move_table_extension::make_native_add_box(
                gas_params.common.clone(),
                gas_params.add_box,
            ),
        ),
        (
            "Table",
            "borrow_box",
            move_table_extension::make_native_borrow_box(
                gas_params.common.clone(),
                gas_params.borrow_box.clone(),
            ),
        ),
        (
            "Table",
            "borrow_box_mut",
            move_table_extension::make_native_borrow_box(
                gas_params.common.clone(),
                gas_params.borrow_box,
            ),
        ),
        (
            "Table",
            "remove_box",
            move_table_extension::make_native_remove_box(
                gas_params.common.clone(),
                gas_params.remove_box,
            ),
        ),
        (
            "Table",
            "contains_box",
            move_table_extension::make_native_contains_box(
                gas_params.common,
                gas_params.contains_box,
            ),
        ),
        (
            "Table",
            "destroy_empty_box",
            move_table_extension::make_native_destroy_empty_box(gas_params.destroy_empty_box),
        ),
        (
            "Table",
            "drop_unchecked_box",
            move_table_extension::make_native_drop_unchecked_box(gas_params.drop_unchecked_box),
        ),
    ];

    native_functions::make_table_from_iter(table_addr, natives)
}
