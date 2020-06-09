use starcoin_types::language_storage::TypeTag;
use starcoin_types::{
    account_address::AccountAddress,
    transaction::{RawUserTransaction, Transaction},
};
use starcoin_vm_types::account_config::stc_type_tag;
use vm_runtime::common_transactions::{
    peer_to_peer_txn_sent_as_association, raw_accept_coin_txn, raw_peer_to_peer_txn,
};

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
