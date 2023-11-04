// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use move_core_types::vm_status::{DiscardedVMStatus, KeptVMStatus};
use starcoin_language_e2e_tests::{
    account::Account, compile::compile_module, current_function_name, executor::FakeExecutor,
    transaction_status_eq, utils, versioning::CURRENT_RELEASE_VERSIONS,
};

use starcoin_vm_types::transaction::{ScriptFunction, TransactionStatus};
fn prepare_module(executor: &mut FakeExecutor, account: &Account, seq_num: u64) -> u64 {
    let program = format!(
        "
        module 0x{}.M {{
            f_private(s: &signer) {{
            label b0:
                return;
            }}

            public f_public(s: &signer) {{
            label b0:
                return;
            }}

            public(script) f_script(s: signer) {{
            label b0:
                return;
            }}
        }}
        ",
        account.address(),
    );
    let compiled_module = compile_module(&program).1;

    let txn = account
        .transaction()
        .module(compiled_module)
        .sequence_number(seq_num)
        .sign();

    let output = executor.execute_transaction(txn);
    // module publishing should always succeed
    assert!(transaction_status_eq(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed),
    ));
    executor.apply_write_set(output.write_set());

    seq_num + 1
}

#[test]
fn script_fn_payload_invoke_private_fn() {
    let (mut executor, dr_account, _, _) = utils::start_with_released_df();
    let mut dr_seqno = 1;
    executor.set_golden_file(current_function_name!());

    let sequence_number = 2;
    let account = executor.create_raw_account_data(1_000_000, sequence_number);
    executor.add_account_data(&account);

    let sequence_number = prepare_module(&mut executor, account.account(), sequence_number);
    let txn = account
        .account()
        .transaction()
        .script_function(ScriptFunction::new(
            ModuleId::new(*account.address(), Identifier::new("M").unwrap()),
            Identifier::new("f_private").unwrap(),
            vec![],
            vec![],
        ))
        .sequence_number(sequence_number)
        .sign();

    let output = executor.execute_transaction(txn.clone());
    assert!(transaction_status_eq(
        output.status(),
        &TransactionStatus::Discard(DiscardedVMStatus::FEATURE_UNDER_GATING),
    ));

    // enable the feature
    utils::upgrade_df(
        &mut executor,
        &dr_account,
        &mut dr_seqno,
        Some(CURRENT_RELEASE_VERSIONS.max().unwrap()),
    );

    let output = executor.execute_transaction(txn);
    assert!(transaction_status_eq(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::MiscellaneousError),
    ));
}

#[test]
fn script_fn_payload_invoke_public_fn() {
    let (mut executor, dr_account, _, _) = utils::start_with_released_df();
    let mut dr_seqno = 1;
    executor.set_golden_file(current_function_name!());

    let sequence_number = 2;
    let account = executor.create_raw_account_data(1_000_000, sequence_number);
    executor.add_account_data(&account);

    let sequence_number = prepare_module(&mut executor, account.account(), sequence_number);
    let txn = account
        .account()
        .transaction()
        .script_function(ScriptFunction::new(
            ModuleId::new(*account.address(), Identifier::new("M").unwrap()),
            Identifier::new("f_public").unwrap(),
            vec![],
            vec![],
        ))
        .sequence_number(sequence_number)
        .sign();

    let output = executor.execute_transaction(txn.clone());
    assert!(transaction_status_eq(
        output.status(),
        &TransactionStatus::Discard(DiscardedVMStatus::FEATURE_UNDER_GATING),
    ));

    // enable the feature
    utils::upgrade_df(
        &mut executor,
        &dr_account,
        &mut dr_seqno,
        Some(CURRENT_RELEASE_VERSIONS.max().unwrap()),
    );

    let output = executor.execute_transaction(txn);
    assert!(transaction_status_eq(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::MiscellaneousError),
    ));
}

#[test]
fn script_fn_payload_invoke_script_fn() {
    let (mut executor, dr_account, _, _) = utils::start_with_released_df();
    let mut dr_seqno = 1;
    executor.set_golden_file(current_function_name!());

    let sequence_number = 2;
    let account = executor.create_raw_account_data(1_000_000, sequence_number);
    executor.add_account_data(&account);

    let sequence_number = prepare_module(&mut executor, account.account(), sequence_number);
    let txn = account
        .account()
        .transaction()
        .script_function(ScriptFunction::new(
            ModuleId::new(*account.address(), Identifier::new("M").unwrap()),
            Identifier::new("f_script").unwrap(),
            vec![],
            vec![],
        ))
        .sequence_number(sequence_number)
        .sign();

    let output = executor.execute_transaction(txn.clone());
    assert!(transaction_status_eq(
        output.status(),
        &TransactionStatus::Discard(DiscardedVMStatus::FEATURE_UNDER_GATING),
    ));

    // enable the feature
    utils::upgrade_df(
        &mut executor,
        &dr_account,
        &mut dr_seqno,
        Some(CURRENT_RELEASE_VERSIONS.max().unwrap()),
    );

    let output = executor.execute_transaction(txn);
    assert!(transaction_status_eq(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed),
    ));
}
