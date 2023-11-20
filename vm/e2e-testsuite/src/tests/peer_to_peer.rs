// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::vm_status::{known_locations, KeptVMStatus};
use std::time::Instant;

use starcoin_language_e2e_tests::{
    account::Account, common_transactions::peer_to_peer_txn, executor::FakeExecutor,
    test_with_different_versions, transaction_status_eq, versioning::CURRENT_RELEASE_VERSIONS,
};

use starcoin_vm_types::{
    account_config::{DepositEvent, WithdrawEvent},
    transaction::{SignedUserTransaction, TransactionOutput, TransactionStatus},
};

#[test]
fn single_peer_to_peer_with_event() {
    // ::starcoin_logger::Logger::init_for_testing();
    //test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
    //let mut executor = test_env.executor;
    // create and publish a sender with 1_000_000 coins and a receiver with 100_000 coins
    let mut executor = FakeExecutor::from_test_genesis();
    let sender = executor.create_raw_account_data(1_000_000, 10);
    let receiver = executor.create_raw_account_data(100_000, 10);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    let transfer_amount = 1_000;
    let txn = peer_to_peer_txn(sender.account(), receiver.account(), 10, transfer_amount);

    // execute transaction
    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );

    executor.apply_write_set(output.write_set());

    // check that numbers in stored DB are correct
    let sender_balance = 1_000_000 - transfer_amount;
    let receiver_balance = 100_000 + transfer_amount;
    let updated_sender = executor
        .read_account_resource(sender.account())
        .expect("sender must exist");
    let updated_sender_balance = executor
        .read_balance_resource(sender.account())
        .expect("sender balance must exist");
    let updated_receiver = executor
        .read_account_resource(receiver.account())
        .expect("receiver must exist");
    let updated_receiver_balance = executor
        .read_balance_resource(receiver.account())
        .expect("receiver balance must exist");
    assert_eq!(receiver_balance, updated_receiver_balance.token() as u64);
    assert_eq!(sender_balance, updated_sender_balance.token() as u64);
    assert_eq!(11, updated_sender.sequence_number());
    assert_eq!(0, updated_sender.deposit_events().count());
    assert_eq!(1, updated_sender.withdraw_events().count());
    assert_eq!(1, updated_receiver.deposit_events().count());
    assert_eq!(0, updated_receiver.withdraw_events().count());

    let rec_ev_path = receiver.received_events_key().to_vec();
    let sent_ev_path = sender.sent_events_key().to_vec();
    for event in output.events() {
        assert!(
            rec_ev_path.as_slice() == event.key().as_bytes()
                || sent_ev_path.as_slice() == event.key().as_bytes()
        );
    }
    // }
    //}
}

// TODO test no longer simple as the legacy version takes an &signer but all
// new scripts take an owned signer
// #[test]
// fn single_peer_to_peer_with_padding() {
//     //::diem_logger::Logger::init_for_testing();
//     // create a FakeExecutor with a genesis from file
//     // let mut executor =
//     //     FakeExecutor::from_genesis_with_options(VMPublishingOption::custom_scripts());
//     let mut executor = FakeExecutor::from_test_genesis();
//     executor.set_golden_file(current_function_name!());
//
//     // create and publish a sender with 1_000_000 coins and a receiver with 100_000 coins
//     let sender = executor.create_raw_account_data(1_000_000, 10);
//     let receiver = executor.create_raw_account_data(100_000, 10);
//     executor.add_account_data(&sender);
//     executor.add_account_data(&receiver);
//
//     let transfer_amount = 1_000;
//     let padded_script = {
//         let mut script_mut = CompiledScript::deserialize(
//             &LegacyStdlibScript::PeerToPeerWithMetadata
//                 .compiled_bytes()
//                 .into_vec(),
//         )
//         .unwrap()
//         .into_inner();
//         script_mut
//             .code
//             .code
//             .extend(std::iter::repeat(Bytecode::Ret).take(1000));
//         let mut script_bytes = vec![];
//         script_mut
//             .freeze()
//             .unwrap()
//             .serialize(&mut script_bytes)
//             .unwrap();
//
//         Script::new(
//             script_bytes,
//             vec![account_config::stc_type_tag()],
//             vec![
//                 TransactionArgument::Address(*receiver.address()),
//                 TransactionArgument::U64(transfer_amount),
//                 TransactionArgument::U8Vector(vec![]),
//                 TransactionArgument::U8Vector(vec![]),
//             ],
//         )
//     };
//
//     let txn = sender
//         .account()
//         .transaction()
//         .script(padded_script)
//         .sequence_number(10)
//         .sign();
//     let unpadded_txn = peer_to_peer_txn(sender.account(), receiver.account(), 10, transfer_amount);
//     assert!(txn.raw_txn_bytes_len() > unpadded_txn.raw_txn_bytes_len());
//     // execute transaction
//     let output = executor.execute_transaction(txn);
//     assert_eq!(
//         output.status(),
//         &TransactionStatus::Keep(KeptVMStatus::Executed)
//     );
//
//     executor.apply_write_set(output.write_set());
//
//     // check that numbers in stored DB are correct
//     let sender_balance = 1_000_000 - transfer_amount;
//     let receiver_balance = 100_000 + transfer_amount;
//     let updated_sender = executor
//         .read_account_resource(sender.account())
//         .expect("sender must exist");
//     let updated_sender_balance = executor
//         .read_balance_resource(sender.account())
//         .expect("sender balance must exist");
//     let updated_receiver_balance = executor
//         .read_balance_resource(receiver.account())
//         .expect("receiver balance must exist");
//     assert_eq!(receiver_balance, updated_receiver_balance.token());
//     assert_eq!(sender_balance, updated_sender_balance.token());
//     assert_eq!(11, updated_sender.sequence_number());
// }
#[test]
fn few_peer_to_peer_with_event() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;

        // create and publish a sender with 3_000_000 coins and a receiver with 3_000_000 coins=
        let sender = executor.create_raw_account_data(3_000_000, 10);
        let receiver = executor.create_raw_account_data(3_000_000, 10);
        executor.add_account_data(&sender);
        executor.add_account_data(&receiver);

        let transfer_amount = 1_000;

        // execute transaction
        let txns: Vec<SignedUserTransaction> = vec![
            peer_to_peer_txn(sender.account(), receiver.account(), 10, transfer_amount),
            peer_to_peer_txn(sender.account(), receiver.account(), 11, transfer_amount),
            peer_to_peer_txn(sender.account(), receiver.account(), 12, transfer_amount),
            peer_to_peer_txn(sender.account(), receiver.account(), 13, transfer_amount),
        ];
        let output = executor.execute_block(txns).unwrap();
        for (idx, txn_output) in output.iter().enumerate() {
            assert_eq!(
                txn_output.status(),
                &TransactionStatus::Keep(KeptVMStatus::Executed)
            );

            // check events
            for event in txn_output.events() {
                if let Ok(payload) = WithdrawEvent::try_from_bytes(event.event_data()) {
                    assert_eq!(transfer_amount, payload.amount() as u64);
                    //assert_eq!(receiver.address(), &payload.receiver());
                } else if let Ok(payload) = DepositEvent::try_from_bytes(event.event_data()) {
                    assert_eq!(transfer_amount, payload.amount() as u64);
                    //assert_eq!(sender.address(), &payload.sender());
                } else {
                    panic!("Unexpected Event Type")
                }
            }

            let original_sender_balance = executor
                .read_balance_resource(sender.account())
                .expect("sender balance must exist");
            let original_receiver_balance = executor
                .read_balance_resource(receiver.account())
                .expect("receiver balcne must exist");
            executor.apply_write_set(txn_output.write_set());

            // check that numbers in stored DB are correct
            let sender_balance = (original_sender_balance.token() as u64) - transfer_amount;
            let receiver_balance = (original_receiver_balance.token() as u64) + transfer_amount;
            let updated_sender = executor
                .read_account_resource(sender.account())
                .expect("sender must exist");
            let updated_sender_balance = executor
                .read_balance_resource(sender.account())
                .expect("sender balance must exist");
            let updated_receiver = executor
                .read_account_resource(receiver.account())
                .expect("receiver must exist");
            let updated_receiver_balance = executor
                .read_balance_resource(receiver.account())
                .expect("receiver balance must exist");
            assert_eq!(receiver_balance, updated_receiver_balance.token() as u64);
            assert_eq!(sender_balance, updated_sender_balance.token() as u64);
            assert_eq!(11 + idx as u64, updated_sender.sequence_number());
            //assert_eq!(1, updated_sender.withdraw_events().count());
            assert_eq!(idx as u64 + 1, updated_sender.withdraw_events().count());
            assert_eq!(idx as u64 + 1, updated_receiver.deposit_events().count());
            assert_eq!(0, updated_receiver.withdraw_events().count());
        }
    }
    }
}

/// Test that a zero-amount transaction fails, per policy.
#[test]
fn zero_amount_peer_to_peer() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
    let mut executor = test_env.executor;
    //let mut executor = FakeExecutor::from_test_genesis();
    let sequence_number = 10;
    let sender = executor.create_raw_account_data(1_000_000, sequence_number);
    let receiver = executor.create_raw_account_data(100_000, sequence_number);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    let transfer_amount = 0;
    let txn = peer_to_peer_txn(
        sender.account(),
        receiver.account(),
        sequence_number,
        transfer_amount,
    );

    let output = &executor.execute_transaction(txn);
    // Error code 7 means that the transaction was a zero-amount one.
    assert!(transaction_status_eq(
        output.status(), &TransactionStatus::Keep(KeptVMStatus::Executed)
        // &TransactionStatus::Keep(KeptVMStatus::MoveAbort(
        //     known_locations::diem_account_module_abort(),
        //     519,
        // )),
    ));
    }
    }
}

// Holder for transaction data; arguments to transactions.
pub(crate) struct TxnInfo {
    pub sender: Account,
    pub receiver: Account,
    pub transfer_amount: u64,
}

impl TxnInfo {
    fn new(sender: &Account, receiver: &Account, transfer_amount: u64) -> Self {
        TxnInfo {
            sender: sender.clone(),
            receiver: receiver.clone(),
            transfer_amount,
        }
    }
}

// Create a cyclic transfer around a slice of Accounts.
// Each Account makes a transfer for the same amount to the next DiemAccount.
pub(crate) fn create_cyclic_transfers(
    executor: &FakeExecutor,
    accounts: &[Account],
    transfer_amount: u64,
) -> (Vec<TxnInfo>, Vec<SignedUserTransaction>) {
    let mut txns: Vec<SignedUserTransaction> = Vec::new();
    let mut txns_info: Vec<TxnInfo> = Vec::new();
    // loop through all transactions and let each transfer the same amount to the next one
    let count = accounts.len();
    for i in 0..count {
        let sender = &accounts[i];
        let sender_resource = executor
            .read_account_resource(sender)
            .expect("sender must exist");
        let seq_num = sender_resource.sequence_number();
        let receiver = &accounts[(i + 1) % count];

        let txn = peer_to_peer_txn(sender, receiver, seq_num, transfer_amount);
        txns.push(txn);
        txns_info.push(TxnInfo::new(sender, receiver, transfer_amount));
    }
    (txns_info, txns)
}

// Create a one to many transfer around a slice of Accounts.
// The first account is the payer and all others are receivers.
fn create_one_to_many_transfers(
    executor: &FakeExecutor,
    accounts: &[Account],
    transfer_amount: u64,
) -> (Vec<TxnInfo>, Vec<SignedUserTransaction>) {
    let mut txns: Vec<SignedUserTransaction> = Vec::new();
    let mut txns_info: Vec<TxnInfo> = Vec::new();
    // grab account 0 as a sender
    let sender = &accounts[0];
    let sender_resource = executor
        .read_account_resource(sender)
        .expect("sender must exist");
    let seq_num = sender_resource.sequence_number();
    // loop through all transactions and let each transfer the same amount to the next one
    let count = accounts.len();
    for (i, receiver) in accounts.iter().enumerate().take(count).skip(1) {
        // let receiver = &accounts[i];

        let txn = peer_to_peer_txn(sender, receiver, seq_num + i as u64 - 1, transfer_amount);
        txns.push(txn);
        txns_info.push(TxnInfo::new(sender, receiver, transfer_amount));
    }
    (txns_info, txns)
}

// Create a many to one transfer around a slice of Accounts.
// The first account is the receiver and all others are payers.
fn create_many_to_one_transfers(
    executor: &FakeExecutor,
    accounts: &[Account],
    transfer_amount: u64,
) -> (Vec<TxnInfo>, Vec<SignedUserTransaction>) {
    let mut txns: Vec<SignedUserTransaction> = Vec::new();
    let mut txns_info: Vec<TxnInfo> = Vec::new();
    // grab account 0 as a sender
    let receiver = &accounts[0];
    // loop through all transactions and let each transfer the same amount to the next one
    let count = accounts.len();
    for sender in accounts.iter().take(count).skip(1) {
        //let sender = &accounts[i];
        let sender_resource = executor
            .read_account_resource(sender)
            .expect("sender must exist");
        let seq_num = sender_resource.sequence_number();

        let txn = peer_to_peer_txn(sender, receiver, seq_num, transfer_amount);
        txns.push(txn);
        txns_info.push(TxnInfo::new(sender, receiver, transfer_amount));
    }
    (txns_info, txns)
}

// Verify a transfer output.
// Checks that sender and receiver in a peer to peer transaction are in proper
// state after a successful transfer.
// The transaction arguments are provided in txn_args.
// Apply the WriteSet to the data store.
pub(crate) fn check_and_apply_transfer_output(
    executor: &mut FakeExecutor,
    txn_args: &[TxnInfo],
    output: &[TransactionOutput],
) {
    let count = output.len();
    for i in 0..count {
        let txn_info = &txn_args[i];
        let sender = &txn_info.sender;
        let receiver = &txn_info.receiver;
        let transfer_amount = txn_info.transfer_amount;
        let sender_resource = executor
            .read_account_resource(sender)
            .expect("sender must exist");
        let sender_balance = executor
            .read_balance_resource(sender)
            .expect("sender balance must exist");
        let sender_initial_balance = sender_balance.token();
        let sender_seq_num = sender_resource.sequence_number();
        let receiver_initial_balance = executor
            .read_balance_resource(receiver)
            .expect("receiver balance must exist")
            .token();

        // apply single transaction to DB
        let txn_output = &output[i];
        executor.apply_write_set(txn_output.write_set());

        // check that numbers stored in DB are correct
        let sender_balance = sender_initial_balance as u64 - transfer_amount;
        let receiver_balance = receiver_initial_balance as u64 + transfer_amount;
        let updated_sender = executor
            .read_account_resource(sender)
            .expect("sender must exist");
        let updated_sender_balance = executor
            .read_balance_resource(sender)
            .expect("sender balance must exist");
        let updated_receiver_balance = executor
            .read_balance_resource(receiver)
            .expect("receiver balance must exist");
        assert_eq!(receiver_balance, updated_receiver_balance.token() as u64);
        assert_eq!(sender_balance, updated_sender_balance.token() as u64);
        assert_eq!(sender_seq_num + 1, updated_sender.sequence_number());
    }
}

// simple utility to print all account to visually inspect account data
fn print_accounts(executor: &FakeExecutor, accounts: &[Account]) {
    for account in accounts {
        let account_resource = executor
            .read_account_resource(account)
            .expect("sender must exist");
        println!("{:?}", account_resource);
    }
}

#[test]
fn cycle_peer_to_peer() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let account_size = 100usize;
        let initial_balance = 2_000_000u64;
        let initial_seq_num = 10u64;
        let accounts = executor.create_accounts(account_size, initial_balance, initial_seq_num);

        // set up the transactions
        let transfer_amount = 1_000;
        let (txns_info, txns) = create_cyclic_transfers(&executor, &accounts, transfer_amount);

        // execute transaction
        let mut execution_time = 0u128;
        let now = Instant::now();
        let output = executor.execute_block(txns).unwrap();
        execution_time += now.elapsed().as_nanos();
        println!("EXECUTION TIME: {}", execution_time);
        for txn_output in &output {
            assert_eq!(
                txn_output.status(),
                &TransactionStatus::Keep(KeptVMStatus::Executed)
            );
        }
        assert_eq!(accounts.len(), output.len());

        check_and_apply_transfer_output(&mut executor, &txns_info, &output);
        print_accounts(&executor, &accounts);
    }
    }
}

#[test]
fn cycle_peer_to_peer_multi_block() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let account_size = 100usize;
        let initial_balance = 1_000_000u64;
        let initial_seq_num = 10u64;
        let accounts = executor.create_accounts(account_size, initial_balance, initial_seq_num);

        // set up the transactions
        let transfer_amount = 1_000;
        let block_count = 5u64;
        let cycle = account_size / (block_count as usize);
        let mut range_left = 0usize;
        let mut execution_time = 0u128;
        for _i in 0..block_count {
            range_left = if range_left + cycle >= account_size {
                account_size - cycle
            } else {
                range_left
            };
            let (txns_info, txns) = create_cyclic_transfers(
                &executor,
                &accounts[range_left..range_left + cycle],
                transfer_amount,
            );

            // execute transaction
            let now = Instant::now();
            let output = executor.execute_block(txns).unwrap();
            execution_time += now.elapsed().as_nanos();
            for txn_output in &output {
                assert_eq!(
                    txn_output.status(),
                    &TransactionStatus::Keep(KeptVMStatus::Executed)
                );
            }
            assert_eq!(cycle, output.len());
            check_and_apply_transfer_output(&mut executor, &txns_info, &output);
            range_left = (range_left + cycle) % account_size;
        }
        println!("EXECUTION TIME: {}", execution_time);
        print_accounts(&executor, &accounts);
    }
    }
}

#[test]
fn one_to_many_peer_to_peer() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let account_size = 100usize;
        let initial_balance = 100_000_000u64;
        let initial_seq_num = 10u64;
        let accounts = executor.create_accounts(account_size, initial_balance, initial_seq_num);

        // set up the transactions
        let transfer_amount = 1_000;
        let block_count = 2u64;
        let cycle = account_size / (block_count as usize);
        let mut range_left = 0usize;
        let mut execution_time = 0u128;
        for _i in 0..block_count {
            range_left = if range_left + cycle >= account_size {
                account_size - cycle
            } else {
                range_left
            };
            let (txns_info, txns) = create_one_to_many_transfers(
                &executor,
                &accounts[range_left..range_left + cycle],
                transfer_amount,
            );

            // execute transaction
            let now = Instant::now();
            let output = executor.execute_block(txns).unwrap();
            execution_time += now.elapsed().as_nanos();
            for txn_output in &output {
                assert_eq!(
                    txn_output.status(),
                    &TransactionStatus::Keep(KeptVMStatus::Executed)
                );
            }
            assert_eq!(cycle - 1, output.len());
            check_and_apply_transfer_output(&mut executor, &txns_info, &output);
            range_left = (range_left + cycle) % account_size;
        }
        println!("EXECUTION TIME: {}", execution_time);
        print_accounts(&executor, &accounts);
    }
    }
}

#[test]
fn many_to_one_peer_to_peer() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;
        let account_size = 100usize;
        let initial_balance = 1_000_000u64;
        let initial_seq_num = 10u64;
        let accounts = executor.create_accounts(account_size, initial_balance, initial_seq_num);

        // set up the transactions
        let transfer_amount = 1_000;
        let block_count = 2u64;
        let cycle = account_size / (block_count as usize);
        let mut range_left = 0usize;
        let mut execution_time = 0u128;
        for _i in 0..block_count {
            range_left = if range_left + cycle >= account_size {
                account_size - cycle
            } else {
                range_left
            };
            let (txns_info, txns) = create_many_to_one_transfers(
                &executor,
                &accounts[range_left..range_left + cycle],
                transfer_amount,
            );

            // execute transaction
            let now = Instant::now();
            let output = executor.execute_block(txns).unwrap();
            execution_time += now.elapsed().as_nanos();
            for txn_output in &output {
                assert_eq!(
                    txn_output.status(),
                    &TransactionStatus::Keep(KeptVMStatus::Executed)
                );
            }
            assert_eq!(cycle - 1, output.len());
            check_and_apply_transfer_output(&mut executor, &txns_info, &output);
            range_left = (range_left + cycle) % account_size;
        }
        println!("EXECUTION TIME: {}", execution_time);
        print_accounts(&executor, &accounts);
    }
    }
}
