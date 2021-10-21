// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account::{create_account_txn_sent_as_association, Account};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use starcoin_dev::playground::call_contract;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{Transaction, TransactionArgument};
use starcoin_vm_types::errors::Location;
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::{StructTag, TypeTag};
use starcoin_vm_types::transaction::{Package, TransactionPayload};
use starcoin_vm_types::vm_status::KeptVMStatus;
use starcoin_vm_types::vm_status::{StatusCode, VMStatus};
use test_helper::executor::{compile_modules_with_address, execute_and_apply, prepare_genesis};

#[stest::test]
fn test_readonly_function_call() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000, 1, &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let module_source = r#"
        module {{sender}}::A {

        struct S has copy,drop,store {
            f1: u64,
        }

        struct R has key,store {
            f1: u64,
        }

        public fun new(): S { Self::S { f1: 20 } }

        public fun get_s(): S {
            let s = Self::new();
            s
        }
        
        public fun get_tuple(): (u64, address) {
            (0, @0x1)
        }

        public fun set_s(account: &signer): u64 {
            let r = Self::R { f1: 20 };
            move_to(account, r);
            1u64
        }
        }
        "#;

    // compile with account 1's address
    let module = compile_modules_with_address(*account1.address(), module_source)
        .pop()
        .unwrap();

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

    let result = starcoin_dev::playground::call_contract(
        &chain_state,
        compiled_module.self_id(),
        "get_s",
        vec![],
        vec![],
        None,
    )?;

    let ty = TypeTag::Struct(StructTag {
        address: *account1.address(),
        module: Identifier::new("A").unwrap(),
        name: Identifier::new("S").unwrap(),
        type_params: vec![],
    });
    assert_eq!(result[0].0, ty);
    #[derive(Serialize, Deserialize)]
    struct S {
        f1: u64,
    }
    let s: S = bcs_ext::from_bytes(result[0].1.as_slice())?;
    assert_eq!(s.f1, 20);

    // test on return multi values.
    {
        let result = call_contract(
            &chain_state,
            compiled_module.self_id(),
            "get_tuple",
            vec![],
            vec![],
            None,
        )?;
        assert_eq!(result.len(), 2);

        assert_eq!(result[0].0, TypeTag::U64);
        assert_eq!(result[1].0, TypeTag::Address);
        assert_eq!(bcs_ext::from_bytes::<u64>(result[0].1.as_slice())?, 0u64);
        assert_eq!(
            bcs_ext::from_bytes::<AccountAddress>(result[1].1.as_slice())?,
            AccountAddress::from_hex_literal("0x1").unwrap()
        );
    }
    let _result = call_contract(
        &chain_state,
        compiled_module.self_id(),
        "set_s",
        vec![],
        vec![TransactionArgument::Address(*account1.address())],
        None,
    )
    .map_err(|err| {
        assert_eq!(
            err.downcast::<VMStatus>().unwrap(),
            VMStatus::Error(StatusCode::REJECTED_WRITE_SET)
        );
    });

    Ok(())
}
