// Copyright (c) Starcoin
// SPDX-License-Identifier: Apache-2.0

//! Support for encoding transactions for common situations.

use crate::account::Account;
use starcoin_config::ChainNetwork;
use starcoin_transaction_builder::{build_signed_empty_txn, peer_to_peer_v2};
use starcoin_types::account::Account as StarcoinAccount;
use starcoin_types::account::DEFAULT_EXPIRATION_TIME;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::transaction::{RawUserTransaction, SignedUserTransaction};
use std::time::{SystemTime, UNIX_EPOCH};
use test_helper::txn::create_account_txn_sent_as_association;

// pub static EMPTY_SCRIPT: Lazy<Vec<u8>> = Lazy::new(|| {
//     let code = "
//     main(account: signer) {
//     label b0:
//       return;
//     }
// ";
//
//     let compiler = Compiler {
//         deps: cached_framework_packages::modules().iter().collect(),
//     };
//     compiler.into_script_blob(code).expect("Failed to compile")
// });

fn now_time() -> u64 {
    let current_time = SystemTime::now();
    current_time.duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub fn empty_txn(
    sender: &Account,
    seq_num: u64,
    _max_gas_amount: u64,
    _gas_unit_price: u64,
) -> SignedUserTransaction {
    build_signed_empty_txn(
        sender.address().clone(),
        &sender.private_key(),
        seq_num,
        now_time() + DEFAULT_EXPIRATION_TIME,
        ChainId::test(),
    )
    // sender
    //     .transaction()
    //     .script(Script::new(EMPTY_SCRIPT.to_vec(), vec![], vec![]))
    //     .sequence_number(seq_num)
    //     .max_gas_amount(max_gas_amount)
    //     .gas_unit_price(gas_unit_price)
    //     .sign()
}

/// Returns a transaction to create a new account with the given arguments.
pub fn create_account_txn(
    _sender: &Account,
    new_account: &Account,
    seq_num: u64,
) -> SignedUserTransaction {
    let (pub_k, priv_k) = new_account.account_keypair();
    let starcoin_acc =
        StarcoinAccount::with_keypair(priv_k, pub_k, Some(new_account.address().clone()));
    create_account_txn_sent_as_association(
        &starcoin_acc,
        seq_num + 1,
        0,
        1,
        &ChainNetwork::new_test(),
    )
    // sender
    //     .transaction()
    //     .payload(starcoin_stdlib::encode_account_create_account(
    //         *new_account.address(),
    //     ))
    //     .sequence_number(seq_num)
    //     .sign()
}

/// Returns a transaction to transfer coin from one account to another (possibly new) one, with the
/// given arguments.
pub fn peer_to_peer_txn(
    sender: &Account,
    receiver: &Account,
    seq_num: u64,
    transfer_amount: u64,
) -> SignedUserTransaction {
    // get a SignedUserTransaction
    // sender
    //     .transaction()
    //     .payload(starcoin_stdlib::encode_test_coin_transfer(
    //         *receiver.address(),
    //         transfer_amount,
    //     ))
    //     .sequence_number(seq_num)
    //     .sign()
    let (pub_k, priv_k) = sender.account_keypair();
    let starcoin_acc = StarcoinAccount::with_keypair(priv_k, pub_k, Some(sender.address().clone()));
    peer_to_peer_v2(
        &starcoin_acc,
        receiver.address(),
        seq_num,
        (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 100) as u64,
        transfer_amount as u128,
        &ChainNetwork::new_test(),
    )
}

/// Returns a transaction to change the keys for the given account.
pub fn rotate_key_txn(
    sender: &Account,
    _new_key_hash: Vec<u8>,
    seq_num: u64,
) -> SignedUserTransaction {
    // sender
    //     .transaction()
    //     .payload(starcoin_stdlib::encode_account_rotate_authentication_key(
    //         new_key_hash,
    //     ))
    //     .sequence_number(seq_num)
    //     .sign()
    // TODO(BobOng): e2e-test
    empty_txn(sender, seq_num, 0, 0)
}

/// Returns a transaction to change the keys for the given account.
pub fn raw_rotate_key_txn(
    sender: &Account,
    _new_key_hash: Vec<u8>,
    seq_num: u64,
) -> RawUserTransaction {
    // sender
    //     .transaction()
    //     .payload(starcoin_stdlib::encode_account_rotate_authentication_key(
    //         new_key_hash,
    //     ))
    //     .sequence_number(seq_num)
    //     .raw()
    // TODO(BobOng): e2e-test
    empty_txn(sender, seq_num, 0, 0).into_raw_transaction()
}
