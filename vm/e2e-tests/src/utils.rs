// Copyright (c) Starcoin
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::account::{AccountData, AccountRoleSpecifier};
use crate::{account::Account, compile, executor::FakeExecutor};

pub fn close_module_publishing(
    executor: &mut FakeExecutor,
    dr_account: &Account,
    dr_seqno: &mut u64,
) {
    let compiled_script = {
        let script = "
            import 0x1.TransactionPublishingOption;
        main(config: signer) {
        label b0:
            TransactionPublishingOption.set_open_module(&config, false);
            return;
        }
        ";
        compile::compile_script_with_extra_deps(script, vec![])
    };

    let txn = dr_account
        .transaction()
        .script(compiled_script)
        .sequence_number(*dr_seqno)
        .sign();

    executor.execute_and_apply(txn);
    *dr_seqno = dr_seqno.checked_add(1).unwrap();
}

pub fn start_with_released_df() -> (FakeExecutor, Account, Account, Account) {
    let mut executor = FakeExecutor::from_test_genesis();

    let dd_account = Account::new_testing_dd();
    let dr_account = Account::new_starcoin_root();
    let tc_account = Account::new_blessed_tc();

    // dd_account.rotate_key(
    //     bcs::from_bytes(executor::RELEASE_1_1_GENESIS_PRIVKEY).unwrap(),
    //     bcs::from_bytes(executor::RELEASE_1_1_GENESIS_PUBKEY).unwrap(),
    // );
    // dr_account.rotate_key(
    //     bcs::from_bytes(executor::RELEASE_1_1_GENESIS_PRIVKEY).unwrap(),
    //     bcs::from_bytes(executor::RELEASE_1_1_GENESIS_PUBKEY).unwrap(),
    // );
    // tc_account.rotate_key(
    //     bcs::from_bytes(executor::RELEASE_1_1_GENESIS_PRIVKEY).unwrap(),
    //     bcs::from_bytes(executor::RELEASE_1_1_GENESIS_PUBKEY).unwrap(),
    // );

    executor.add_account_data(&AccountData::with_account(
        dd_account.clone(),
        100_000_000,
        0,
        AccountRoleSpecifier::Root,
    ));

    executor.add_account_data(&AccountData::with_account(
        dr_account.clone(),
        100_000_000,
        0,
        AccountRoleSpecifier::Root,
    ));

    executor.add_account_data(&AccountData::with_account(
        tc_account.clone(),
        100_000_000,
        0,
        AccountRoleSpecifier::Root,
    ));

    (executor, dr_account, tc_account, dd_account)

    // let executor = FakeExecutor::from_fresh_genesis();
    // let mut dr_account = Account::new_starcoin_root();
    //
    // let (private_key, public_key) = vm_genesis::GENESIS_KEYPAIR.clone();
    // dr_account.rotate_key(private_key, public_key);
    //
    // (executor, dr_account)
    // (
    //     FakeExecutor::from_fresh_genesis(),
    //     Account::new_starcoin_root(),
    // )
}

pub fn upgrade_df(
    _executor: &mut FakeExecutor,
    _dr_account: &Account,
    _dr_seqno: &mut u64,
    _update_version_number: Option<u64>,
) {
    // TODO(BobOng): e2e-test
    // close_module_publishing(executor, dr_account, dr_seqno);
    // for compiled_module_bytes in cached_framework_packages::module_blobs().iter().cloned() {
    //     let compiled_module_id = CompiledModule::deserialize(&compiled_module_bytes)
    //         .unwrap()
    //         .self_id();
    //     executor.add_module(&compiled_module_id, compiled_module_bytes);
    // }
    //
    // if let Some(version_number) = update_version_number {
    //     executor.execute_and_apply(
    //         dr_account
    //             .transaction()
    //             .payload(starcoin_stdlib::encode_version_set_version(version_number))
    //             .sequence_number(*dr_seqno)
    //             .sign(),
    //     );
    //     *dr_seqno = dr_seqno.checked_add(1).unwrap();
    // }
}
