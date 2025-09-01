// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::convert::TryInto;

use anyhow::Result;
use starcoin_crypto::{hash::PlainCryptoHash, HashValue};

use starcoin_cached_packages::starcoin_framework_sdk_builder::{
    empty_scripts_empty_script, on_chain_config_scripts_propose_update_vm_config,
    stc_genesis_initialize, transfer_scripts_batch_peer_to_peer_v2, transfer_scripts_peer_to_peer,
    transfer_scripts_peer_to_peer_v2,
};

use starcoin_cached_packages::starcoin_stdlib::{
    dao_queue_proposal_action, dao_upgrade_module_proposal_propose_module_upgrade_v2,
    dao_upgrade_module_proposal_submit_module_upgrade_plan,
};
use starcoin_crypto::_once_cell::sync::Lazy;
use starcoin_vm2_types::account::Account;
use starcoin_vm2_vm_types::genesis_config::{ChainId, GenesisConfig};
use starcoin_vm2_vm_types::{
    access::ModuleAccess,
    account_address::AccountAddress,
    account_config::{self, core_code_address, genesis_address, STCUnit},
    file_format::CompiledModule,
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
    on_chain_config::{Features, VMConfig},
    on_chain_resource::nft::NFTUUID,
    token::{
        stc::{stc_type_tag, G_STC_TOKEN_CODE},
        token_code::TokenCode,
        token_value::TokenValue,
    },
    transaction::{
        authenticator::{AccountPrivateKey, AuthenticationKey},
        EntryFunction, Module, Package, RawUserTransaction, SignedUserTransaction, Transaction,
        TransactionPayload,
    },
};

pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;
pub const DEFAULT_MAX_GAS_AMOUNT: u64 = 40000000;
pub static G_TOTAL_STC_AMOUNT: Lazy<TokenValue<STCUnit>> =
    Lazy::new(|| STCUnit::STC.value_of(3185136000));

pub fn build_transfer_from_association(
    addr: AccountAddress,
    association_sequence_num: u64,
    amount: u128,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
    genesis_config: &GenesisConfig,
) -> Transaction {
    Transaction::UserTransaction(peer_to_peer_txn_sent_as_association(
        addr,
        association_sequence_num,
        amount,
        expiration_timestamp_secs,
        chain_id,
        genesis_config,
    ))
}

pub fn build_transfer_txn(
    sender: AccountAddress,
    receiver: AccountAddress,
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
        seq_num,
        amount,
        gas_price,
        max_gas,
        G_STC_TOKEN_CODE.clone(),
        expiration_timestamp_secs,
        chain_id,
    )
}

pub fn build_batch_payload(
    receivers: Vec<AccountAddress>,
    amounts: Vec<u128>,
) -> TransactionPayload {
    transfer_scripts_batch_peer_to_peer_v2(stc_type_tag(), receivers, amounts)
}

pub fn build_batch_payload_same_amount(
    receivers: Vec<AccountAddress>,
    amount: u128,
) -> TransactionPayload {
    let len = receivers.len();
    build_batch_payload(receivers, (0..len).map(|_| amount).collect())
}

pub fn build_batch_transfer_txn(
    sender: AccountAddress,
    receivers: Vec<AccountAddress>,
    seq_num: u64,
    amount: u128,
    gas_price: u64,
    max_gas: u64,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> RawUserTransaction {
    let payload = build_batch_payload_same_amount(receivers, amount);

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
    transfer_amount: u128,
    seq_num: u64,
    gas_price: u64,
    max_gas: u64,
    token_code: TokenCode,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> RawUserTransaction {
    let token_type_tag = TypeTag::Struct(Box::new(token_code.try_into().unwrap()));
    RawUserTransaction::new_with_default_gas_token(
        sender,
        seq_num,
        transfer_scripts_peer_to_peer(token_type_tag, receiver, vec![], transfer_amount),
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
    let payload = TransactionPayload::EntryFunction(EntryFunction::new(
        ModuleId::new(core_code_address(), Identifier::new("account").unwrap()),
        Identifier::new("accept_token").unwrap(),
        vec![TypeTag::Struct(Box::new(token_code.try_into().unwrap()))],
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
    _version: u64,
    token_type: TypeTag,
    account_address: &AccountAddress,
    auth_key: AuthenticationKey,
    initial_balance: u128,
) -> EntryFunction {
    EntryFunction::new(
        ModuleId::new(core_code_address(), Identifier::new("account").unwrap()),
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
    recipient: AccountAddress,
    amount: u128,
) -> TransactionPayload {
    encode_transfer_script_by_token_code(recipient, amount, G_STC_TOKEN_CODE.clone())
}

pub fn encode_transfer_script_by_token_code(
    recipient: AccountAddress,
    amount: u128,
    token_code: TokenCode,
) -> TransactionPayload {
    let token_type_tag = TypeTag::Struct(Box::new(token_code.try_into().unwrap()));
    transfer_scripts_peer_to_peer_v2(token_type_tag, recipient, amount)
}

pub fn encode_nft_transfer_script(uuid: NFTUUID, recipient: AccountAddress) -> EntryFunction {
    EntryFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("NFTGalleryScripts").unwrap(),
        ),
        Identifier::new("transfer").unwrap(),
        vec![uuid.nft_type.meta_type, uuid.nft_type.body_type],
        vec![
            bcs_ext::to_bytes(&uuid.id).unwrap(),
            bcs_ext::to_bytes(&recipient).unwrap(),
        ],
    )
}

pub fn peer_to_peer_txn_sent_as_association(
    recipient: AccountAddress,
    seq_num: u64,
    amount: u128,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
    genesis_config: &GenesisConfig,
) -> SignedUserTransaction {
    create_signed_txn_with_association_account(
        transfer_scripts_peer_to_peer(stc_type_tag(), recipient, vec![], amount),
        seq_num,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        expiration_timestamp_secs,
        chain_id,
        genesis_config,
    )
}

pub fn peer_to_peer_v2(
    sender: &Account,
    recipient: &AccountAddress,
    seq_num: u64,
    amount: u128,
    chain_id: ChainId,
) -> SignedUserTransaction {
    // It's ok to unwrap here, because we know the script exists in the stdlib.
    sender
        .sign_txn(RawUserTransaction::new_with_default_gas_token(
            *sender.address(),
            seq_num,
            transfer_scripts_batch_peer_to_peer_v2(stc_type_tag(), vec![*recipient], vec![amount]),
            10000000,
            1,
            1000 + 60 * 60,
            chain_id,
        ))
        .unwrap()
}

//this only work for DEV or TEST
pub fn create_signed_txn_with_association_account(
    payload: TransactionPayload,
    sequence_number: u64,
    max_gas_amount: u64,
    gas_unit_price: u64,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
    genesis_config: &GenesisConfig,
) -> SignedUserTransaction {
    let raw_txn = RawUserTransaction::new_with_default_gas_token(
        account_config::association_address(),
        sequence_number,
        payload,
        max_gas_amount,
        gas_unit_price,
        expiration_timestamp_secs,
        chain_id,
    );
    genesis_config
        .sign_with_association(raw_txn)
        .expect("Sign txn should work.")
}

// fixme: enable stdlib_option
pub fn build_stdlib_package(
    chain_id: ChainId,
    genesis_config: &GenesisConfig,
    _stdlib_option: Option<u64>,
) -> Result<Package> {
    let init_script = build_init_script(chain_id.id(), genesis_config);
    let modules = starcoin_cached_packages::head_release_bundle().legacy_copy_code();
    Package::new(
        modules.into_iter().map(Module::new).collect(),
        Some(init_script),
    )
}

pub fn build_stdlib_package_with_modules(
    chain_id: ChainId,
    genesis_config: &GenesisConfig,
    modules: Vec<Vec<u8>>,
) -> Result<Package> {
    let init_script = build_init_script(chain_id.id(), genesis_config);
    Package::new(
        modules.into_iter().map(Module::new).collect(),
        Some(init_script),
    )
}

fn build_init_script(chain_id: u8, genesis_config: &GenesisConfig) -> EntryFunction {
    let genesis_timestamp = genesis_config.genesis_block_parameter().unwrap().timestamp;
    let genesis_parent_hash = genesis_config
        .genesis_block_parameter()
        .unwrap()
        .parent_hash;

    let genesis_auth_key = genesis_config
        .genesis_key_pair
        .as_ref()
        .map(|(_, public_key)| AuthenticationKey::ed25519(public_key).to_vec())
        .unwrap_or_default();

    let association_auth_key =
        AuthenticationKey::multi_ed25519(&genesis_config.association_key_pair.1).to_vec();

    let payload = stc_genesis_initialize(
        genesis_config.stdlib_version.version(),
        genesis_config.reward_delay,
        G_TOTAL_STC_AMOUNT.scaling(),
        genesis_config.pre_mine_amount,
        genesis_config.time_mint_amount,
        genesis_config.time_mint_period,
        genesis_parent_hash.to_vec(),
        association_auth_key,
        genesis_auth_key,
        chain_id,
        genesis_timestamp,
        //consensus config
        genesis_config.consensus_config.uncle_rate_target,
        genesis_config.consensus_config.epoch_block_count,
        genesis_config.consensus_config.base_block_time_target,
        genesis_config.consensus_config.base_block_difficulty_window,
        genesis_config.consensus_config.base_reward_per_block,
        genesis_config
            .consensus_config
            .base_reward_per_uncle_percent,
        genesis_config.consensus_config.min_block_time_target,
        genesis_config.consensus_config.max_block_time_target,
        genesis_config.consensus_config.base_max_uncles_per_block,
        genesis_config.consensus_config.base_block_gas_limit,
        genesis_config.consensus_config.strategy,
        genesis_config.consensus_config.max_transaction_per_block,
        genesis_config.consensus_config.pruning_depth,
        genesis_config.consensus_config.pruning_finality,
        //vm config
        genesis_config.publishing_option.is_script_allowed(),
        genesis_config
            .publishing_option
            .is_module_publishing_allowed(),
        bcs_ext::to_bytes(&genesis_config.vm_config.gas_schedule).unwrap(),
        // dao config params
        genesis_config.dao_config.voting_delay,
        genesis_config.dao_config.voting_period,
        genesis_config.dao_config.voting_quorum_rate,
        genesis_config.dao_config.min_action_delay,
        //transaction timeout config
        genesis_config.transaction_timeout,
        Features::default().features,
    );

    match payload {
        TransactionPayload::EntryFunction(e) => e,
        _ => panic!("Expected EntryFunction payload"),
    }
}

pub fn build_package_with_stdlib_module(
    _stdlib_option: Option<u64>,
    module_names: Vec<&str>,
    init_script: Option<EntryFunction>,
) -> Result<Package> {
    let modules = starcoin_cached_packages::head_release_bundle().legacy_copy_code();
    let mut package = Package::new_with_modules(
        modules
            .iter()
            .cloned()
            .filter_map(|blob| {
                let m = CompiledModule::deserialize(&blob).expect("serializing stdlib must work");
                let handle = &m.module_handles()[0];
                let name = m.identifier_at(handle.name).as_str();
                let mut found = false;
                for module in module_names.iter() {
                    if name == *module {
                        found = true;
                    }
                }
                if found {
                    Some(Module::new(blob))
                } else {
                    None
                }
            })
            .collect(),
    )?;
    if let Some(script_function) = init_script {
        package.set_init_script(script_function);
    }
    Ok(package)
}

pub fn build_stdlib_package_for_test(
    _stdlib_option: Option<u64>,
    init_script: Option<EntryFunction>,
) -> Result<Package> {
    let modules = starcoin_cached_packages::head_release_bundle().legacy_copy_code();
    Package::new(modules.into_iter().map(Module::new).collect(), init_script)
}

pub fn build_module_upgrade_proposal(
    package: &Package,
    version: u64,
    exec_delay: u64,
    enforced: bool,
    token_code: TokenCode,
) -> Result<(TransactionPayload, HashValue)> {
    let package_hash = package.crypto_hash();
    Ok((
        dao_upgrade_module_proposal_propose_module_upgrade_v2(
            token_code.try_into()?,
            package.package_address(),
            package_hash.clone().to_vec(),
            version,
            exec_delay,
            enforced,
        ),
        package_hash,
    ))
}

pub fn build_module_upgrade_plan(
    proposer_address: AccountAddress,
    proposal_id: u64,
    token_code: TokenCode,
) -> Result<TransactionPayload> {
    Ok(dao_upgrade_module_proposal_submit_module_upgrade_plan(
        token_code.try_into()?,
        proposer_address,
        proposal_id,
    ))
}

pub fn build_module_upgrade_queue(
    proposal_address: AccountAddress,
    proposal_id: u64,
    token_code: TokenCode,
) -> Result<TransactionPayload> {
    let action_type_tag = TypeTag::Struct(Box::new(StructTag {
        address: genesis_address(),
        module: Identifier::new("dao_upgrade_module_proposal").unwrap(),
        name: Identifier::new("UpgradeModuleV2").unwrap(),
        type_args: vec![],
    }));
    Ok(dao_queue_proposal_action(
        token_code.try_into()?,
        action_type_tag,
        proposal_address,
        proposal_id,
    ))
}

pub fn build_vm_config_upgrade_proposal(
    vm_config: VMConfig,
    exec_delay: u64,
) -> Result<TransactionPayload> {
    Ok(on_chain_config_scripts_propose_update_vm_config(
        bcs_ext::to_bytes(&vm_config.gas_schedule)?,
        exec_delay,
    ))
}

pub fn empty_txn_payload() -> TransactionPayload {
    empty_scripts_empty_script()
}

pub fn build_signed_empty_txn(
    user_address: AccountAddress,
    prikey: &AccountPrivateKey,
    seq_num: u64,
    expiration_timestamp_secs: u64,
    chain_id: ChainId,
) -> SignedUserTransaction {
    let txn = RawUserTransaction::new_with_default_gas_token(
        user_address,
        seq_num,
        empty_txn_payload(),
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        expiration_timestamp_secs,
        chain_id,
    );
    // It's ok to unwrap here, signing an empty txn should never fail.
    let signature = prikey.sign(&txn).unwrap();
    SignedUserTransaction::new(txn, signature)
}
