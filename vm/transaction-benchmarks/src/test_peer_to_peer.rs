// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::transactions::TransactionBencher;
use proptest::arbitrary::any_with;
use starcoin_language_e2e_tests::{
    account::AccountData,
    account_universe::P2PTransferGen,
    common_transactions::peer_to_peer_txn,
    executor::FakeExecutor,
};


#[test]
pub fn bencher_sequence() {
    let default_num_account = 2;
    let default_num_transactions = 1;
    let maxium_transfer_balance = 100;
    let minium_transfer_balance = 10;

    let bencher = TransactionBencher::new(
        any_with::<P2PTransferGen>((minium_transfer_balance, maxium_transfer_balance)),
        // default_num_account,
        // default_num_transactions,
    );
    //bencher.manual_sequence(default_num_account, default_num_transactions, 1, 1);
    let ret = bencher.blockstm_benchmark(
        default_num_account,
        default_num_transactions,
        true,
        true,
        10,
        1,
        num_cpus::get(),
    );
    drop(ret);
}

#[test]
pub fn fake_execute_with_account_data() {
    // Compute gas used by running a placeholder transaction.
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = AccountData::new(1_000_000_000, 10);
    let receiver = AccountData::new(1_000_000_000, 10);
    executor.add_account_data(&sender);
    executor.add_account_data(&receiver);

    let txn = peer_to_peer_txn(sender.account(), receiver.account(), 10, 10);
    let result = executor.execute_block(vec![txn]);
    match result {
        Ok(outputs) => {
            println!("Outputs: {:#?}", outputs);
        }
        Err(err) => {
            println!("Error: {:#?}", err);
        }
    }
}
