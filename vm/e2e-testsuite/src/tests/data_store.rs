// Copyright (c) The Starcoin Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::CompiledModule;
use move_bytecode_verifier::verify_module;
use move_core_types::vm_status::KeptVMStatus;
use move_ir_compiler::Compiler;
use starcoin_language_e2e_tests::{
    account::AccountData, compile::compile_script_with_extra_deps, executor::FakeExecutor,
};
use starcoin_transaction_builder::{stdlib_compiled_modules, StdLibOptions};
use starcoin_vm_types::{
    genesis_config::StdlibVersion::Latest,
    transaction::{Module, SignedUserTransaction, Transaction, TransactionStatus},
};

#[test]
fn move_from_across_blocks() {
    let mut executor = FakeExecutor::from_genesis_file();
    //executor.set_golden_file(current_function_name!());
    let sender = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);

    // publish module with add and remove resource
    let (module, txn) = add_module_txn(&sender, 10);
    executor.execute_and_apply(txn);

    // remove resource fails given no resource were published
    let rem_txn = remove_resource_txn(&sender, 11, vec![module.clone()]);
    let output = executor.execute_transaction(rem_txn);
    assert!(matches!(
        output.status().status(),
        // StatusCode::MISSING_DATA
        Ok(KeptVMStatus::ExecutionFailure { .. })
    ));
    executor.apply_write_set(output.write_set());

    // publish resource
    let add_txn = add_resource_txn(&sender, 12, vec![module.clone()]);
    executor.execute_and_apply(add_txn);

    // borrow resource
    let borrow_txn = borrow_resource_txn(&sender, 13, vec![module.clone()]);
    executor.execute_and_apply(borrow_txn);

    // remove resource
    let rem_txn = remove_resource_txn(&sender, 14, vec![module.clone()]);
    executor.execute_and_apply(rem_txn);

    // remove resource fails given it was removed already
    let rem_txn = remove_resource_txn(&sender, 15, vec![module.clone()]);
    let output = executor.execute_transaction(rem_txn);
    assert!(matches!(
        output.status().status(),
        // StatusCode::MISSING_DATA
        Ok(KeptVMStatus::ExecutionFailure { .. })
    ));
    executor.apply_write_set(output.write_set());

    // borrow resource fail given it was removed
    let borrow_txn = borrow_resource_txn(&sender, 16, vec![module.clone()]);
    let output = executor.execute_transaction(borrow_txn);
    assert!(matches!(
        output.status().status(),
        // StatusCode::MISSING_DATA
        Ok(KeptVMStatus::ExecutionFailure { .. })
    ));
    executor.apply_write_set(output.write_set());

    // publish resource again
    let add_txn = add_resource_txn(&sender, 17, vec![module.clone()]);
    executor.execute_and_apply(add_txn);

    // create 2 remove resource transaction over the same resource in one block
    let txns = vec![
        Transaction::UserTransaction(remove_resource_txn(&sender, 18, vec![module.clone()])),
        Transaction::UserTransaction(remove_resource_txn(&sender, 19, vec![module])),
    ];
    let output = executor
        .execute_transaction_block(txns)
        .expect("Must execute transactions");
    assert_eq!(
        output[0].status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    assert!(matches!(
        output[1].status().status(),
        // StatusCode::MISSING_DATA
        Ok(KeptVMStatus::ExecutionFailure { .. })
    ));
    for out in output {
        executor.apply_write_set(out.write_set());
    }
}

#[test]
fn borrow_after_move() {
    let mut executor = FakeExecutor::from_genesis_file();
    // executor.set_golden_file(current_function_name!());
    let sender = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);

    // publish module with add and remove resource
    let (module, txn) = add_module_txn(&sender, 10);
    executor.execute_and_apply(txn);

    // remove resource fails given no resource were published
    let rem_txn = remove_resource_txn(&sender, 11, vec![module.clone()]);
    let output = executor.execute_transaction(rem_txn);
    assert!(matches!(
        output.status().status(),
        // StatusCode::MISSING_DATA
        Ok(KeptVMStatus::ExecutionFailure { .. })
    ));
    executor.apply_write_set(output.write_set());

    // publish resource
    let add_txn = add_resource_txn(&sender, 12, vec![module.clone()]);
    executor.execute_and_apply(add_txn);

    // borrow resource
    let borrow_txn = borrow_resource_txn(&sender, 13, vec![module.clone()]);
    executor.execute_and_apply(borrow_txn);

    // create a remove and a borrow resource transaction over the same resource in one block
    let txns = vec![
        Transaction::UserTransaction(remove_resource_txn(&sender, 14, vec![module.clone()])),
        Transaction::UserTransaction(borrow_resource_txn(&sender, 15, vec![module])),
    ];
    let output = executor
        .execute_transaction_block(txns)
        .expect("Must execute transactions");
    assert_eq!(
        output[0].status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    assert!(matches!(
        output[1].status().status(),
        // StatusCode::MISSING_DATA
        Ok(KeptVMStatus::ExecutionFailure { .. })
    ));
    for out in output {
        executor.apply_write_set(out.write_set());
    }
}

#[test]
fn change_after_move() {
    let mut executor = FakeExecutor::from_genesis_file();
    //executor.set_golden_file(current_function_name!());
    let sender = executor.create_raw_account_data(1_000_000, 10);
    executor.add_account_data(&sender);

    // publish module with add and remove resource
    let (module, txn) = add_module_txn(&sender, 10);
    executor.execute_and_apply(txn);

    // remove resource fails given no resource were published
    let rem_txn = remove_resource_txn(&sender, 11, vec![module.clone()]);
    let output = executor.execute_transaction(rem_txn);
    assert!(matches!(
        output.status().status(),
        // StatusCode::MISSING_DATA
        Ok(KeptVMStatus::ExecutionFailure { .. })
    ));
    executor.apply_write_set(output.write_set());

    // publish resource
    let add_txn = add_resource_txn(&sender, 12, vec![module.clone()]);
    executor.execute_and_apply(add_txn);

    // borrow resource
    let borrow_txn = borrow_resource_txn(&sender, 13, vec![module.clone()]);
    executor.execute_and_apply(borrow_txn);

    // create a remove and a change resource transaction over the same resource in one block
    let txns = vec![
        Transaction::UserTransaction(remove_resource_txn(&sender, 14, vec![module.clone()])),
        Transaction::UserTransaction(change_resource_txn(&sender, 15, vec![module.clone()])),
    ];
    let output = executor
        .execute_transaction_block(txns)
        .expect("Must execute transactions");
    assert_eq!(
        output[0].status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    assert!(matches!(
        output[1].status().status(),
        // StatusCode::MISSING_DATA
        Ok(KeptVMStatus::ExecutionFailure { .. })
    ));
    for out in output {
        executor.apply_write_set(out.write_set());
    }

    // borrow resource
    let borrow_txn = borrow_resource_txn(&sender, 16, vec![module]);
    let output = executor.execute_transaction(borrow_txn);
    assert!(matches!(
        output.status().status(),
        // StatusCode::MISSING_DATA
        Ok(KeptVMStatus::ExecutionFailure { .. })
    ));
    executor.apply_write_set(output.write_set());
}

fn add_module_txn(sender: &AccountData, seq_num: u64) -> (CompiledModule, SignedUserTransaction) {
    let module_code = format!(
        "
        module {}.M {{
            import 0x1.Signer;
            struct T1 has key {{ v: u64 }}

            public borrow_t1(account: &signer) acquires T1 {{
                let t1: &Self.T1;
            label b0:
                t1 = borrow_global<T1>(Signer.address_of(move(account)));
                return;
            }}

            public change_t1(account: &signer, v: u64) acquires T1 {{
                let t1: &mut Self.T1;
            label b0:
                t1 = borrow_global_mut<T1>(Signer.address_of(move(account)));
                *&mut move(t1).T1::v = move(v);
                return;
            }}

            public remove_t1(account: &signer) acquires T1 {{
                let v: u64;
            label b0:
                T1 {{ v }} = move_from<T1>(Signer.address_of(move(account)));
                return;
            }}

            public publish_t1(account: &signer) {{
            label b0:
                move_to<T1>(move(account), T1 {{ v: 3 }});
                return;
            }}
        }}
        ",
        sender.address(),
    );

    let deps = stdlib_compiled_modules(StdLibOptions::Compiled(Latest));
    let compiler = Compiler {
        deps: deps.iter().collect(),
    };
    let module = compiler
        .into_compiled_module(module_code.as_str())
        .expect("Module compilation failed");
    let mut module_blob = vec![];
    module
        .serialize(&mut module_blob)
        .expect("Module must serialize");
    verify_module(&module).expect("Module must verify");
    (
        module,
        sender
            .account()
            .transaction()
            .module(Module::new(module_blob))
            .sequence_number(seq_num)
            .sign(),
    )
}

fn add_resource_txn(
    sender: &AccountData,
    seq_num: u64,
    extra_deps: Vec<CompiledModule>,
) -> SignedUserTransaction {
    let program = format!(
        "
            import {}.M;

            main(account: signer) {{
            label b0:
                M.publish_t1(&account);
                return;
            }}
        ",
        sender.address(),
    );

    let module = compile_script_with_extra_deps(&program, extra_deps);
    sender
        .account()
        .transaction()
        .script(module)
        .sequence_number(seq_num)
        .sign()
}

fn remove_resource_txn(
    sender: &AccountData,
    seq_num: u64,
    extra_deps: Vec<CompiledModule>,
) -> SignedUserTransaction {
    let program = format!(
        "
            import {}.M;

            main(account: signer) {{
            label b0:
                M.remove_t1(&account);
                return;
            }}
        ",
        sender.address(),
    );

    let module = compile_script_with_extra_deps(&program, extra_deps);
    sender
        .account()
        .transaction()
        .script(module)
        .sequence_number(seq_num)
        .sign()
}

fn borrow_resource_txn(
    sender: &AccountData,
    seq_num: u64,
    extra_deps: Vec<CompiledModule>,
) -> SignedUserTransaction {
    let program = format!(
        "
            import {}.M;

            main(account: signer) {{
            label b0:
                M.borrow_t1(&account);
                return;
            }}
        ",
        sender.address(),
    );

    let module = compile_script_with_extra_deps(&program, extra_deps);
    sender
        .account()
        .transaction()
        .script(module)
        .sequence_number(seq_num)
        .sign()
}

fn change_resource_txn(
    sender: &AccountData,
    seq_num: u64,
    extra_deps: Vec<CompiledModule>,
) -> SignedUserTransaction {
    let program = format!(
        "
            import {}.M;

            main(account: signer) {{
            label b0:
                M.change_t1(&account, 20);
                return;
            }}
        ",
        sender.address(),
    );

    let module = compile_script_with_extra_deps(&program, extra_deps);
    sender
        .account()
        .transaction()
        .script(module)
        .sequence_number(seq_num)
        .sign()
}
