use starcoin_types::language_storage::TypeTag;
use starcoin_types::{
    account_address::AccountAddress,
    transaction::{RawUserTransaction, SignedUserTransaction, Transaction},
};
use starcoin_vm_types::account_config::stc_type_tag;
use starcoin_vm_types::transaction::{Script, TransactionPayload};
use starcoin_vm_types::transaction_argument::TransactionArgument;
use std::time::Duration;
use vm_runtime::common_transactions::DEFAULT_EXPIRATION_TIME;
use vm_runtime::transaction_scripts::{ACCEPT_COIN_TXN, CREATE_ACCOUNT_TXN, PEER_TO_PEER_TXN};

pub fn build_mint_txn(
    addr: AccountAddress,
    auth_key_prefix: Vec<u8>,
    seq_num: u64,
    amount: u64,
) -> Transaction {
    Transaction::UserTransaction(peer_to_peer_txn_sent_as_association(
        addr,
        auth_key_prefix,
        seq_num,
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

    crate::create_signed_txn_with_association_account(
        PEER_TO_PEER_TXN.clone(),
        vec![stc_type_tag()],
        args,
        seq_num,
        vm_runtime::common_transactions::TXN_RESERVED,
        1,
    )
}
