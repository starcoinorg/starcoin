// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_config::ChainNetwork;
use starcoin_vm2_transaction_builder::{
    create_signed_txn_with_association_account, encode_transfer_script_by_token_code,
    DEFAULT_MAX_GAS_AMOUNT,
};
use starcoin_vm2_types::account::Account;
use starcoin_vm2_types::{
    account_address::AccountAddress,
    account_config::{core_code_address, stc_type_tag, G_STC_TOKEN_CODE},
    genesis_config::ChainId,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    transaction::{
        EntryFunction, RawUserTransaction, SignedUserTransaction, Transaction, TransactionPayload,
    },
};
// const NEW_ACCOUNT_AMOUNT: u128 = 1_000_000_000;
// const TRANSFER_AMOUNT: u128 = 1_000;

// pub struct AccountWithSeqNum {
//     account: Account,
//     seq_num: u64,
// }
// pub fn create_account(
//     net: &ChainNetwork,
//     seq_number: u64,
//     account_count: u64,
// ) -> Vec<(Account, SignedUserTransaction)> {
//     assert!(account_count > 0);
//     let mut new_accounts = Vec::new();
//     for i in 0..account_count {
//         let new_account = Account::new();
//         let new_txn = create_account_txn_sent_as_association(
//             &new_account,
//             seq_number + i,
//             NEW_ACCOUNT_AMOUNT,
//             1,
//             net,
//         );
//         new_accounts.push((new_account, new_txn));
//     }
//     new_accounts
// }
//
// pub fn create_account_with_txpool(
//     net: &ChainNetwork,
//     txpool_service: &TxPoolService,
//     account_count: u64,
// ) -> Vec<(Account, SignedUserTransaction)> {
//     let next_sequence_number = txpool_service
//         .next_sequence_number(starcoin_vm_types::account_config::association_address())
//         .unwrap();
//     create_account(net, next_sequence_number, account_count)
// }
//
// pub fn gen_random_txn(
//     net: &ChainNetwork,
//     accounts: Vec<AccountWithSeqNum>,
//     txn_count_per_account: u64,
// ) -> Vec<SignedUserTransaction> {
//     assert!(accounts.len() >= 2);
//     let mut accounts = accounts;
//     let mut txns = Vec::new();
//     let len = accounts.len();
//     for i in 0..len {
//         let account1 = accounts.get(i).expect("account1 is none.").account.clone();
//         for _j in 0..txn_count_per_account {
//             loop {
//                 let index = rand::random::<usize>() % len;
//                 if index != i {
//                     let txn = peer_to_peer_txn(
//                         &account1,
//                         &accounts.get(index).expect("account2 is none").account,
//                         accounts.get(i).expect("account1 is none.").seq_num,
//                         TRANSFER_AMOUNT,
//                         1,
//                         net.chain_id(),
//                     );
//                     txns.push(txn);
//                     accounts.get_mut(i).expect("account1 is none.").seq_num += 1;
//                     break;
//                 }
//             }
//         }
//     }
//     txns
// }
//
// pub fn gen_txns_with_txpool(
//     net: &ChainNetwork,
//     txpool_service: &TxPoolService,
//     accounts: Vec<Account>,
//     txn_count_per_account: u64,
// ) -> Vec<SignedUserTransaction> {
//     let mut accounts_with_seq_num = Vec::new();
//     for account in accounts {
//         let seq_num = txpool_service
//             .next_sequence_number(*account.address())
//             .unwrap();
//         let account_with_seq_num = AccountWithSeqNum { account, seq_num };
//         accounts_with_seq_num.push(account_with_seq_num);
//     }
//     gen_random_txn(net, accounts_with_seq_num, txn_count_per_account)
// }

pub fn build_transfer_from_association(
    receiver: AccountAddress,
    seq_number: u64,
    amount: u128,
    expire_time: u64,
    net: &ChainNetwork,
) -> Transaction {
    starcoin_vm2_transaction_builder::build_transfer_from_association(
        receiver,
        seq_number,
        amount,
        expire_time,
        net.chain_id().id().into(),
        net.genesis_config2(),
    )
}

pub fn create_account_txn_sent_as_association(
    new_account: &Account,
    seq_num: u64,
    initial_amount: u128,
    expiration_timestamp_secs: u64,
    net: &ChainNetwork,
) -> SignedUserTransaction {
    create_signed_txn_with_association_account(
        encode_transfer_script_by_token_code(
            *new_account.address(),
            initial_amount,
            G_STC_TOKEN_CODE.clone(),
        ),
        seq_num,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        expiration_timestamp_secs,
        net.chain_id().id().into(),
        net.genesis_config2(),
    )
}

pub fn build_transfer_txn(
    receiver: AccountAddress,
    sender: AccountAddress,
    seq_number: u64,
    amount: u128,
    gas_price: u64,
    max_gas: u64,
    expire_timestamp_in_secs: u64,
    chain_id: u8,
) -> RawUserTransaction {
    starcoin_vm2_transaction_builder::build_transfer_txn(
        receiver,
        sender,
        seq_number,
        amount,
        gas_price,
        max_gas,
        expire_timestamp_in_secs,
        chain_id.into(),
    )
}

fn build_transaction(
    user_address: AccountAddress,
    seq_number: u64,
    payload: TransactionPayload,
    expire_time: u64,
) -> RawUserTransaction {
    RawUserTransaction::new_with_default_gas_token(
        user_address,
        seq_number,
        payload,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        expire_time + 60 * 60,
        ChainId::test(),
    )
}

pub fn create_user_txn(
    address: AccountAddress,
    seq_number: u64,
    net: &ChainNetwork,
    alice: &Account,
    pre_mint_amount: u128,
    expire_time: u64,
) -> anyhow::Result<Vec<SignedUserTransaction>> {
    let script_function = encode_transfer_script_by_token_code(
        *alice.address(),
        pre_mint_amount / 4,
        G_STC_TOKEN_CODE.clone(),
    );
    let txn = net
        .genesis_config2()
        .sign_with_association(build_transaction(
            address,
            seq_number,
            script_function,
            expire_time + 60 * 60,
        ))?;
    Ok(vec![txn])
}

pub fn build_create_vote_txn(
    alice: &Account,
    seq_number: u64,
    vote_script_function: EntryFunction,
    expire_time: u64,
) -> SignedUserTransaction {
    alice
        .sign_txn(build_transaction(
            *alice.address(),
            seq_number,
            TransactionPayload::EntryFunction(vote_script_function),
            expire_time,
        ))
        .unwrap()
}

pub fn build_cast_vote_txn(
    seq_number: u64,
    alice: &Account,
    action_type_tag: TypeTag,
    voting_power: u128,
    expire_time: u64,
) -> SignedUserTransaction {
    let proposer_id: u64 = 0;
    println!("alice voting power: {}", voting_power);
    let vote_script_function = EntryFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("dao_vote_scripts").unwrap(),
        ),
        Identifier::new("cast_vote").unwrap(),
        vec![stc_type_tag(), action_type_tag],
        vec![
            bcs_ext::to_bytes(alice.address()).unwrap(),
            bcs_ext::to_bytes(&proposer_id).unwrap(),
            bcs_ext::to_bytes(&true).unwrap(),
            bcs_ext::to_bytes(&(voting_power / 2)).unwrap(),
        ],
    );
    alice
        .sign_txn(build_transaction(
            *alice.address(),
            seq_number,
            TransactionPayload::EntryFunction(vote_script_function),
            expire_time,
        ))
        .unwrap()
}

pub fn build_queue_txn(
    seq_number: u64,
    alice: &Account,
    _net: &ChainNetwork,
    action_type_tag: TypeTag,
    expire_time: u64,
) -> SignedUserTransaction {
    let script_function = EntryFunction::new(
        ModuleId::new(core_code_address(), Identifier::new("dao").unwrap()),
        Identifier::new("queue_proposal_action").unwrap(),
        vec![stc_type_tag(), action_type_tag],
        vec![
            bcs_ext::to_bytes(alice.address()).unwrap(),
            bcs_ext::to_bytes(&0u64).unwrap(),
        ],
    );
    alice
        .sign_txn(build_transaction(
            *alice.address(),
            seq_number,
            TransactionPayload::EntryFunction(script_function),
            expire_time,
        ))
        .unwrap()
}

pub fn build_execute_txn(
    seq_number: u64,
    alice: &Account,
    execute_script_function: EntryFunction,
    expire_time: u64,
) -> SignedUserTransaction {
    alice
        .sign_txn(build_transaction(
            *alice.address(),
            seq_number,
            TransactionPayload::EntryFunction(execute_script_function),
            expire_time,
        ))
        .unwrap()
}
