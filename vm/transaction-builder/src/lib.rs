// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use starcoin_config::{genesis_config::TOTAL_STC_AMOUNT, ChainNetwork};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_vm_types::access::ModuleAccess;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config;
use starcoin_vm_types::account_config::{core_code_address, genesis_address};
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::gas_schedule::GasAlgebra;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::language_storage::{StructTag, TypeTag};
use starcoin_vm_types::on_chain_config::VMConfig;
use starcoin_vm_types::on_chain_resource::nft::NFTUUID;
use starcoin_vm_types::token::stc::{stc_type_tag, STC_TOKEN_CODE};
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::transaction::authenticator::{AccountPrivateKey, AuthenticationKey};
use starcoin_vm_types::transaction::{
    Module, Package, RawUserTransaction, ScriptFunction, SignedUserTransaction, Transaction,
    TransactionPayload,
};
use starcoin_vm_types::value::MoveValue;
use std::convert::TryInto;
use stdlib::stdlib_package;
pub use stdlib::{stdlib_modules, StdLibOptions, StdlibVersion};

pub const DEFAULT_EXPIRATION_TIME: u64 = 40_000;
pub const DEFAULT_MAX_GAS_AMOUNT: u64 = 40000000;

pub fn build_transfer_from_association(
    addr: AccountAddress,
    association_sequence_num: u64,
    amount: u128,
    expiration_timestamp_secs: u64,
    net: &ChainNetwork,
) -> Transaction {
    Transaction::UserTransaction(peer_to_peer_txn_sent_as_association(
        addr,
        association_sequence_num,
        amount,
        expiration_timestamp_secs,
        net,
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
        STC_TOKEN_CODE.clone(),
        expiration_timestamp_secs,
        chain_id,
    )
}

pub fn build_batch_script_function(
    receivers: Vec<AccountAddress>,
    amounts: Vec<u128>,
) -> ScriptFunction {
    let addresses = MoveValue::vector_address(receivers);
    let amounts = MoveValue::Vector(amounts.into_iter().map(MoveValue::U128).collect());
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("TransferScripts").unwrap(),
        ),
        Identifier::new("batch_peer_to_peer_v2").unwrap(),
        vec![stc_type_tag()],
        vec![
            bcs_ext::to_bytes(&addresses).unwrap(),
            bcs_ext::to_bytes(&amounts).unwrap(),
        ],
    )
}

pub fn build_batch_script_function_same_amount(
    receivers: Vec<AccountAddress>,
    amount: u128,
) -> ScriptFunction {
    let len = receivers.len();
    build_batch_script_function(receivers, (0..len).map(|_| amount).collect())
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
    let payload = TransactionPayload::ScriptFunction(build_batch_script_function_same_amount(
        receivers, amount,
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
    RawUserTransaction::new_with_default_gas_token(
        sender,
        seq_num,
        TransactionPayload::ScriptFunction(encode_transfer_script_by_token_code(
            receiver,
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
        vec![TypeTag::Struct(token_code.try_into().unwrap())],
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

pub fn encode_transfer_script_function(recipient: AccountAddress, amount: u128) -> ScriptFunction {
    encode_transfer_script_by_token_code(recipient, amount, STC_TOKEN_CODE.clone())
}

pub fn encode_transfer_script_by_token_code(
    recipient: AccountAddress,
    amount: u128,
    token_code: TokenCode,
) -> ScriptFunction {
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("TransferScripts").unwrap(),
        ),
        Identifier::new("peer_to_peer_v2").unwrap(),
        vec![TypeTag::Struct(token_code.try_into().unwrap())],
        vec![
            bcs_ext::to_bytes(&recipient).unwrap(),
            bcs_ext::to_bytes(&amount).unwrap(),
        ],
    )
}

pub fn encode_nft_transfer_script(uuid: NFTUUID, recipient: AccountAddress) -> ScriptFunction {
    ScriptFunction::new(
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
    net: &ChainNetwork,
) -> SignedUserTransaction {
    crate::create_signed_txn_with_association_account(
        TransactionPayload::ScriptFunction(encode_transfer_script_function(recipient, amount)),
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

pub fn build_stdlib_package(net: &ChainNetwork, stdlib_option: StdLibOptions) -> Result<Package> {
    let init_script = match net.genesis_config().stdlib_version {
        StdlibVersion::Version(1) => build_init_script_v1(net),
        _ => build_init_script_v2(net),
    };
    stdlib_package(stdlib_option, Some(init_script))
}

pub fn build_init_script_v1(net: &ChainNetwork) -> ScriptFunction {
    let genesis_config = net.genesis_config();
    let chain_id = net.chain_id().id();
    let genesis_timestamp = net.genesis_block_parameter().timestamp;
    let genesis_parent_hash = net.genesis_block_parameter().parent_hash;

    let genesis_auth_key = genesis_config
        .genesis_key_pair
        .as_ref()
        .map(|(_, public_key)| AuthenticationKey::ed25519(public_key).to_vec())
        .unwrap_or_else(Vec::new);

    let association_auth_key =
        AuthenticationKey::multi_ed25519(&genesis_config.association_key_pair.1).to_vec();

    let instruction_schedule =
        bcs_ext::to_bytes(&genesis_config.vm_config.gas_schedule.instruction_table)
            .expect("Cannot serialize gas schedule");
    let native_schedule = bcs_ext::to_bytes(&genesis_config.vm_config.gas_schedule.native_table)
        .expect("Cannot serialize gas schedule");
    ScriptFunction::new(
        ModuleId::new(core_code_address(), Identifier::new("Genesis").unwrap()),
        Identifier::new("initialize").unwrap(),
        vec![],
        vec![
            bcs_ext::to_bytes(&net.genesis_config().stdlib_version.version()).unwrap(),
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
            bcs_ext::to_bytes(&genesis_config.consensus_config.base_max_uncles_per_block).unwrap(),
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
    )
}

pub fn build_init_script_v2(net: &ChainNetwork) -> ScriptFunction {
    let genesis_config = net.genesis_config();
    let chain_id = net.chain_id().id();
    let genesis_timestamp = net.genesis_block_parameter().timestamp;
    let genesis_parent_hash = net.genesis_block_parameter().parent_hash;

    let genesis_auth_key = genesis_config
        .genesis_key_pair
        .as_ref()
        .map(|(_, public_key)| AuthenticationKey::ed25519(public_key).to_vec())
        .unwrap_or_else(Vec::new);

    let association_auth_key =
        AuthenticationKey::multi_ed25519(&genesis_config.association_key_pair.1).to_vec();

    let instruction_schedule =
        bcs_ext::to_bytes(&genesis_config.vm_config.gas_schedule.instruction_table)
            .expect("Cannot serialize gas schedule");
    let native_schedule = bcs_ext::to_bytes(&genesis_config.vm_config.gas_schedule.native_table)
        .expect("Cannot serialize gas schedule");
    ScriptFunction::new(
        ModuleId::new(core_code_address(), Identifier::new("Genesis").unwrap()),
        Identifier::new("initialize_v2").unwrap(),
        vec![],
        vec![
            bcs_ext::to_bytes(&net.genesis_config().stdlib_version.version()).unwrap(),
            bcs_ext::to_bytes(&genesis_config.reward_delay).unwrap(),
            bcs_ext::to_bytes(&TOTAL_STC_AMOUNT.scaling()).unwrap(),
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
            bcs_ext::to_bytes(&genesis_config.consensus_config.base_max_uncles_per_block).unwrap(),
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
    )
}

pub fn build_package_with_stdlib_module(
    stdlib_option: StdLibOptions,
    module_names: Vec<&str>,
    init_script: Option<ScriptFunction>,
) -> Result<Package> {
    let modules = stdlib_modules(stdlib_option);
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
    stdlib_option: StdLibOptions,
    init_script: Option<ScriptFunction>,
) -> Result<Package> {
    stdlib_package(stdlib_option, init_script)
}

pub fn build_module_upgrade_proposal(
    package: &Package,
    version: u64,
    exec_delay: u64,
    enforced: bool,
    token_code: TokenCode,
    stdlib_version: StdlibVersion,
) -> (ScriptFunction, HashValue) {
    let package_hash = package.crypto_hash();
    // propose_module_upgrade_v2 is available after v2 upgrade.
    let (function_name, args) = if stdlib_version >= StdlibVersion::Version(2) {
        (
            "propose_module_upgrade_v2",
            vec![
                bcs_ext::to_bytes(&package.package_address()).unwrap(),
                bcs_ext::to_bytes(&package_hash.clone().to_vec()).unwrap(),
                bcs_ext::to_bytes(&version).unwrap(),
                bcs_ext::to_bytes(&exec_delay).unwrap(),
                bcs_ext::to_bytes(&enforced).unwrap(),
            ],
        )
    } else {
        (
            "propose_module_upgrade",
            vec![
                bcs_ext::to_bytes(&package.package_address()).unwrap(),
                bcs_ext::to_bytes(&package_hash.to_vec()).unwrap(),
                bcs_ext::to_bytes(&version).unwrap(),
                bcs_ext::to_bytes(&exec_delay).unwrap(),
            ],
        )
    };

    (
        ScriptFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("ModuleUpgradeScripts").unwrap(),
            ),
            Identifier::new(function_name).unwrap(),
            vec![token_code
                .try_into()
                .expect("Token code to type tag should success")],
            args,
        ),
        package_hash,
    )
}

pub fn build_module_upgrade_plan(
    proposer_address: AccountAddress,
    proposal_id: u64,
    token_code: TokenCode,
) -> ScriptFunction {
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("ModuleUpgradeScripts").unwrap(),
        ),
        Identifier::new("submit_module_upgrade_plan").unwrap(),
        vec![token_code
            .try_into()
            .expect("Token code to type tag should success")],
        vec![
            bcs_ext::to_bytes(&proposer_address).unwrap(),
            bcs_ext::to_bytes(&proposal_id).unwrap(),
        ],
    )
}

pub fn build_module_upgrade_queue(
    proposal_address: AccountAddress,
    proposal_id: u64,
    token_code: TokenCode,
    stdlib_version: StdlibVersion,
) -> ScriptFunction {
    let upgrade_module = if stdlib_version >= StdlibVersion::Version(2) {
        TypeTag::Struct(StructTag {
            address: genesis_address(),
            module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
            name: Identifier::new("UpgradeModuleV2").unwrap(),
            type_params: vec![],
        })
    } else {
        TypeTag::Struct(StructTag {
            address: genesis_address(),
            module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
            name: Identifier::new("UpgradeModule").unwrap(),
            type_params: vec![],
        })
    };

    ScriptFunction::new(
        ModuleId::new(core_code_address(), Identifier::new("Dao").unwrap()),
        Identifier::new("queue_proposal_action").unwrap(),
        vec![
            token_code
                .try_into()
                .expect("Token code to type tag should success"),
            upgrade_module,
        ],
        vec![
            bcs_ext::to_bytes(&proposal_address).unwrap(),
            bcs_ext::to_bytes(&proposal_id).unwrap(),
        ],
    )
}

pub fn build_vm_config_upgrade_proposal(vm_config: VMConfig, exec_delay: u64) -> ScriptFunction {
    let gas_constants = &vm_config.gas_schedule.gas_constants;
    ScriptFunction::new(
        ModuleId::new(
            core_code_address(),
            Identifier::new("OnChainConfigScripts").unwrap(),
        ),
        Identifier::new("propose_update_vm_config").unwrap(),
        vec![],
        vec![
            bcs_ext::to_bytes(
                &bcs_ext::to_bytes(&vm_config.gas_schedule.instruction_table).unwrap(),
            )
            .unwrap(),
            bcs_ext::to_bytes(&bcs_ext::to_bytes(&vm_config.gas_schedule.native_table).unwrap())
                .unwrap(),
            bcs_ext::to_bytes(&gas_constants.global_memory_per_byte_cost.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.global_memory_per_byte_write_cost.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.min_transaction_gas_units.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.large_transaction_cutoff.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.intrinsic_gas_per_byte.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.maximum_number_of_gas_units.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.min_price_per_gas_unit.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.max_price_per_gas_unit.get()).unwrap(),
            bcs_ext::to_bytes(&gas_constants.max_transaction_size_in_bytes).unwrap(),
            bcs_ext::to_bytes(&gas_constants.gas_unit_scaling_factor).unwrap(),
            bcs_ext::to_bytes(&gas_constants.default_account_size.get()).unwrap(),
            bcs_ext::to_bytes(&exec_delay).unwrap(),
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

fn empty_txn_payload() -> TransactionPayload {
    TransactionPayload::ScriptFunction(build_empty_script())
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
    let signature = prikey.sign(&txn);
    SignedUserTransaction::new(txn, signature)
}
