// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_config::ChainNetwork;
use starcoin_logger::prelude::*;
use starcoin_vm_types::access::ModuleAccess;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config;
use starcoin_vm_types::gas_schedule::GasAlgebra;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::language_storage::TypeTag;
use starcoin_vm_types::token::stc::STC_TOKEN_CODE;
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::transaction::authenticator::AuthenticationKey;
use starcoin_vm_types::transaction::{
    Module, Package, RawUserTransaction, Script, SignedUserTransaction, Transaction,
    TransactionArgument, TransactionPayload,
};
pub use stdlib::init_scripts::{compiled_init_script, InitScript};
pub use stdlib::transaction_scripts::compiled_transaction_script;
pub use stdlib::transaction_scripts::{CompiledBytes, StdlibScript};
pub use stdlib::{stdlib_modules, StdLibOptions, StdlibVersion};

pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;
pub const DEFAULT_MAX_GAS_AMOUNT: u64 = 20000;

pub fn build_transfer_from_association(
    addr: AccountAddress,
    recipient_public_key_vec: Vec<u8>,
    association_sequence_num: u64,
    amount: u128,
    expiration_timestamp_secs: u64,
    net: &ChainNetwork,
) -> Transaction {
    Transaction::UserTransaction(peer_to_peer_txn_sent_as_association(
        addr,
        recipient_public_key_vec,
        association_sequence_num,
        amount,
        expiration_timestamp_secs,
        net,
    ))
}

pub fn build_transfer_txn(
    sender: AccountAddress,
    receiver: AccountAddress,
    receiver_public_key_vec: Vec<u8>,
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
        receiver_public_key_vec,
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
    receiver_public_key_vec: Vec<u8>,
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
        receiver_public_key_vec,
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
    recipient_public_key_vec: Vec<u8>,
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
            chain_id.net().unwrap().stdlib_version(),
            receiver,
            recipient_public_key_vec,
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
            compiled_transaction_script(
                chain_id.net().unwrap().stdlib_version(),
                StdlibScript::AcceptToken,
            )
            .into_vec(),
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
    version: StdlibVersion,
    token_type: TypeTag,
    account_address: &AccountAddress,
    public_key_vec: Vec<u8>,
    initial_balance: u128,
) -> Script {
    Script::new(
        compiled_transaction_script(version, StdlibScript::CreateAccount).into_vec(),
        vec![token_type],
        vec![
            TransactionArgument::Address(*account_address),
            TransactionArgument::U8Vector(public_key_vec),
            TransactionArgument::U128(initial_balance),
        ],
    )
}

pub fn encode_transfer_script(
    version: StdlibVersion,
    recipient: AccountAddress,
    recipient_public_key_vec: Vec<u8>,
    amount: u128,
) -> Script {
    encode_transfer_script_by_token_code(
        version,
        recipient,
        recipient_public_key_vec,
        amount,
        STC_TOKEN_CODE.clone(),
    )
}

pub fn encode_transfer_script_by_token_code(
    version: StdlibVersion,
    recipient: AccountAddress,
    recipient_public_key_vec: Vec<u8>,
    amount: u128,
    token_code: TokenCode,
) -> Script {
    Script::new(
        compiled_transaction_script(version, StdlibScript::PeerToPeer).into_vec(),
        vec![token_code.into()],
        vec![
            TransactionArgument::Address(recipient),
            TransactionArgument::U8Vector(recipient_public_key_vec),
            TransactionArgument::U128(amount),
        ],
    )
}

pub fn peer_to_peer_txn_sent_as_association(
    recipient: AccountAddress,
    recipient_public_key_vec: Vec<u8>,
    seq_num: u64,
    amount: u128,
    expiration_timestamp_secs: u64,
    net: &ChainNetwork,
) -> SignedUserTransaction {
    crate::create_signed_txn_with_association_account(
        TransactionPayload::Script(encode_transfer_script(
            net.stdlib_version(),
            recipient,
            recipient_public_key_vec,
            amount,
        )),
        seq_num,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        expiration_timestamp_secs,
        net,
    )
}

//this only work for DEV,
pub fn create_signed_txn_with_association_account(
    payload: TransactionPayload,
    sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    expiration_timestamp_secs: u64,
    net: &ChainNetwork,
) -> SignedUserTransaction {
    let raw_txn = RawUserTransaction::new(
        account_config::association_address(),
        sequence_number,
        payload,
        max_gas_amount,
        gas_unit_price,
        expiration_timestamp_secs,
        net.chain_id(),
    );
    net.genesis_config()
        .sign_with_association(raw_txn)
        .expect("Sign txn should work.")
}

pub fn build_stdlib_package(
    net: &ChainNetwork,
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
        let genesis_config = net.genesis_config();
        let chain_id = net.chain_id().id();
        let genesis_timestamp = net.genesis_config().timestamp;

        let genesis_auth_key = genesis_config
            .genesis_key_pair
            .as_ref()
            .map(|(_, public_key)| AuthenticationKey::ed25519(&public_key).to_vec())
            .unwrap_or_else(Vec::new);

        let association_auth_key =
            AuthenticationKey::ed25519(&genesis_config.association_key_pair.1).to_vec();

        // for test
        // let initial_script_allow_list =
        //     VersionedStdlibScript::new(net.stdlib_version()).whitelist();
        let initial_script_allow_list = genesis_config.publishing_option.allowed_script();

        let mut merged_script_allow_list: Vec<u8> = Vec::new();
        for i in 0..initial_script_allow_list.len() {
            let tmp = &mut initial_script_allow_list
                .get(i)
                .expect("Cannot get script allow list member")
                .to_vec();
            merged_script_allow_list.append(tmp);
        }

        let instruction_schedule =
            scs::to_bytes(&genesis_config.vm_config.gas_schedule.instruction_table)
                .expect("Cannot serialize gas schedule");
        let native_schedule = scs::to_bytes(&genesis_config.vm_config.gas_schedule.native_table)
            .expect("Cannot serialize gas schedule");

        package.set_init_script(Script::new(
            compiled_init_script(net.stdlib_version(), InitScript::GenesisInit).into_vec(),
            vec![],
            vec![
                TransactionArgument::U64(genesis_config.reward_delay),
                TransactionArgument::U128(genesis_config.pre_mine_amount),
                TransactionArgument::U128(genesis_config.time_mint_amount),
                TransactionArgument::U64(genesis_config.time_mint_period),
                TransactionArgument::U8Vector(genesis_config.parent_hash.to_vec()),
                TransactionArgument::U8Vector(association_auth_key),
                TransactionArgument::U8Vector(genesis_auth_key),
                TransactionArgument::U8(chain_id),
                TransactionArgument::U64(genesis_timestamp),
                //consensus config
                TransactionArgument::U64(genesis_config.consensus_config.uncle_rate_target),
                TransactionArgument::U64(genesis_config.consensus_config.epoch_block_count),
                TransactionArgument::U64(genesis_config.consensus_config.base_block_time_target),
                TransactionArgument::U64(
                    genesis_config.consensus_config.base_block_difficulty_window,
                ),
                TransactionArgument::U128(genesis_config.consensus_config.base_reward_per_block),
                TransactionArgument::U64(
                    genesis_config
                        .consensus_config
                        .base_reward_per_uncle_percent,
                ),
                TransactionArgument::U64(genesis_config.consensus_config.min_block_time_target),
                TransactionArgument::U64(genesis_config.consensus_config.max_block_time_target),
                TransactionArgument::U64(genesis_config.consensus_config.base_max_uncles_per_block),
                TransactionArgument::U64(genesis_config.consensus_config.base_block_gas_limit),
                TransactionArgument::U8(genesis_config.consensus_config.strategy),
                //vm config
                TransactionArgument::U8Vector(merged_script_allow_list),
                TransactionArgument::Bool(genesis_config.publishing_option.is_open()),
                TransactionArgument::U8Vector(instruction_schedule),
                TransactionArgument::U8Vector(native_schedule),
                //gas constants
                TransactionArgument::U64(
                    genesis_config
                        .gas_constants
                        .global_memory_per_byte_cost
                        .get(),
                ),
                TransactionArgument::U64(
                    genesis_config
                        .gas_constants
                        .global_memory_per_byte_write_cost
                        .get(),
                ),
                TransactionArgument::U64(
                    genesis_config.gas_constants.min_transaction_gas_units.get(),
                ),
                TransactionArgument::U64(
                    genesis_config.gas_constants.large_transaction_cutoff.get(),
                ),
                TransactionArgument::U64(genesis_config.gas_constants.intrinsic_gas_per_byte.get()),
                TransactionArgument::U64(
                    genesis_config
                        .gas_constants
                        .maximum_number_of_gas_units
                        .get(),
                ),
                TransactionArgument::U64(genesis_config.gas_constants.min_price_per_gas_unit.get()),
                TransactionArgument::U64(genesis_config.gas_constants.max_price_per_gas_unit.get()),
                TransactionArgument::U64(
                    genesis_config.gas_constants.max_transaction_size_in_bytes,
                ),
                TransactionArgument::U64(genesis_config.gas_constants.gas_unit_scaling_factor),
                TransactionArgument::U64(genesis_config.gas_constants.default_account_size.get()),
                // dao config params
                TransactionArgument::U64(genesis_config.dao_config.voting_delay),
                TransactionArgument::U64(genesis_config.dao_config.voting_period),
                TransactionArgument::U8(genesis_config.dao_config.voting_quorum_rate),
                TransactionArgument::U64(genesis_config.dao_config.min_action_delay),
                //transaction timeout config
                TransactionArgument::U64(genesis_config.transaction_timeout),
            ],
        ));
    }
    Ok(package)
}
