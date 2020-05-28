// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Support for encoding transactions for common situations.

use crate::genesis::GENESIS_KEYPAIR;
use crate::transaction_scripts::{ACCEPT_COIN_TXN, CREATE_ACCOUNT_TXN, PEER_TO_PEER_TXN};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::stc_type_tag;
use starcoin_types::transaction::{
    RawUserTransaction, Script, SignedUserTransaction, TransactionArgument, TransactionPayload,
};
use starcoin_vm_types::account_config;
use starcoin_vm_types::language_storage::TypeTag;
use std::time::Duration;

//TODO move to transaction_builder crate.
pub const TXN_RESERVED: u64 = 2_000_000;
pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;

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
        vec![stc_type_tag()],
        args,
        seq_num,
        TXN_RESERVED,
        1,
    )
}

pub fn raw_peer_to_peer_txn(
    sender: AccountAddress,
    receiver: AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,
    transfer_amount: u64,
    seq_num: u64,
    gas_price: u64,
    max_gas: u64,
    coin_type: TypeTag,
) -> RawUserTransaction {
    let mut args: Vec<TransactionArgument> = Vec::new();
    args.push(TransactionArgument::Address(receiver));
    args.push(TransactionArgument::U8Vector(receiver_auth_key_prefix));
    args.push(TransactionArgument::U64(transfer_amount));

    RawUserTransaction::new(
        sender,
        seq_num,
        TransactionPayload::Script(Script::new(PEER_TO_PEER_TXN.clone(), vec![coin_type], args)),
        max_gas,
        gas_price,
        Duration::from_secs(DEFAULT_EXPIRATION_TIME),
    )
}

pub fn raw_accept_coin_txn(
    sender: AccountAddress,
    seq_num: u64,
    gas_price: u64,
    max_gas: u64,
    coin_type: TypeTag,
) -> RawUserTransaction {
    RawUserTransaction::new(
        sender,
        seq_num,
        TransactionPayload::Script(Script::new(
            ACCEPT_COIN_TXN.clone(),
            vec![coin_type],
            vec![],
        )),
        max_gas,
        gas_price,
        Duration::from_secs(DEFAULT_EXPIRATION_TIME),
    )
}

pub fn encode_create_account_script(
    account_address: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    initial_balance: u64,
) -> Script {
    Script::new(
        CREATE_ACCOUNT_TXN.clone(),
        vec![],
        vec![
            TransactionArgument::Address(*account_address),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U64(initial_balance),
        ],
    )
}

pub fn encode_transfer_script(
    recipient: &AccountAddress,
    auth_key_prefix: Vec<u8>,
    amount: u64,
) -> Script {
    Script::new(
        PEER_TO_PEER_TXN.clone(),
        vec![stc_type_tag()],
        vec![
            TransactionArgument::Address(*recipient),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U64(amount),
        ],
    )
}

pub fn create_signed_txn_with_association_account(
    program: Vec<u8>,
    ty_args: Vec<TypeTag>,
    args: Vec<TransactionArgument>,
    sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
) -> SignedUserTransaction {
    RawUserTransaction::new(
        account_config::association_address(),
        sequence_number,
        TransactionPayload::Script(Script::new(program, ty_args, args)),
        max_gas_amount,
        gas_unit_price,
        // TTL is 86400s. Initial time was set to 0.
        Duration::from_secs(DEFAULT_EXPIRATION_TIME),
    )
    .sign(&GENESIS_KEYPAIR.0, GENESIS_KEYPAIR.1.clone())
    .unwrap()
    .into_inner()
}
