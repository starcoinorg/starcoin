// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_config::ChainNetwork;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config;
use starcoin_vm_types::account_config::stc_type_tag;
use starcoin_vm_types::language_storage::TypeTag;
use starcoin_vm_types::transaction::helpers::TransactionSigner;
use starcoin_vm_types::transaction::{
    RawUserTransaction, Script, SignedUserTransaction, Transaction, TransactionArgument,
    TransactionPayload,
};
use std::time::Duration;

pub use stdlib::transaction_scripts::{CompiledBytes, StdlibScript};
pub use stdlib::{stdlib_modules, StdLibOptions};

pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;
pub const TXN_RESERVED: u64 = 2_000_000;

pub fn build_transfer_from_association(
    addr: AccountAddress,
    auth_key_prefix: Vec<u8>,
    association_sequence_num: u64,
    amount: u64,
) -> Transaction {
    Transaction::UserTransaction(peer_to_peer_txn_sent_as_association(
        addr,
        auth_key_prefix,
        association_sequence_num,
        amount,
    ))
}

pub fn build_transfer_txn(
    sender: AccountAddress,
    receiver: AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,
    seq_num: u64,
    amount: u64,
    gas_price: u64,
    max_gas: u64,
) -> RawUserTransaction {
    build_transfer_txn_by_coin_type(
        sender,
        receiver,
        receiver_auth_key_prefix,
        seq_num,
        amount,
        gas_price,
        max_gas,
        stc_type_tag(),
    )
}

pub fn build_transfer_txn_by_coin_type(
    sender: AccountAddress,
    receiver: AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,
    seq_num: u64,
    amount: u64,
    gas_price: u64,
    max_gas: u64,
    coin_type: TypeTag,
) -> RawUserTransaction {
    raw_peer_to_peer_txn(
        sender,
        receiver,
        receiver_auth_key_prefix,
        amount,
        seq_num,
        gas_price,
        max_gas,
        coin_type,
    )
}

pub fn build_accept_coin_txn(
    sender: AccountAddress,
    seq_num: u64,
    gas_price: u64,
    max_gas: u64,
    coin_type: TypeTag,
) -> RawUserTransaction {
    raw_accept_coin_txn(sender, seq_num, gas_price, max_gas, coin_type)
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
        TransactionPayload::Script(Script::new(
            StdlibScript::PeerToPeer.compiled_bytes().into_vec(),
            vec![coin_type],
            args,
        )),
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
            StdlibScript::AcceptCoin.compiled_bytes().into_vec(),
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
        StdlibScript::CreateAccount.compiled_bytes().into_vec(),
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
        StdlibScript::PeerToPeer.compiled_bytes().into_vec(),
        vec![stc_type_tag()],
        vec![
            TransactionArgument::Address(*recipient),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U64(amount),
        ],
    )
}

pub fn peer_to_peer_txn_sent_as_association(
    recipient: AccountAddress,
    auth_key_prefix: Vec<u8>,
    seq_num: u64,
    amount: u64,
) -> SignedUserTransaction {
    crate::create_signed_txn_with_association_account(
        encode_transfer_script(&recipient, auth_key_prefix, amount),
        seq_num,
        TXN_RESERVED,
        1,
    )
}

//this only work for DEV,
//TODO move to ChainNetwork::DEV ?
pub fn create_signed_txn_with_association_account(
    script: Script,
    sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
) -> SignedUserTransaction {
    ChainNetwork::Dev
        .get_config()
        .pre_mine_config
        .as_ref()
        .expect("Dev network pre mine config should exist")
        .sign_txn(RawUserTransaction::new(
            account_config::association_address(),
            sequence_number,
            TransactionPayload::Script(script),
            max_gas_amount,
            gas_unit_price,
            // TTL is 86400s. Initial time was set to 0.
            Duration::from_secs(DEFAULT_EXPIRATION_TIME),
        ))
        .expect("Sign txn should work.")
}
