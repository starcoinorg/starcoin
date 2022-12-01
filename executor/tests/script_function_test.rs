// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_config::ChainNetwork;
use starcoin_types::account::Account;
use starcoin_types::transaction::Transaction;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::transaction::{
    Package, ScriptFunction, TransactionPayload, TransactionStatus,
};
use starcoin_vm_types::vm_status::{KeptVMStatus, VMStatus};
use starcoin_vm_types::vm_status::StatusCode::INTERNAL_TYPE_ERROR;
use statedb::ChainStateDB;
use test_helper::executor::{compile_modules_with_address, execute_and_apply, prepare_genesis};
use test_helper::txn::create_account_txn_sent_as_association;

fn prepare_module(chain_state: &ChainStateDB, net: &ChainNetwork) -> ModuleId {
    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 0, 50_000_000, 1, net,
    ));
    let output1 = execute_and_apply(chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());
    let module_source = r#"
        module {{sender}}::Test {
            struct R has key, store {
                i: u64,
            }

            fun fn_private() {
            }

            public fun fn_public() {
            }

            public(script) fun fn_script() {
            }

            public(script) fun fn_script_with_args(account: &signer, i: u64) {
                let r = Self::R { i };
                move_to(account, r);
            }
        }
        "#;
    let compiled_module = compile_modules_with_address(*account1.address(), module_source)
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

    //publish the module
    let output = execute_and_apply(chain_state, txn);
    assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());

    //return the module id
    ModuleId::new(*account1.address(), Identifier::new("Test").unwrap())
}

#[stest::test]
fn test_invoke_script_function() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let module_id = prepare_module(&chain_state, &net);

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 1, 50_000_000, 1, &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let payload = TransactionPayload::ScriptFunction(ScriptFunction::new(
        module_id,
        Identifier::new("fn_script").unwrap(),
        vec![],
        vec![],
    ));
    let txn = Transaction::UserTransaction(account1.create_signed_txn_impl(
        *account1.address(),
        payload,
        0,
        100_000,
        1,
        1,
        net.chain_id(),
    ));

    let output = execute_and_apply(&chain_state, txn);
    assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());
    Ok(())
}

#[stest::test]
fn test_invoke_public_function() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let module_id = prepare_module(&chain_state, &net);

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 1, 50_000_000, 1, &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let payload = TransactionPayload::ScriptFunction(ScriptFunction::new(
        module_id,
        Identifier::new("fn_public").unwrap(),
        vec![],
        vec![],
    ));
    let txn = Transaction::UserTransaction(account1.create_signed_txn_impl(
        *account1.address(),
        payload,
        0,
        100_000,
        1,
        1,
        net.chain_id(),
    ));

    let output = execute_and_apply(&chain_state, txn);
    assert_eq!(
        KeptVMStatus::MiscellaneousError,
        output.status().status().unwrap()
    );
    Ok(())
}

#[stest::test]
fn test_invoke_private_function() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let module_id = prepare_module(&chain_state, &net);

    let account1 = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &account1, 1, 50_000_000, 1, &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());

    let payload = TransactionPayload::ScriptFunction(ScriptFunction::new(
        module_id,
        Identifier::new("fn_private").unwrap(),
        vec![],
        vec![],
    ));
    let txn = Transaction::UserTransaction(account1.create_signed_txn_impl(
        *account1.address(),
        payload,
        0,
        100_000,
        1,
        1,
        net.chain_id(),
    ));

    let output = execute_and_apply(&chain_state, txn);
    assert_eq!(
        KeptVMStatus::MiscellaneousError,
        output.status().status().unwrap()
    );
    Ok(())
}

#[stest::test]
fn test_nft_package() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let alice = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &alice, 0, 50_000_000, 1, &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());
    let module_source = r#"
        module {{sender}}::IdentifierNFTTest {
            use StarcoinFramework::NFT::{MintCapability, BurnCapability, UpdateCapability};
            use StarcoinFramework::NFT;

            struct Body has store{}
            struct Meta has copy, store, drop{}

            struct ShardCap has store, key{
                mint_cap:MintCapability<Meta>,
                burn_cap:BurnCapability<Meta>,
                update_cap:UpdateCapability<Meta>
            }
            public(script) fun init(sender: signer){
                let meta_data = NFT::empty_meta();
                NFT::register_v2<Meta>(&sender, meta_data);
                let mint_cap = NFT::remove_mint_capability<Meta>(&sender);
                let update_cap  = NFT::remove_update_capability<Meta>(&sender);
                let burn_cap = NFT::remove_burn_capability<Meta>(&sender);
                move_to(&sender,ShardCap{
                    mint_cap,
                    update_cap,
                    burn_cap
                });
            }
        }
        "#;
    let compiled_module = compile_modules_with_address(*alice.address(), module_source)
        .pop()
        .unwrap();
    let init_script = Some(ScriptFunction::new(
        ModuleId::new(
            *alice.address(),
            Identifier::new("IdentifierNFTTest").unwrap(),
        ),
        Identifier::new("init").unwrap(),
        vec![],
        vec![],
    ));
    let txn = Transaction::UserTransaction(alice.create_signed_txn_impl(
        *alice.address(),
        TransactionPayload::Package(Package::new(vec![compiled_module], init_script).unwrap()),
        0,
        10_000_000,
        1,
        1,
        net.chain_id(),
    ));

    let output = execute_and_apply(&chain_state, txn);
    // master code should run this
    // assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());
    assert_eq!(
        TransactionStatus::Discard(INTERNAL_TYPE_ERROR),
        output.status().clone()
    );
    Ok(())
}


#[stest::test]
fn test_signer_cap() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let alice = Account::new();
    let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
        &alice, 0, 50_000_000, 1, &net,
    ));
    let output1 = execute_and_apply(&chain_state, txn1);
    assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());
    let module_source = r#"
        module {{sender}}::TestSignerCap {
            use StarcoinFramework::Account;
            use StarcoinFramework::Signer;

            struct CapHolder has key{
                cap: Account::SignerCapability,
            }

            public(script) fun init(sender: signer){
                let (_addr,cap) = Account::create_delegate_account(&sender);
                move_to(&sender, CapHolder{cap});
            }

            public(script) fun test(sender: signer) acquires CapHolder{
                let addr = Signer::address_of(&sender);
                let cap = borrow_global<CapHolder>(addr);
                let _da = Account::create_signer_with_cap(&cap.cap);
            }
        }
        "#;
    let compiled_module = compile_modules_with_address(*alice.address(), module_source)
        .pop()
        .unwrap();
    let init_script = Some(ScriptFunction::new(
        ModuleId::new(
            *alice.address(),
            Identifier::new("TestSignerCap").unwrap(),
        ),
        Identifier::new("init").unwrap(),
        vec![],
        vec![],
    ));
    let txn = Transaction::UserTransaction(alice.create_signed_txn_impl(
        *alice.address(),
        TransactionPayload::Package(Package::new(vec![compiled_module], init_script).unwrap()),
        0,
        10_000_000,
        1,
        1,
        net.chain_id(),
    ));

    let output = execute_and_apply(&chain_state, txn);
    assert_eq!(
        TransactionStatus::Keep(KeptVMStatus::Executed),
        output.status().clone()
    );

    let test_script = ScriptFunction::new(
        ModuleId::new(
            *alice.address(),
            Identifier::new("TestSignerCap").unwrap(),
        ),
        Identifier::new("test").unwrap(),
        vec![],
        vec![],
    );
    let txn = Transaction::UserTransaction(alice.create_signed_txn_impl(
        *alice.address(),
        TransactionPayload::ScriptFunction(test_script),
        1,
        10_000_000,
        1,
        1,
        net.chain_id(),
    ));

    let output = execute_and_apply(&chain_state, txn);
    // master code should run this
    // assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());
    assert_eq!(
        TransactionStatus::Keep(KeptVMStatus::Executed),
        output.status().clone()
    );
    Ok(())
}

//
// #[stest::test]
// fn test_borrow_field() -> Result<()> {
//     let (chain_state, net) = prepare_genesis();
//     let alice = Account::new();
//     let txn1 = Transaction::UserTransaction(create_account_txn_sent_as_association(
//         &alice, 0, 50_000_000, 1, &net,
//     ));
//     let output1 = execute_and_apply(&chain_state, txn1);
//     assert_eq!(KeptVMStatus::Executed, output1.status().status().unwrap());
//     let module_source = r#"
//         module {{sender}}::test {
//             use StarcoinFramework::Signer;
//             struct TestObjWrapper has key{
//                 obj: TestObj,
//             }
//             struct TestObj has store{
//                 addr: address,
//             }
//
//             fun use_field(_addr: address){}
//             fun use_object(obj: &TestObj){
//                 use_field(obj.addr);
//             }
//
//             fun borrow_field(addr: address) acquires TestObjWrapper{
//                 let obj_wrapper = borrow_global<TestObjWrapper>(addr);
//                 use_object(&obj_wrapper.obj)
//             }
//
//
//             public fun test_borrow_field(sender: &signer) acquires TestObjWrapper {
//                 let addr = Signer::address_of(sender);
//                 borrow_field(addr);
//             }
//
//             public entry fun init(sender: signer) {
//                 let addr = Signer::address_of(&sender);
//                 let obj = TestObj{addr};
//                 move_to(&sender, TestObjWrapper{obj});
//             }
//
//             public entry fun test(sender: signer) acquires TestObjWrapper {
//                 test_borrow_field(&sender);
//             }
//         }
//         "#;
//     let compiled_module = compile_modules_with_address(*alice.address(), module_source)
//         .pop()
//         .unwrap();
//     let init_script = Some(ScriptFunction::new(
//         ModuleId::new(
//             *alice.address(),
//             Identifier::new("test").unwrap(),
//         ),
//         Identifier::new("init").unwrap(),
//         vec![],
//         vec![],
//     ));
//     let txn = Transaction::UserTransaction(alice.create_signed_txn_impl(
//         *alice.address(),
//         TransactionPayload::Package(Package::new(vec![compiled_module], init_script).unwrap()),
//         0,
//         10_000_000,
//         1,
//         1,
//         net.chain_id(),
//     ));
//
//     let output = execute_and_apply(&chain_state, txn);
//     assert_eq!(
//         TransactionStatus::Keep(KeptVMStatus::Executed),
//         output.status().clone()
//     );
//
//
//     let test_script = ScriptFunction::new(
//         ModuleId::new(
//             *alice.address(),
//             Identifier::new("test").unwrap(),
//         ),
//         Identifier::new("test").unwrap(),
//         vec![],
//         vec![],
//     );
//     let txn = Transaction::UserTransaction(alice.create_signed_txn_impl(
//         *alice.address(),
//         TransactionPayload::ScriptFunction(test_script),
//         1,
//         10_000_000,
//         1,
//         1,
//         net.chain_id(),
//     ));
//
//     let output = execute_and_apply(&chain_state, txn);
//     // master code should run this
//     // assert_eq!(KeptVMStatus::Executed, output.status().status().unwrap());
//     assert_eq!(
//         TransactionStatus::Discard(INTERNAL_TYPE_ERROR),
//         output.status().clone()
//     );
//
//     Ok(())
// }
