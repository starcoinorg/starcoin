// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Account;
use starcoin_config::ChainNetwork;
use starcoin_executor::account::{create_account_txn_sent_as_association, peer_to_peer_txn};
use starcoin_txpool::TxPoolService;
use starcoin_txpool_api::TxPoolSyncService;
use starcoin_types::account_config;
use starcoin_types::transaction::SignedUserTransaction;

const NEW_ACCOUNT_AMOUNT: u128 = 1_000_000_000;
const TRANSFER_AMOUNT: u128 = 1_000;

pub struct AccountWithSeqNum {
    account: Account,
    seq_num: u64,
}

pub fn create_account(
    net: &ChainNetwork,
    seq_number: u64,
    account_count: u64,
) -> Vec<(Account, SignedUserTransaction)> {
    assert!(account_count > 0);
    let mut new_accounts = Vec::new();
    for i in 0..account_count {
        let new_account = Account::new();
        let new_txn = create_account_txn_sent_as_association(
            &new_account,
            seq_number + i,
            NEW_ACCOUNT_AMOUNT,
            1,
            net,
        );
        new_accounts.push((new_account, new_txn));
    }
    new_accounts
}

pub fn create_account_with_txpool(
    net: &ChainNetwork,
    txpool_service: &TxPoolService,
    account_count: u64,
) -> Vec<(Account, SignedUserTransaction)> {
    let next_sequence_number = txpool_service
        .next_sequence_number(account_config::association_address())
        .unwrap();
    create_account(net, next_sequence_number, account_count)
}

pub fn gen_random_txn(
    net: &ChainNetwork,
    accounts: Vec<AccountWithSeqNum>,
    txn_count_per_account: u64,
) -> Vec<SignedUserTransaction> {
    assert!(accounts.len() >= 2);
    let mut accounts = accounts;
    let mut txns = Vec::new();
    let len = accounts.len();
    for i in 0..len {
        let account1 = accounts.get(i).expect("account1 is none.").account.clone();
        for _j in 0..txn_count_per_account {
            loop {
                let index = rand::random::<usize>() % len;
                if index != i {
                    let txn = peer_to_peer_txn(
                        &account1,
                        &accounts.get(index).expect("account2 is none").account,
                        accounts.get(i).expect("account1 is none.").seq_num,
                        TRANSFER_AMOUNT,
                        1,
                        net.chain_id(),
                    );
                    txns.push(txn);
                    accounts.get_mut(i).expect("account1 is none.").seq_num += 1;
                    break;
                }
            }
        }
    }
    txns
}

pub fn gen_txns_with_txpool(
    net: &ChainNetwork,
    txpool_service: &TxPoolService,
    accounts: Vec<Account>,
    txn_count_per_account: u64,
) -> Vec<SignedUserTransaction> {
    let mut accounts_with_seq_num = Vec::new();
    for account in accounts {
        let seq_num = txpool_service
            .next_sequence_number(*account.address())
            .unwrap();
        let account_with_seq_num = AccountWithSeqNum { account, seq_num };
        accounts_with_seq_num.push(account_with_seq_num);
    }
    gen_random_txn(net, accounts_with_seq_num, txn_count_per_account)
}
