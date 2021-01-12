// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account::{create_account_txn_sent_as_association, Account};
use anyhow::Result;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::Transaction;
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_vm_types::errors::Location;
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::{StructTag, TypeTag};
use starcoin_vm_types::transaction::{Package, TransactionPayload};
use starcoin_vm_types::values::{Struct, Value};
use starcoin_vm_types::vm_status::KeptVMStatus;
use starcoin_vm_types::vm_status::{StatusCode, VMStatus};
use test_helper::executor::{compile_module_with_address, execute_and_apply, prepare_genesis};

macro_rules! module_republish_test {
    ($name:ident, $prog1:literal, $prog2:literal, $result:ident) => {
        #[stest::test]
        fn $name() -> Result<()> {

    let (chain_state, net) = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000, 1, &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let program1 = String::from($prog1);

    println!(">>> {:?}", program1);

    // compile with account 1's address
    let compiled_module = compile_module_with_address(*account1.address(), &program1);

    let txn = Transaction::UserTransaction(account1.create_signed_txn_impl(
        *account1.address(),
        TransactionPayload::Package(Package::new_with_module(compiled_module).unwrap()),
        0,
        100_000,
        1,
        1,
        net.chain_id(),
    ));

    let output = execute_and_apply(&chain_state, txn);
    assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());

// ////////////////////////////
//         let txn = Transaction::BlockMetadata(BlockMetadata::new(
//             starcoin_crypto::HashValue::random(),
//             net.time_service().now_millis(),
//             *account1.address(),
//             Some(account1.auth_key()),
//             0,
//             i + 1,
//             net.chain_id(),
//             0,
//         ));
//         let output = execute_and_apply(&chain_state, txn);
//         assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());
// /////////////////////////////



    let program2 = String::from($prog2);

    // compile with account 1's address
    let compiled_module = compile_module_with_address(*account1.address(), &program2);

    let txn = Transaction::UserTransaction(account1.create_signed_txn_impl(
        *account1.address(),
        TransactionPayload::Package(Package::new_with_module(compiled_module).unwrap()),
        1,
        100_000,
        1,
        1,
        net.chain_id(),
    ));

    let output = execute_and_apply(&chain_state, txn);
    assert_eq!(KeptVMStatus::$result, output.status().status().unwrap());


    Ok(())
}
   };
}

// // Publishing a module named M under the same address twice is OK (a module is self-compatible)
// module_republish_test!(
//     duplicate_module,
//     "
//     module M {
//         resource struct T { f: u64 }
//         public fun f() { return }
//     }
//     ",
//     "
//     module M {
//         resource struct T { f: u64 }
//         public fun f() { return }
//     }
//     ",
//     Executed
// );
//
// // Republishing a module named M under the same address with a superset of the structs is OK
// module_republish_test!(
//     layout_compatible_module,
//     "
//     module M {
//     }
//     ",
//     "
//     module M {
//         resource struct T { f: u64 }
//     }
//     ",
//     Executed
// );
//
// // Republishing a module named M under the same address with a superset of public functions is OK
// module_republish_test!(
//     linking_compatible_module,
//     "
//     module M {
//     }
//     ",
//     "
//     module M {
//         public fun f() { return }
//     }
//     ",
//     Executed
// );

// // Republishing a module named M under the same address that breaks data layout should be rejected
// module_republish_test!(
//     layout_incompatible_module_with_new_field,
//     "
//     module M {
//         resource struct T { f: u64 }
//     }
//     ",
//     "
//     module M {
//         resource struct T { f: u64, g: bool }
//     }
//     ",
//     MiscellaneousError
// );
//
// module_republish_test!(
//     layout_incompatible_module_with_changed_field,
//     "
//     module M {
//         resource struct T { f: u64 }
//     }
//     ",
//     "
//     module M {
//         resource struct T { f: bool }
//     }
//     ",
//     MiscellaneousError
// );
//
// module_republish_test!(
//     layout_incompatible_module_with_removed_field,
//     "
//     module M {
//         resource struct T { f: u64 }
//     }
//     ",
//     "
//     module M {
//         resource struct T {}
//     }
//     ",
//     MiscellaneousError
// );
//
// module_republish_test!(
//     layout_incompatible_module_with_removed_struct,
//     "
//     module M {
//         resource struct T { f: u64 }
//     }
//     ",
//     "
//     module M {
//     }
//     ",
//     MiscellaneousError
// );
//
// // Republishing a module named M under the same address that breaks linking should be rejected
// module_republish_test!(
//     linking_incompatible_module_with_added_param,
//     "
//     module M {
//         public fun f() { return }
//     }
//     ",
//     "
//     module M {
//         public fun f(_a: u64) { return }
//     }
//     ",
//     MiscellaneousError
// );
//
module_republish_test!(
    linking_incompatible_module_with_changed_param,
    "
    module M {
        public fun f(_a: u64) { return }
    }
    ",
    "
    module M {
        public fun f(_a: bool) { return }
    }
    ",
    VERIFICATION_ERROR
);
//
// module_republish_test!(
//     linking_incompatible_module_with_removed_pub_fn,
//     "
//     module M {
//         public fun f() { return }
//     }
//     ",
//     "
//     module M {
//     }
//     ",
//     MiscellaneousError
// );

