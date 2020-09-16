// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor_test::{compile_module_with_address, execute_and_apply, prepare_genesis};
use anyhow::Result;
use starcoin_functional_tests::account::{create_account_txn_sent_as_association, Account};
use starcoin_types::transaction::Transaction;
use starcoin_vm_types::errors::Location;
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::transaction::{Package, TransactionPayload};
use starcoin_vm_types::values::{Struct, Value};
use starcoin_vm_types::vm_status::KeptVMStatus;
use starcoin_vm_types::value::{MoveStructLayout, MoveTypeLayout};
use starcoin_vm_types::vm_status::{StatusCode, VMStatus};
#[stest::test]
fn test_readonly_function_call() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000, 1, net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let module_source = r#"
        module A {

        struct S {
            f1: u64,
        }

        resource struct R {
            f1: u64,
        }

        public fun new(): S { Self::S { f1: 20 } }

        public fun get_s(): S {
            let s = Self::new();
            s
        }

        public fun set_s(account: &signer): u64 {
            let r = Self::R { f1: 20 };
            move_to(account, r);
            1u64
        }
        }
        "#;

    // compile with account 1's address
    let module = compile_module_with_address(*account1.address(), module_source);

    let txn = Transaction::UserTransaction(account1.create_signed_txn_impl(
        *account1.address(),
        TransactionPayload::Package(Package::new_with_module(module.clone()).unwrap()),
        0,
        100_000,
        1,
        1,
        net.chain_id(),
    ));

    //publish the module
    let output = execute_and_apply(&chain_state, txn);
    assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());

    let compiled_module = CompiledModule::deserialize(module.code())
        .map_err(|e| e.finish(Location::Undefined).into_vm_status())?;

    let result = crate::execute_readonly_function(
        &chain_state,
        &compiled_module.self_id(),
        &Identifier::new("get_s").unwrap(),
        vec![],
        vec![],
        *account1.address(),
    )?;

    let value = Value::struct_(Struct::pack(vec![Value::u64(20)], false));
    assert!(
        result[0].0 == MoveTypeLayout::Struct(MoveStructLayout::new(vec![MoveTypeLayout::U64]))
    );
    assert!(result[0].1
        .equals(&value)
        .map_err(|e| e.finish(Location::Undefined).into_vm_status())?);

    let value = Value::transaction_argument_signer_reference(*account1.address());
    let result = crate::execute_readonly_function(
        &chain_state,
        &compiled_module.self_id(),
        &Identifier::new("set_s").unwrap(),
        vec![],
        vec![value],
        *account1.address(),
    );
    //assert_eq!(result, Err(VMStatus::Error(StatusCode::REJECTED_WRITE_SET)));
    Ok(())
}
