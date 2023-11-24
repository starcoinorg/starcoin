// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bcs_ext::Sample;
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, CORE_CODE_ADDRESS},
    vm_status::{KeptVMStatus, StatusCode},
};
use move_core_types::language_storage::ModuleId;
use starcoin_config::ChainNetwork;
use starcoin_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use starcoin_language_e2e_tests::{
    account::Account as E2eTestAccount, assert_prologue_parity,
    common_transactions::rotate_key_txn, test_with_different_versions, transaction_status_eq,
    versioning::CURRENT_RELEASE_VERSIONS,
};
use starcoin_language_e2e_tests::executor::FakeExecutor;
use starcoin_types::account::Account as StarcoinAccount;
use starcoin_vm_types::{
    transaction::authenticator::AuthenticationKey,
    transaction::{Script, SignedUserTransaction, TransactionStatus},
    transaction::{ScriptFunction, TransactionPayload},
    account_config::stc_type_tag,
};
use test_helper::txn::create_account_txn_sent_as_association;

fn create_account_data_transaction(
    account: Option<E2eTestAccount>,
    init_amount: u128,
    seq_num: u64,
) -> SignedUserTransaction {
    let result_acc = match account {
        Some(acc) => StarcoinAccount::new_genesis_account(acc.address().clone()),
        None => StarcoinAccount::new(),
    };
    create_account_txn_sent_as_association(
        &result_acc,
        seq_num,
        init_amount,
        1,
        &ChainNetwork::new_test(),
    )
}

#[test]
fn invalid_write_set_signer() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = FakeExecutor::from_test_genesis();//test_env.executor;
        //let genesis_account = test_env.dr_account;
        executor.new_block();

        // TODO(BobOng): e2e-testsuit, disable the WriteSetPayload
        // Create a WriteSet that adds an account on a new address.
        //let new_account_data = executor.create_raw_account_data(0, 10);
        //let write_set = new_account_data.to_writeset();
        // Signing the txn with a key that does not match the sender should fail.
        // let writeset_txn = genesis_account
        //     .transaction()
        //     .payload(WriteSetPayload::Direct(ChangeSet::new(write_set, vec![])))
        //     .sequence_number(test_env.dr_sequence_number)
        //     .raw()
        //     .sign(
        //         &new_account_data.account().privkey,
        //         new_account_data.account().pubkey.clone(),
        //     )
        //     .unwrap()
        //     .into_inner();

        let initial_amount: u128 = 100_000_000;
        let new_account_data = executor.create_raw_account_data(0, 10);
        let new_account = new_account_data.account();
        executor.add_account_data(&new_account_data);


        let (public_key, private_key) = test_env.dr_account.ed25519_key_pair();

        //let write_set = new_account_data.to_writeset();
        // Signing the txn with a key that does not match the sender should fail.
        let writeset_txn = new_account
            .transaction()
            .payload(TransactionPayload::ScriptFunction(ScriptFunction::new(
                ModuleId::new(
                    starcoin_vm_types::account_config::core_code_address(),
                    Identifier::new("Account").unwrap(),
                ),
                Identifier::new("create_account_with_initial_amount").unwrap(),
                vec![stc_type_tag()],
                vec![
                    bcs_ext::to_bytes(new_account.address()).unwrap(),
                    bcs_ext::to_bytes(&new_account.auth_key().to_vec()).unwrap(),
                    bcs_ext::to_bytes(&initial_amount).unwrap(),
                ],
            )))
            .sequence_number(test_env.dr_sequence_number)
            .raw()
            .sign(
                &private_key,
                public_key,
            )
            .unwrap()
            .into_inner();
        // let writeset_txn = create_account_data_transaction(Option::None, 0, 0);
        assert_prologue_parity!(
            executor.verify_transaction(writeset_txn.clone()).unwrap().status_code(),
            executor.execute_transaction(writeset_txn).status(),
            StatusCode::INVALID_AUTH_KEY
        );
    }
    }
}

#[test]
fn verify_and_execute_writeset() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        executor.new_block();

        // TODO(BobOng): e2e-testsuit, disable the WriteSetPayload
        // Create a WriteSet that adds an account on a new address.
         let new_account_data = executor.create_raw_account_data(0, 10);
        //let genesis_account = test_env.dd_account;
        // let write_set = new_account_data.to_writeset();
        //
        // // (1) Test that a correct WriteSet is executed as expected.
        // let writeset_txn = genesis_account
        //     .transaction()
        //     .write_set(WriteSetPayload::Direct(ChangeSet::new(
        //         write_set.clone(),
        //         vec![],
        //     )))
        //     .sequence_number(test_env.dr_sequence_number)
        //     .sign();
        let new_account = new_account_data.account();
        let writeset_txn = create_account_data_transaction(Option::Some(new_account.clone()), 0, 0);
        let output = executor.execute_transaction(writeset_txn.clone());
        assert_eq!(
            output.status(),
            &TransactionStatus::Keep(KeptVMStatus::Executed)
        );
        assert!(executor
            .verify_transaction(writeset_txn.clone())
            .is_none());
        executor.apply_write_set(output.write_set());

        let updated_account = executor
            .read_account_resource(&new_account)
            .expect("sender must exist");
        let updated_sender = executor
            .read_account_resource(new_account_data.account())
            .expect("sender must exist");
        let updated_sender_balance = executor
            .read_balance_resource(new_account_data.account())
            .expect("sender balance must exist");

        //assert_eq!(test_env.dr_sequence_number.checked_add(1).unwrap(), updated_account.sequence_number());
        assert_eq!(0, updated_sender_balance.token() as u64);
        //assert_eq!(10, updated_sender.sequence_number());

        // (2) Cannot reapply the same writeset.
        assert_prologue_parity!(
            executor.verify_transaction(writeset_txn.clone()).unwrap().status_code(),
            executor.execute_transaction(writeset_txn).status(),
            StatusCode::SEQUENCE_NUMBER_TOO_OLD
        );

        // (3) Cannot apply the writeset with future sequence number.
        // let writeset_txn = genesis_account
        //     .transaction()
        //     .write_set(WriteSetPayload::Direct(ChangeSet::new(write_set, vec![])))
        //     .sequence_number(test_env.dr_sequence_number.checked_add(10).unwrap())
        //     .sign();
        let writeset_txn = create_account_data_transaction(Option::Some(new_account.clone()), 0, 20);
        let output = executor.execute_transaction(writeset_txn.clone());
        assert_eq!(
            output.status(),
            &TransactionStatus::Discard(StatusCode::SEQUENCE_NUMBER_TOO_NEW)
        );
        // "Too new" sequence numbers are accepted during validation.
        assert!(executor.verify_transaction(writeset_txn).is_none());
    }
    }
}
//
// #[test]
// fn bad_writesets() {
//     test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
//         let mut executor = test_env.executor;
//         let genesis_account = test_env.dr_account;
//         executor.new_block();
//
//         // // Create a WriteSet that adds an account on a new address
//         // let new_account_data = executor.create_raw_account_data(1000, 10);
//         // let write_set = new_account_data.to_writeset();
//         //
//         // // (1) A WriteSet signed by an arbitrary account, not Diem root, should be rejected.
//         // let writeset_txn = new_account_data
//         //     .account()
//         //     .transaction()
//         //     .write_set(WriteSetPayload::Direct(ChangeSet::new(
//         //         write_set.clone(),
//         //         vec![],
//         //     )))
//         //     .sequence_number(0)
//         //     .sign();
//
//         // TODO(BobOng): e2e-testsuit, disabled the WriteSetPayload
//         let writeset_txn = create_account_data_transaction(Some(genesis_account.clone()), 1000, 10);
//         assert_prologue_parity!(
//             executor.verify_transaction(writeset_txn.clone()).unwrap().status_code(),
//             executor.execute_transaction(writeset_txn).status(),
//             StatusCode::REJECTED_WRITE_SET
//         );
//
//         // TODO(BobOng): e2e-testsuit, disabled a invalid Contract Event
//         // (2) A WriteSet containing a reconfiguration event should be dropped.
//         // let event = ContractEvent::new(
//         //     new_epoch_event_key(),
//         //     0,
//         //     stc_type_tag(),
//         //     vec![]
//         // );
//         //writeset_txn = create_contract_event_txn();
//         // let writeset_txn = genesis_account
//         //     .transaction()
//         //     .write_set(WriteSetPayload::Direct(ChangeSet::new(
//         //         write_set,
//         //         vec![event],
//         //     )))
//         //     .sequence_number(test_env.dr_sequence_number)
//         //     .sign();
//         // assert_eq!(
//         //     executor.execute_transaction(writeset_txn).status(),
//         //     &TransactionStatus::Discard(StatusCode::INVALID_WRITE_SET)
//         // );
//
//         // TODO(BobOng): e2e-testsuit, test with unreadable resource
//         // (3) A WriteSet attempting to change DiemWriteSetManager should be dropped.
//         // let key = ResourceKey::new(
//         //     *genesis_account.address(),
//         //     StructTag {
//         //         address: CORE_CODE_ADDRESS,
//         //         module: Identifier::new("Account").unwrap(),
//         //         name: Identifier::new("Account123").unwrap(),
//         //         type_params: vec![],
//         //     },
//         // );
//         // let write_set = WriteSetMut::new(vec![(path, WriteOp::Value(vec![]))])
//         //     .freeze()
//         //     .unwrap();
//         // let writeset_txn = genesis_account
//         //     .transaction()
//         //     .write_set(WriteSetPayload::Direct(ChangeSet::new(write_set, vec![])))
//         //     .sequence_number(test_env.dr_sequence_number)
//         //     .sign();
//         // let output = executor.execute_transaction(writeset_txn);
//         // assert_eq!(
//         //     output.status(),
//         //     &TransactionStatus::Discard(StatusCode::INVALID_WRITE_SET)
//         // );
//         let path = AccessPath::resource_access_path(genesis_account.address().clone(), StructTag {
//             address: CORE_CODE_ADDRESS,
//             module: Identifier::new("Account").unwrap(),
//             name: Identifier::new("Account1234").unwrap(),
//             type_params: vec![],
//         });
//         assert!(executor.get_state_view().get_state_value(&StateKey::AccessPath(path)).unwrap().is_none());
//
//         // TODO(BobOng): e2e-testsuit, 4 same as 3
//         // (4) A WriteSet attempting to change Diem root AccountResource should be dropped.
//         // let key = ResourceKey::new(
//         //     *genesis_account.address(),
//         //     StructTag {
//         //         address: CORE_CODE_ADDRESS,
//         //         module: Identifier::new("DiemAccount").unwrap(),
//         //         name: Identifier::new("DiemAccount").unwrap(),
//         //         type_params: vec![],
//         //     },
//         // );
//         // let path = AccessPath::resource_access_path(key);
//         //
//         // let write_set = WriteSetMut::new(vec![(path, WriteOp::Value(vec![]))])
//         //     .freeze()
//         //     .unwrap();
//         // let writeset_txn = genesis_account
//         //     .transaction()
//         //     .write_set(WriteSetPayload::Direct(ChangeSet::new(write_set, vec![])))
//         //     .sequence_number(test_env.dr_sequence_number)
//         //     .sign();
//         // let output = executor.execute_transaction(writeset_txn);
//         // assert_eq!(
//         //     output.status(),
//         //     &TransactionStatus::Discard(StatusCode::INVALID_WRITE_SET)
//         // );
//
//         // (5) A WriteSet with a bad ChainId should be rejected.
//         // let writeset_txn = genesis_account
//         //     .transaction()
//         //     .write_set(WriteSetPayload::Direct(ChangeSet::new(
//         //         WriteSet::default(),
//         //         vec![],
//         //     )))
//         //     .sequence_number(test_env.dr_sequence_number)
//         //     .chain_id(ChainId::new(NamedChain::DEVNET.id()))
//         //     .sign();
//         // assert_prologue_parity!(
//         //     executor.verify_transaction(writeset_txn.clone()).status(),
//         //     executor.execute_transaction(writeset_txn).status(),
//         //     StatusCode::BAD_CHAIN_ID
//         // );
//
//         // (6) A WriteSet that has expired should be rejected.
//         // let writeset_txn = genesis_account
//         //     .transaction()
//         //     .write_set(WriteSetPayload::Direct(ChangeSet::new(
//         //         WriteSet::default(),
//         //         vec![],
//         //     )))
//         //     .sequence_number(test_env.dr_sequence_number)
//         //     .ttl(0)
//         //     .sign();
//         let writeset_txn = genesis_account
//             .transaction().script(Script::sample())
//             .sequence_number(test_env.dr_sequence_number)
//             .ttl(0)
//             .sign();
//         assert_prologue_parity!(
//             executor.verify_transaction(writeset_txn.clone()).unwrap().status_code(),
//             executor.execute_transaction(writeset_txn).status(),
//             StatusCode::TRANSACTION_EXPIRED
//         );
//
//         // (7) The gas currency specified in the transaction must be valid
//         // (even though WriteSet transactions are not charged for gas).
//         // let writeset_txn = genesis_account
//         //     .transaction()
//         //     .write_set(WriteSetPayload::Direct(ChangeSet::new(
//         //         WriteSet::default(),
//         //         vec![],
//         //     )))
//         //     .sequence_number(test_env.dr_sequence_number)
//         //     .gas_currency_code("Bad_ID")
//         //     .sign();
//         let writeset_txn = genesis_account
//             .transaction().script(Script::sample())
//             .sequence_number(test_env.dr_sequence_number)
//             .gas_currency_code("Bad_ID")
//             .sign();
//         assert_prologue_parity!(
//             executor.verify_transaction(writeset_txn.clone()).unwrap().status_code(),
//             executor.execute_transaction(writeset_txn).status(),
//             StatusCode::INVALID_GAS_SPECIFIER
//         );
//
//         // (8) The gas currency code must also correspond to a registered currency
//         // (even though WriteSet transactions are not charged for gas).
//         // let writeset_txn = genesis_account
//         //     .transaction()
//         //     .write_set(WriteSetPayload::Direct(ChangeSet::new(
//         //         WriteSet::default(),
//         //         vec![],
//         //     )))
//         //     .sequence_number(test_env.dr_sequence_number)
//         //     .gas_currency_code("INVALID")
//         //     .sign();
//         let writeset_txn = genesis_account
//             .transaction().script(Script::sample())
//             .sequence_number(test_env.dr_sequence_number)
//             .gas_currency_code("INVALID")
//             .sign();
//         assert_prologue_parity!(
//             executor.verify_transaction(writeset_txn.clone()).unwrap().status_code(),
//             executor.execute_transaction(writeset_txn).status(),
//             StatusCode::CURRENCY_INFO_DOES_NOT_EXIST
//         );
//     }
//     }
// }

#[test]
fn transfer_and_execute_writeset() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let genesis_account = test_env.dr_account;
        let blessed_account = test_env.tc_account;
        executor.new_block();

        let receiver = executor.create_raw_account_data(100_000, 10);
        executor.add_account_data(&receiver);

        // (1) Association mint some coin
        let privkey = Ed25519PrivateKey::generate_for_testing();
        let pubkey = privkey.public_key();
        let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();

        let new_account_data = executor.create_raw_account_data(0, 10);
        executor.execute_and_apply(rotate_key_txn(&blessed_account, new_key_hash, test_env.tc_sequence_number));

        // (2) Create a WriteSet that adds an account on a new address

        // let write_set = new_account_data.to_writeset();
        //
        // let writeset_txn = genesis_account
        //     .transaction()
        //     .write_set(WriteSetPayload::Direct(ChangeSet::new(write_set, vec![])))
        //     .sequence_number(test_env.dr_sequence_number)
        //     .sign();
        let writeset_txn = create_account_data_transaction(
            Some(new_account_data.account().clone()), 0, 0);
        let output = executor.execute_transaction(writeset_txn.clone());
        assert_eq!(
            output.status(),
            &TransactionStatus::Keep(KeptVMStatus::Executed)
        );
        assert!(executor.verify_transaction(writeset_txn).is_none());

        executor.apply_write_set(output.write_set());

        let updated_diem_root_account = executor
            .read_account_resource(&genesis_account)
            .expect("sender must exist");
        let updated_sender = executor
            .read_account_resource(new_account_data.account())
            .expect("sender must exist");
        let updated_sender_balance = executor
            .read_balance_resource(new_account_data.account())
            .expect("sender balance must exist");

        //assert_eq!(test_env.dr_sequence_number.checked_add(1).unwrap(), updated_diem_root_account.sequence_number());
        assert_eq!(0, updated_sender_balance.token() as u64);
        //assert_eq!(10, updated_sender.sequence_number());

        // (3) Rotate the accounts key
        let privkey = Ed25519PrivateKey::generate_for_testing();
        let pubkey = privkey.public_key();
        let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();
        let txn = rotate_key_txn(new_account_data.account(), new_key_hash, 0);

        // execute transaction
        let output = executor.execute_transaction(txn);
        assert_eq!(
            output.status(),
            &TransactionStatus::Keep(KeptVMStatus::Executed)
        );

        executor.apply_write_set(output.write_set());
    }
    }
}
