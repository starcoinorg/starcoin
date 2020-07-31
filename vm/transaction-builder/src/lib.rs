// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_config::ChainNetwork;
use starcoin_logger::prelude::*;
use starcoin_vm_types::access::ModuleAccess;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config;
use starcoin_vm_types::chain_config::ChainId;
use starcoin_vm_types::token::stc::STC_TOKEN_CODE;
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::transaction::authenticator::AuthenticationKey;
use starcoin_vm_types::transaction::{
    Module, Package, RawUserTransaction, Script, SignedUserTransaction, Transaction,
    TransactionArgument, TransactionPayload,
};
use stdlib::init_scripts::InitScript;
pub use stdlib::transaction_scripts::{CompiledBytes, StdlibScript};
pub use stdlib::{stdlib_modules, StdLibOptions};

pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;
pub const DEFAULT_MAX_GAS_AMOUNT: u64 = 20000;

pub fn build_transfer_from_association(
    addr: AccountAddress,
    auth_key_prefix: Vec<u8>,
    association_sequence_num: u64,
    amount: u128,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> Transaction {
    Transaction::UserTransaction(peer_to_peer_txn_sent_as_association(
        addr,
        auth_key_prefix,
        association_sequence_num,
        amount,
        expiration_timestamp_secs,
        chain_id,
    ))
}

pub fn build_transfer_txn(
    sender: AccountAddress,
    receiver: AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,
    seq_num: u64,
    amount: u128,
    gas_price: u64,
    max_gas: u64,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> RawUserTransaction {
    build_transfer_txn_by_token_type(
        sender,
        receiver,
        receiver_auth_key_prefix,
        seq_num,
        amount,
        gas_price,
        max_gas,
        STC_TOKEN_CODE.clone(),
        expiration_timestamp_secs,
        chain_id,
    )
}

pub fn build_transfer_txn_by_token_type(
    sender: AccountAddress,
    receiver: AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,
    seq_num: u64,
    amount: u128,
    gas_price: u64,
    max_gas: u64,
    token_code: TokenCode,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> RawUserTransaction {
    raw_peer_to_peer_txn(
        sender,
        receiver,
        receiver_auth_key_prefix,
        amount,
        seq_num,
        gas_price,
        max_gas,
        token_code,
        expiration_timestamp_secs,
        chain_id,
    )
}

pub fn build_accept_token_txn(
    sender: AccountAddress,
    seq_num: u64,
    gas_price: u64,
    max_gas: u64,
    token_code: TokenCode,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> RawUserTransaction {
    raw_accept_token_txn(
        sender,
        seq_num,
        gas_price,
        max_gas,
        token_code,
        expiration_timestamp_secs,
        chain_id,
    )
}

pub fn raw_peer_to_peer_txn(
    sender: AccountAddress,
    receiver: AccountAddress,
    receiver_auth_key_prefix: Vec<u8>,
    transfer_amount: u128,
    seq_num: u64,
    gas_price: u64,
    max_gas: u64,
    token_code: TokenCode,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> RawUserTransaction {
    RawUserTransaction::new(
        sender,
        seq_num,
        TransactionPayload::Script(encode_transfer_script_by_token_code(
            receiver,
            receiver_auth_key_prefix,
            transfer_amount,
            token_code,
        )),
        max_gas,
        gas_price,
        expiration_timestamp_secs,
        chain_id,
    )
}

pub fn raw_accept_token_txn(
    sender: AccountAddress,
    seq_num: u64,
    gas_price: u64,
    max_gas: u64,
    token_code: TokenCode,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> RawUserTransaction {
    RawUserTransaction::new(
        sender,
        seq_num,
        TransactionPayload::Script(Script::new(
            StdlibScript::AcceptToken.compiled_bytes().into_vec(),
            vec![token_code.into()],
            vec![],
        )),
        max_gas,
        gas_price,
        expiration_timestamp_secs,
        chain_id,
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
    recipient: AccountAddress,
    auth_key_prefix: Vec<u8>,
    amount: u128,
) -> Script {
    encode_transfer_script_by_token_code(recipient, auth_key_prefix, amount, STC_TOKEN_CODE.clone())
}

pub fn encode_transfer_script_by_token_code(
    recipient: AccountAddress,
    auth_key_prefix: Vec<u8>,
    amount: u128,
    token_code: TokenCode,
) -> Script {
    Script::new(
        StdlibScript::PeerToPeer.compiled_bytes().into_vec(),
        vec![token_code.into()],
        vec![
            TransactionArgument::Address(recipient),
            TransactionArgument::U8Vector(auth_key_prefix),
            TransactionArgument::U128(amount),
        ],
    )
}

pub fn peer_to_peer_txn_sent_as_association(
    recipient: AccountAddress,
    auth_key_prefix: Vec<u8>,
    seq_num: u64,
    amount: u128,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> SignedUserTransaction {
    crate::create_signed_txn_with_association_account(
        TransactionPayload::Script(encode_transfer_script(recipient, auth_key_prefix, amount)),
        seq_num,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        expiration_timestamp_secs,
        chain_id,
    )
}

//this only work for DEV,
pub fn create_signed_txn_with_association_account(
    payload: TransactionPayload,
    sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> SignedUserTransaction {
    let raw_txn = RawUserTransaction::new(
        account_config::association_address(),
        sequence_number,
        payload,
        max_gas_amount,
        gas_unit_price,
        expiration_timestamp_secs,
        chain_id,
    );
    ChainNetwork::Dev
        .sign_with_association(raw_txn)
        .expect("Sign txn should work.")
}

pub fn build_stdlib_package(
    net: ChainNetwork,
    stdlib_option: StdLibOptions,
    with_init_script: bool,
) -> Result<Package> {
    let modules = stdlib_modules(stdlib_option);
    let mut package = Package::new_with_modules(
        modules
            .iter()
            .map(|m| {
                let mut blob = vec![];
                m.serialize(&mut blob)
                    .expect("serializing stdlib must work");
                let handle = &m.module_handles()[0];
                debug!(
                    "Add module: {}::{}",
                    m.address_identifier_at(handle.address),
                    m.identifier_at(handle.name)
                );
                Module::new(blob)
            })
            .collect(),
    )?;
    if with_init_script {
        let chain_config = net.get_config();
        let chain_id = net.chain_id().id();
        let consensus_strategy = net.consensus();
        let genesis_timestamp = net.get_config().timestamp;

        let genesis_auth_key = chain_config
            .genesis_key_pair
            .as_ref()
            .map(|(_, public_key)| AuthenticationKey::ed25519(&public_key).to_vec())
            .unwrap_or_else(Vec::new);

        let association_auth_key =
            AuthenticationKey::ed25519(&chain_config.association_key_pair.1).to_vec();

        let publish_option_bytes = scs::to_bytes(&chain_config.vm_config.publishing_option)
            .expect("Cannot serialize publishing option");
        let instruction_schedule =
            scs::to_bytes(&chain_config.vm_config.gas_schedule.instruction_table)
                .expect("Cannot serialize gas schedule");
        let native_schedule = scs::to_bytes(&chain_config.vm_config.gas_schedule.native_table)
            .expect("Cannot serialize gas schedule");

        let pre_mine_percent = chain_config.pre_mine_percent;

        package.set_init_script(Script::new(
            InitScript::GenesisInit.compiled_bytes().into_vec(),
            vec![],
            vec![
                TransactionArgument::U8Vector(publish_option_bytes),
                TransactionArgument::U8Vector(instruction_schedule),
                TransactionArgument::U8Vector(native_schedule),
                TransactionArgument::U64(chain_config.reward_delay),
                TransactionArgument::U64(chain_config.uncle_rate_target),
                TransactionArgument::U64(chain_config.epoch_time_target),
                TransactionArgument::U64(chain_config.reward_half_epoch),
                TransactionArgument::U64(chain_config.init_block_time_target),
                TransactionArgument::U64(chain_config.block_difficulty_window),
                TransactionArgument::U64(chain_config.reward_per_uncle_percent),
                TransactionArgument::U64(chain_config.min_time_target),
                TransactionArgument::U64(chain_config.max_uncles_per_block),
                TransactionArgument::U128(chain_config.total_supply),
                TransactionArgument::U64(pre_mine_percent),
                TransactionArgument::U8Vector(chain_config.parent_hash.to_vec()),
                TransactionArgument::U8Vector(association_auth_key),
                TransactionArgument::U8Vector(genesis_auth_key),
                TransactionArgument::U8(chain_id),
                TransactionArgument::U8(consensus_strategy.value()),
                TransactionArgument::U64(genesis_timestamp),
                TransactionArgument::U64(chain_config.vm_config.block_gas_limit),
            ],
        ));
    }
    Ok(package)
}
