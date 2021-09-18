// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_executor::account::create_account_txn_sent_as_association;
use starcoin_types::transaction::Transaction;
use starcoin_vm_types::transaction::{Package, TransactionPayload};
use starcoin_vm_types::vm_status::KeptVMStatus;
use test_helper::executor::{compile_modules_with_address, execute_and_apply, prepare_genesis};
use test_helper::Account;

#[stest::test]
pub fn test_publish_module_bundle() -> Result<()> {
    let (chain_state, net) = prepare_genesis();

    let account1 = Account::new();

    {
        let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
            &account1, 0, 50_000_000, 1, &net,
        ));
        let output1 = execute_and_apply(&chain_state, txn1);
        assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());
    }

    let module_source = r#"
        address {{sender}} {
        
        module A {
            friend {{sender}}::B;
            friend {{sender}}::C;
            
            struct S has copy,drop,store {
                f1: u64,
            }
            
            public(friend) fun new(): S {
                S {f1: 0}
            }
            public fun get_f(s: &S): u64 {
                s.f1
            }
        }
        
        module B {
            use {{sender}}::A;
            struct BS has drop {
                f1: A::S
            }
            public fun new(): BS {
                BS{ f1: A::new() }
            }
            public fun get_f(bs: &BS): &A::S {
                &bs.f1
            } 
        }
        module C {
            use {{sender}}::A;
            use {{sender}}::B;
        
            public fun check(): bool {
                let s1 = A::new();
                let bs = B::new();
                let s2 = B::get_f(&bs);
                A::get_f(&s1) == A::get_f(s2)
            }    
        }        
        }
        "#;

    let modules = compile_modules_with_address(*account1.address(), module_source);

    let mut sequence_number = 0;
    {
        // change the order of modules
        let mut package = Package::new_with_module(modules[1].clone()).unwrap();
        package.add_module(modules[0].clone()).unwrap();
        package.add_module(modules[2].clone()).unwrap();
        let txn = Transaction::UserTransaction(account1.create_signed_txn_impl(
            *account1.address(),
            TransactionPayload::Package(package),
            sequence_number,
            100_000,
            1,
            1,
            net.chain_id(),
        ));
        //publish the module
        let output = execute_and_apply(&chain_state, txn);
        assert_eq!(
            Ok(KeptVMStatus::MiscellaneousError),
            output.status().status()
        );
        sequence_number += 1;
    }
    // the right path
    {
        let package = Package::new_with_modules(modules).unwrap();
        let txn = Transaction::UserTransaction(account1.create_signed_txn_impl(
            *account1.address(),
            TransactionPayload::Package(package),
            sequence_number,
            100_000,
            1,
            1,
            net.chain_id(),
        ));
        //publish the module
        let output = execute_and_apply(&chain_state, txn);
        assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());

        // sequence_number += 1;
    }

    Ok(())
}
