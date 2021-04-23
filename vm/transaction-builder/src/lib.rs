// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_config::ChainNetwork;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_vm_types::access::ModuleAccess;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config;
use starcoin_vm_types::account_config::{core_code_address, genesis_address};
use starcoin_vm_types::gas_schedule::GasAlgebra;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::language_storage::{StructTag, TypeTag};
use starcoin_vm_types::token::stc::{stc_type_tag, STC_TOKEN_CODE};
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::transaction::authenticator::AuthenticationKey;
use starcoin_vm_types::transaction::{
    Module, Package, RawUserTransaction, ScriptFunction, SignedUserTransaction, Transaction,
    TransactionPayload,
};
pub use stdlib::{stdlib_modules, StdLibOptions, StdlibVersion};

pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;
pub const DEFAULT_MAX_GAS_AMOUNT: u64 = 20000000;

pub fn build_transfer_from_association(
    addr: AccountAddress,
    recipient_auth_key: Option<AuthenticationKey>,
    association_sequence_num: u64,
    amount: u128,
    expiration_timestamp_secs: u64,
    net: &ChainNetwork,
) -> Transaction {
    Transaction::UserTransaction(peer_to_peer_txn_sent_as_association(
        addr,
        recipient_auth_key,
        association_sequence_num,
        amount,
        expiration_timestamp_secs,
        net,
    ))
}

pub fn build_transfer_txn(
    sender: AccountAddress,
    receiver: AccountAddress,
    recipient_auth_key: Option<AuthenticationKey>,
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
        recipient_auth_key,
        seq_num,
        amount,
        gas_price,
        max_gas,
        STC_TOKEN_CODE.clone(),
        expiration_timestamp_secs,
        chain_id,
    )
}

pub fn build_batch_transfer_txn(
    sender: AccountAddress,
    receivers: Vec<AccountAddress>,
    recipient_auth_keys: Vec<AuthenticationKey>,
    seq_num: u64,
    amount: u128,
    gas_price: u64,
    max_gas: u64,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> RawUserTransaction {
    let mut address_vec = vec![];
    for receiver in receivers {
        address_vec.extend_from_slice(receiver.to_vec().as_slice());
    }
    let mut auth_key_vec = vec![];
    for auth_key in recipient_auth_keys {
        auth_key_vec.extend_from_slice(auth_key.to_vec().as_slice());
    }

    let payload = TransactionPayload::ScriptFunction(ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("TransferScripts").unwrap(),
        ),
        Identifier::new("peer_to_peer_batch").unwrap(),
        vec![stc_type_tag()],
        vec![
            bcs_ext::to_bytes(&address_vec).unwrap(),
            bcs_ext::to_bytes(&auth_key_vec).unwrap(),
            bcs_ext::to_bytes(&amount).unwrap(),
        ],
    ));

    RawUserTransaction::new_with_default_gas_token(
        sender,
        seq_num,
        payload,
        max_gas,
        gas_price,
        expiration_timestamp_secs,
        chain_id,
    )
}

pub fn build_transfer_txn_by_token_type(
    sender: AccountAddress,
    receiver: AccountAddress,
    recipient_auth_key: Option<AuthenticationKey>,
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
        recipient_auth_key,
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
    recipient_auth_key: Option<AuthenticationKey>,
    transfer_amount: u128,
    seq_num: u64,
    gas_price: u64,
    max_gas: u64,
    token_code: TokenCode,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> RawUserTransaction {
    RawUserTransaction::new_with_default_gas_token(
        sender,
        seq_num,
        TransactionPayload::ScriptFunction(encode_transfer_script_by_token_code(
            //TODO should use latest?
            StdlibVersion::Latest,
            receiver,
            recipient_auth_key,
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
    let payload = TransactionPayload::ScriptFunction(ScriptFunction::new(
        ModuleId::new(core_code_address(), Identifier::new("Account").unwrap()),
        Identifier::new("accept_token").unwrap(),
        vec![token_code.into()],
        vec![],
    ));

    RawUserTransaction::new_with_default_gas_token(
        sender,
        seq_num,
        payload,
        max_gas,
        gas_price,
        expiration_timestamp_secs,
        chain_id,
    )
}

pub fn encode_create_account_script_function(
    _version: StdlibVersion,
    token_type: TypeTag,
    account_address: &AccountAddress,
    auth_key: AuthenticationKey,
    initial_balance: u128,
) -> ScriptFunction {
    ScriptFunction::new(
        ModuleId::new(core_code_address(), Identifier::new("Account").unwrap()),
        Identifier::new("create_account_with_initial_amount").unwrap(),
        vec![token_type],
        vec![
            bcs_ext::to_bytes(account_address).unwrap(),
            bcs_ext::to_bytes(&auth_key.to_vec()).unwrap(),
            bcs_ext::to_bytes(&initial_balance).unwrap(),
        ],
    )
}

pub fn encode_transfer_script_function(
    version: StdlibVersion,
    recipient: AccountAddress,
    recipient_auth_key: Option<AuthenticationKey>,
    amount: u128,
) -> ScriptFunction {
    encode_transfer_script_by_token_code(
        version,
        recipient,
        recipient_auth_key,
        amount,
        STC_TOKEN_CODE.clone(),
    )
}

pub fn encode_transfer_script_by_token_code(
    _version: StdlibVersion,
    recipient: AccountAddress,
    recipient_auth_key: Option<AuthenticationKey>,
    amount: u128,
    token_code: TokenCode,
) -> ScriptFunction {
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("TransferScripts").unwrap(),
        ),
        Identifier::new("peer_to_peer").unwrap(),
        vec![token_code.into()],
        vec![
            bcs_ext::to_bytes(&recipient).unwrap(),
            bcs_ext::to_bytes(&recipient_auth_key.map(|k| k.to_vec()).unwrap_or_default()).unwrap(),
            bcs_ext::to_bytes(&amount).unwrap(),
        ],
    )
}

pub fn peer_to_peer_txn_sent_as_association(
    recipient: AccountAddress,
    recipient_auth_key: Option<AuthenticationKey>,
    seq_num: u64,
    amount: u128,
    expiration_timestamp_secs: u64,
    net: &ChainNetwork,
) -> SignedUserTransaction {
    crate::create_signed_txn_with_association_account(
        TransactionPayload::ScriptFunction(encode_transfer_script_function(
            net.stdlib_version(),
            recipient,
            recipient_auth_key,
            amount,
        )),
        seq_num,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        expiration_timestamp_secs,
        net,
    )
}

//this only work for DEV or TEST
pub fn create_signed_txn_with_association_account(
    payload: TransactionPayload,
    sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    expiration_timestamp_secs: u64,
    net: &ChainNetwork,
) -> SignedUserTransaction {
    let raw_txn = RawUserTransaction::new_with_default_gas_token(
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
        let genesis_timestamp = net.genesis_block_parameter().timestamp;
        let genesis_parent_hash = net.genesis_block_parameter().parent_hash;

        let genesis_auth_key = genesis_config
            .genesis_key_pair
            .as_ref()
            .map(|(_, public_key)| AuthenticationKey::ed25519(&public_key).to_vec())
            .unwrap_or_else(Vec::new);

        let association_auth_key =
            AuthenticationKey::multi_ed25519(&genesis_config.association_key_pair.1).to_vec();

        let instruction_schedule =
            bcs_ext::to_bytes(&genesis_config.vm_config.gas_schedule.instruction_table)
                .expect("Cannot serialize gas schedule");
        let native_schedule =
            bcs_ext::to_bytes(&genesis_config.vm_config.gas_schedule.native_table)
                .expect("Cannot serialize gas schedule");
        let init_script = ScriptFunction::new(
            ModuleId::new(core_code_address(), Identifier::new("Genesis").unwrap()),
            Identifier::new("initialize").unwrap(),
            vec![],
            vec![
                bcs_ext::to_bytes(&net.stdlib_version().version()).unwrap(),
                bcs_ext::to_bytes(&genesis_config.reward_delay).unwrap(),
                bcs_ext::to_bytes(&genesis_config.pre_mine_amount).unwrap(),
                bcs_ext::to_bytes(&genesis_config.time_mint_amount).unwrap(),
                bcs_ext::to_bytes(&genesis_config.time_mint_period).unwrap(),
                bcs_ext::to_bytes(&genesis_parent_hash.to_vec()).unwrap(),
                bcs_ext::to_bytes(&association_auth_key).unwrap(),
                bcs_ext::to_bytes(&genesis_auth_key).unwrap(),
                bcs_ext::to_bytes(&chain_id).unwrap(),
                bcs_ext::to_bytes(&genesis_timestamp).unwrap(),
                //consensus config
                bcs_ext::to_bytes(&genesis_config.consensus_config.uncle_rate_target).unwrap(),
                bcs_ext::to_bytes(&genesis_config.consensus_config.epoch_block_count).unwrap(),
                bcs_ext::to_bytes(&genesis_config.consensus_config.base_block_time_target).unwrap(),
                bcs_ext::to_bytes(&genesis_config.consensus_config.base_block_difficulty_window)
                    .unwrap(),
                bcs_ext::to_bytes(&genesis_config.consensus_config.base_reward_per_block).unwrap(),
                bcs_ext::to_bytes(
                    &genesis_config
                        .consensus_config
                        .base_reward_per_uncle_percent,
                )
                .unwrap(),
                bcs_ext::to_bytes(&genesis_config.consensus_config.min_block_time_target).unwrap(),
                bcs_ext::to_bytes(&genesis_config.consensus_config.max_block_time_target).unwrap(),
                bcs_ext::to_bytes(&genesis_config.consensus_config.base_max_uncles_per_block)
                    .unwrap(),
                bcs_ext::to_bytes(&genesis_config.consensus_config.base_block_gas_limit).unwrap(),
                bcs_ext::to_bytes(&genesis_config.consensus_config.strategy).unwrap(),
                //vm config
                bcs_ext::to_bytes(&genesis_config.publishing_option.is_script_allowed()).unwrap(),
                bcs_ext::to_bytes(
                    &genesis_config
                        .publishing_option
                        .is_module_publishing_allowed(),
                )
                .unwrap(),
                bcs_ext::to_bytes(&instruction_schedule).unwrap(),
                bcs_ext::to_bytes(&native_schedule).unwrap(),
                //gas constants
                bcs_ext::to_bytes(
                    &genesis_config
                        .vm_config
                        .gas_schedule
                        .gas_constants
                        .global_memory_per_byte_cost
                        .get(),
                )
                .unwrap(),
                bcs_ext::to_bytes(
                    &genesis_config
                        .vm_config
                        .gas_schedule
                        .gas_constants
                        .global_memory_per_byte_write_cost
                        .get(),
                )
                .unwrap(),
                bcs_ext::to_bytes(
                    &genesis_config
                        .vm_config
                        .gas_schedule
                        .gas_constants
                        .min_transaction_gas_units
                        .get(),
                )
                .unwrap(),
                bcs_ext::to_bytes(
                    &genesis_config
                        .vm_config
                        .gas_schedule
                        .gas_constants
                        .large_transaction_cutoff
                        .get(),
                )
                .unwrap(),
                bcs_ext::to_bytes(
                    &genesis_config
                        .vm_config
                        .gas_schedule
                        .gas_constants
                        .intrinsic_gas_per_byte
                        .get(),
                )
                .unwrap(),
                bcs_ext::to_bytes(
                    &genesis_config
                        .vm_config
                        .gas_schedule
                        .gas_constants
                        .maximum_number_of_gas_units
                        .get(),
                )
                .unwrap(),
                bcs_ext::to_bytes(
                    &genesis_config
                        .vm_config
                        .gas_schedule
                        .gas_constants
                        .min_price_per_gas_unit
                        .get(),
                )
                .unwrap(),
                bcs_ext::to_bytes(
                    &genesis_config
                        .vm_config
                        .gas_schedule
                        .gas_constants
                        .max_price_per_gas_unit
                        .get(),
                )
                .unwrap(),
                bcs_ext::to_bytes(
                    &genesis_config
                        .vm_config
                        .gas_schedule
                        .gas_constants
                        .max_transaction_size_in_bytes,
                )
                .unwrap(),
                bcs_ext::to_bytes(
                    &genesis_config
                        .vm_config
                        .gas_schedule
                        .gas_constants
                        .gas_unit_scaling_factor,
                )
                .unwrap(),
                bcs_ext::to_bytes(
                    &genesis_config
                        .vm_config
                        .gas_schedule
                        .gas_constants
                        .default_account_size
                        .get(),
                )
                .unwrap(),
                // dao config params
                bcs_ext::to_bytes(&genesis_config.dao_config.voting_delay).unwrap(),
                bcs_ext::to_bytes(&genesis_config.dao_config.voting_period).unwrap(),
                bcs_ext::to_bytes(&genesis_config.dao_config.voting_quorum_rate).unwrap(),
                bcs_ext::to_bytes(&genesis_config.dao_config.min_action_delay).unwrap(),
                //transaction timeout config
                bcs_ext::to_bytes(&genesis_config.transaction_timeout).unwrap(),
            ],
        );
        package.set_init_script(init_script);
    }
    Ok(package)
}

pub fn build_module_upgrade_proposal(
    package: &Package,
    version: u64,
    day: u64,
) -> (ScriptFunction, HashValue) {
    let package_hash = package.crypto_hash();
    (
        ScriptFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("ModuleUpgradeScripts").unwrap(),
            ),
            Identifier::new("propose_module_upgrade").unwrap(),
            vec![stc_type_tag()],
            vec![
                bcs_ext::to_bytes(&package.package_address()).unwrap(),
                bcs_ext::to_bytes(&package_hash.clone().to_vec()).unwrap(),
                bcs_ext::to_bytes(&version).unwrap(),
                bcs_ext::to_bytes(&day).unwrap(),
            ],
        ),
        package_hash,
    )
}

pub fn build_module_upgrade_plan(
    proposer_address: AccountAddress,
    proposal_id: u64,
) -> ScriptFunction {
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("ModuleUpgradeScripts").unwrap(),
        ),
        Identifier::new("submit_module_upgrade_plan").unwrap(),
        vec![stc_type_tag()],
        vec![
            bcs_ext::to_bytes(&proposer_address).unwrap(),
            bcs_ext::to_bytes(&proposal_id).unwrap(),
        ],
    )
}

pub fn build_module_upgrade_queue(
    proposal_address: AccountAddress,
    proposal_id: u64,
) -> ScriptFunction {
    let upgrade_module = TypeTag::Struct(StructTag {
        address: genesis_address(),
        module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
        name: Identifier::new("UpgradeModule").unwrap(),
        type_params: vec![],
    });

    ScriptFunction::new(
        ModuleId::new(core_code_address(), Identifier::new("Dao").unwrap()),
        Identifier::new("queue_proposal_action").unwrap(),
        vec![stc_type_tag(), upgrade_module],
        vec![
            bcs_ext::to_bytes(&proposal_address).unwrap(),
            bcs_ext::to_bytes(&proposal_id).unwrap(),
        ],
    )
}

pub fn build_empty_script() -> ScriptFunction {
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("EmptyScripts").unwrap(),
        ),
        Identifier::new("empty_script").unwrap(),
        vec![],
        vec![],
    )
}
