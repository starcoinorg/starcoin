// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account::{create_account_txn_sent_as_association, Account};
use anyhow::Result;
use starcoin_types::transaction::Transaction;
use starcoin_vm_types::transaction::{Package, TransactionPayload};
use starcoin_vm_types::vm_status::KeptVMStatus;
use test_helper::executor::{compile_modules_with_address, execute_and_apply, prepare_genesis};

macro_rules! module_compatibility_test {
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

            // compile with account 1's address
            let compiled_module = compile_modules_with_address(*account1.address(), &program1)
                .pop()
                .unwrap();

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

            let program2 = String::from($prog2);

            // compile with account 1's address
            let compiled_module = compile_modules_with_address(*account1.address(), &program2)
                .pop()
                .unwrap();

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

// Publishing a module named M under the same address twice is OK (a module is self-compatible)
module_compatibility_test!(
    duplicate_module,
    "
    module {{sender}}::M {
        struct T has key,store { f: u64 }
        public fun f() { return }
    }
    ",
    "
    module {{sender}}::M {
        struct T has key,store { f: u64 }
        public fun f() { return }
    }
    ",
    Executed
);

// Republishing a module named M under the same address with a superset of the structs is OK
module_compatibility_test!(
    layout_compatible_module,
    "
    module {{sender}}::M {
    }
    ",
    "
    module {{sender}}::M {
        struct T has key,store { f: u64 }
    }
    ",
    Executed
);

// Republishing a module named M under the same address with a superset of public functions is OK
module_compatibility_test!(
    linking_compatible_module,
    "
    module {{sender}}::M {
    }
    ",
    "
    module {{sender}}::M {
        public fun f() { return }
    }
    ",
    Executed
);

// Republishing a module named M under the same address that breaks data layout should be rejected
module_compatibility_test!(
    layout_incompatible_module_with_new_field,
    "
    module {{sender}}::M {
        struct T has key,store { f: u64 }
    }
    ",
    "
    module {{sender}}::M {
        struct T has key,store { f: u64, g: bool }
    }
    ",
    MiscellaneousError
);

module_compatibility_test!(
    layout_incompatible_module_with_changed_field,
    "
    module {{sender}}::M {
        struct T has key,store { f: u64 }
    }
    ",
    "
    module {{sender}}::M {
        struct T has key,store { f: bool }
    }
    ",
    MiscellaneousError
);

module_compatibility_test!(
    layout_incompatible_module_with_removed_field,
    "
    module {{sender}}::M {
        struct T has key,store { f: u64 }
    }
    ",
    "
    module {{sender}}::M {
        struct T has key,store {}
    }
    ",
    MiscellaneousError
);

module_compatibility_test!(
    layout_incompatible_module_with_removed_struct,
    "
    module {{sender}}::M {
        struct T has key,store { f: u64 }
    }
    ",
    "
    module {{sender}}::M {
    }
    ",
    MiscellaneousError
);

// Republishing a module named M under the same address that breaks linking should be rejected
module_compatibility_test!(
    linking_incompatible_module_with_added_param,
    "
    module {{sender}}::M {
        public fun f() { return }
    }
    ",
    "
    module {{sender}}::M {
        public fun f(_a: u64) { return }
    }
    ",
    MiscellaneousError
);

module_compatibility_test!(
    linking_incompatible_module_with_changed_param,
    "
    module {{sender}}::M {
        public fun f(_a: u64) { return }
    }
    ",
    "
    module {{sender}}::M {
        public fun f(_a: bool) { return }
    }
    ",
    MiscellaneousError
);

module_compatibility_test!(
    linking_incompatible_module_with_removed_pub_fn,
    "
    module {{sender}}::M {
        public fun f() { return }
    }
    ",
    "
    module {{sender}}::M {
    }
    ",
    MiscellaneousError
);
