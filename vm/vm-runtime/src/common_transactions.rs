// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for encoding transactions for common situations.

use crate::starcoin_vm::DEFAULT_CURRENCY_TY;
use crate::transaction_scripts::{CREATE_ACCOUNT_TXN, MINT_TXN, PEER_TO_PEER_TXN};
use crate::{account::create_signed_txn_with_association_account, account::Account};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::starcoin_type_tag;
use starcoin_types::transaction::{
    RawUserTransaction, Script, SignedUserTransaction, TransactionArgument, TransactionPayload,
};
use std::time::Duration;

//TODO
pub const TXN_RESERVED: u64 = 50_000_000;
pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;

/// Returns a transaction to create a new account with the given arguments.
pub fn create_account_txn(
    sender: &Account,
    new_account: &Account,
    seq_num: u64,
    initial_amount: u64,
) -> SignedUserTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*new_account.address()));
    args.push(TransactionArgument::U8Vector(new_account.auth_key_prefix()));
    args.push(TransactionArgument::U64(initial_amount));

    sender.create_signed_txn_with_args(
        CREATE_ACCOUNT_TXN.clone(),
        vec![],
        args,
        seq_num,
        TXN_RESERVED,
        1,
        starcoin_type_tag(),
    )
}

/// Returns a transaction to transfer coin from one account to another (possibly new) one, with the
/// given arguments.
pub fn peer_to_peer_txn(
    sender: &Account,
    receiver: &Account,
    seq_num: u64,
    transfer_amount: u64,
) -> SignedUserTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*receiver.address()));
    args.push(TransactionArgument::U8Vector(receiver.auth_key_prefix()));
    args.push(TransactionArgument::U64(transfer_amount));

    // get a SignedTransaction
    sender.create_signed_txn_with_args(
        PEER_TO_PEER_TXN.clone(),
        vec![starcoin_type_tag()],
        args,
        seq_num,
        TXN_RESERVED, // this is a default for gas
        1,            // this is a default for gas
        starcoin_type_tag(),
    )
}

/// Returns a transaction to mint new funds with the given arguments.
pub fn mint_txn(
    sender: &Account,
    receiver: &Account,
    seq_num: u64,
    transfer_amount: u64,
) -> SignedUserTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*receiver.address()));
    args.push(TransactionArgument::U8Vector(receiver.auth_key_prefix()));
    args.push(TransactionArgument::U64(transfer_amount));

    // get a SignedTransaction
    sender.create_signed_txn_with_args(
        MINT_TXN.clone(),
        vec![],
        args,
        seq_num,
        TXN_RESERVED, // this is a default for gas
        1,            // this is a default for gas
        starcoin_type_tag(),
    )
}

/// Returns a transaction to create a new account with the given arguments.
pub fn create_account_txn_sent_as_association(
    new_account: &Account,
    seq_num: u64,
    initial_amount: u64,
) -> SignedUserTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(*new_account.address()));
    args.push(TransactionArgument::U8Vector(new_account.auth_key_prefix()));
    args.push(TransactionArgument::U64(initial_amount));

    create_signed_txn_with_association_account(
        CREATE_ACCOUNT_TXN.clone(),
        vec![DEFAULT_CURRENCY_TY.clone()],
        args,
        seq_num,
        TXN_RESERVED,
        1,
        starcoin_type_tag(),
    )
}

pub fn peer_to_peer_txn_sent_as_association(
    addr: AccountAddress,
    auth_key_prefix: Vec<u8>,
    seq_num: u64,
    amount: u64,
) -> SignedUserTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(addr));
    args.push(TransactionArgument::U8Vector(auth_key_prefix));
    args.push(TransactionArgument::U64(amount));

    create_signed_txn_with_association_account(
        PEER_TO_PEER_TXN.clone(),
        vec![starcoin_type_tag()],
        args,
        seq_num,
        TXN_RESERVED,
        1,
        starcoin_type_tag(),
    )
}

pub fn raw_peer_to_peer_txn(
    sender: AccountAddress,
    receiver: AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,
    transfer_amount: u64,
    seq_num: u64,
) -> RawUserTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(receiver));
    args.push(TransactionArgument::U8Vector(receiver_auth_key_prefix));
    args.push(TransactionArgument::U64(transfer_amount));

    RawUserTransaction::new(
        sender,
        seq_num,
        TransactionPayload::Script(Script::new(
            PEER_TO_PEER_TXN.clone(),
            vec![starcoin_type_tag()],
            args,
        )),
        TXN_RESERVED,
        1,
        starcoin_type_tag(),
        Duration::from_secs(DEFAULT_EXPIRATION_TIME),
    )
}
