// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::data_cache::StateViewCache;
use crate::genesis_context::{GenesisContext, GenesisStateView};
use crate::genesis_gas_schedule::INITIAL_GAS_SCHEDULE;
use anyhow::Result;
use bytecode_verifier::VerifiedModule;
use crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    PrivateKey, Uniform,
};
use once_cell::sync::Lazy;
use rand::{rngs::StdRng, SeedableRng};
use starcoin_config::ChainConfig;
use starcoin_types::{
    contract_event::ContractEvent, transaction::authenticator::AuthenticationKey,
};
use starcoin_vm_types::account_config::config_address;
use starcoin_vm_types::{
    account_config,
    data_store::DataStore,
    language_storage::{StructTag, TypeTag},
    loaded_data::types::FatStructType,
    on_chain_config::VMPublishingOption,
    transaction::ChangeSet,
    values::Value,
    write_set::WriteSet,
};
use std::collections::BTreeMap;
use stdlib::{stdlib_modules, StdLibOptions};
use vm::access::ModuleAccess;

const GENESIS_SEED: [u8; 32] = [42; 32];

/// The initial balance of the association account.
pub const ASSOCIATION_INIT_BALANCE: u64 = 1_000_000_000_000_000;
pub const SUBSIDY_BALANCE: u64 = ASSOCIATION_INIT_BALANCE / 2;

pub static GENESIS_KEYPAIR: Lazy<(Ed25519PrivateKey, Ed25519PublicKey)> = Lazy::new(|| {
    let mut rng = StdRng::from_seed(GENESIS_SEED);
    let private_key = Ed25519PrivateKey::generate(&mut rng);
    let public_key = private_key.public_key();
    (private_key, public_key)
});

static INITIALIZE_NAME: &str = "initialize";
static MINT_TO_ADDRESS: &str = "mint_to_address";

static REWARD_INIT: &str = "initialize_reward_info";

const GENESIS_MODULE_NAME: &str = "Genesis";

pub fn generate_genesis_state_set(chain_config: &ChainConfig) -> Result<ChangeSet> {
    let modules = stdlib_modules(StdLibOptions::Staged);
    let (write_set, events, _) = encode_genesis_write_set(chain_config, modules);
    Ok(ChangeSet::new(write_set, events))
}

pub fn encode_genesis_write_set(
    chain_config: &ChainConfig,
    stdlib_modules: &[VerifiedModule],
) -> (
    WriteSet,
    Vec<ContractEvent>,
    BTreeMap<Vec<u8>, FatStructType>,
) {
    // create a data view for move_vm
    let mut state_view = GenesisStateView::new();
    for module in stdlib_modules {
        let module_id = module.self_id();
        state_view.add_module(&module_id, &module);
    }
    let data_cache = StateViewCache::new(&state_view);

    let mut genesis_context = GenesisContext::new(&data_cache, stdlib_modules);

    let stc_ty = TypeTag::Struct(StructTag {
        address: *account_config::STC_MODULE.address(),
        module: account_config::STC_MODULE.name().to_owned(),
        name: account_config::STC_STRUCT_NAME.to_owned(),
        type_params: vec![],
    });

    // generate the genesis WriteSet
    create_and_initialize_main_accounts(&mut genesis_context, chain_config, &stc_ty);
    setup_vm_config(&mut genesis_context);
    reconfigure(&mut genesis_context);

    let mut data_store = genesis_context.into_data_store();
    publish_stdlib(&mut data_store, stdlib_modules);

    //verify_genesis_write_set(interpreter_context.events());
    (
        data_store
            .make_write_set()
            .expect("Genesis WriteSet failure"),
        data_store.events().to_vec(),
        data_store.get_type_map(),
    )
}

/// Create an initialize Association, Transaction Fee and Core Code accounts.
fn create_and_initialize_main_accounts(
    context: &mut GenesisContext,
    chain_config: &ChainConfig,
    stc_ty: &TypeTag,
) {
    let mut miner_reward_balance = chain_config.total_supply;

    let genesis_auth_key = chain_config
        .pre_mine_config
        .as_ref()
        .map(|pre_mine_config| AuthenticationKey::ed25519(&pre_mine_config.public_key).to_vec())
        .unwrap_or_else(|| vec![0u8; AuthenticationKey::LENGTH]);

    let root_association_address = account_config::association_address();
    // create the mint account
    let mint_address: starcoin_vm_types::account_address::AccountAddress =
        account_config::mint_address();

    context.set_sender(root_association_address);
    context.exec(
        GENESIS_MODULE_NAME,
        "initialize",
        vec![],
        vec![
            Value::transaction_argument_signer_reference(root_association_address),
            Value::transaction_argument_signer_reference(account_config::config_address()),
            Value::transaction_argument_signer_reference(account_config::transaction_fee_address()),
            Value::transaction_argument_signer_reference(mint_address),
            Value::vector_u8(genesis_auth_key.to_vec()),
        ],
    );

    // init reward config
    context.set_sender(config_address());
    context.exec(
        account_config::REWARD_CONF_MODULE_NAME,
        INITIALIZE_NAME,
        vec![],
        vec![
            Value::transaction_argument_signer_reference(config_address()),
            Value::u64(chain_config.reward_halving_interval),
            Value::u64(chain_config.base_block_reward),
            Value::u64(chain_config.reward_delay),
        ],
    );

    context.set_sender(mint_address);
    context.exec(
        account_config::BLOCK_MODULE_NAME,
        REWARD_INIT,
        vec![],
        vec![Value::transaction_argument_signer_reference(mint_address)],
    );

    if let Some(pre_mine_config) = &chain_config.pre_mine_config {
        context.set_sender(root_association_address);

        let association_balance =
            chain_config.total_supply * pre_mine_config.pre_mine_percent / 100;
        miner_reward_balance -= association_balance;
        context.exec(
            account_config::ACCOUNT_MODULE_NAME,
            MINT_TO_ADDRESS,
            vec![stc_ty.clone()],
            vec![
                Value::transaction_argument_signer_reference(root_association_address),
                Value::address(root_association_address),
                Value::u64(association_balance),
            ],
        );
    }

    // mint coins to mint address
    context.set_sender(root_association_address);
    context.exec(
        account_config::ACCOUNT_MODULE_NAME,
        MINT_TO_ADDRESS,
        vec![stc_ty.clone()],
        vec![
            Value::transaction_argument_signer_reference(root_association_address),
            Value::address(mint_address),
            Value::u64(miner_reward_balance),
        ],
    );

    context.set_sender(root_association_address);
    // Bump the sequence number for the Association account. If we don't do this and a
    // subsequent transaction (e.g., minting) is sent from the Assocation account, a problem
    // arises: both the genesis transaction and the subsequent transaction have sequence
    // number 0
    context.exec(
        "Account",
        "epilogue",
        vec![stc_ty.clone()],
        vec![
            Value::transaction_argument_signer_reference(root_association_address),
            Value::u64(/* txn_sequence_number */ 0),
            Value::u64(/* txn_gas_price */ 0),
            Value::u64(/* txn_max_gas_units */ 0),
            Value::u64(/* gas_units_remaining */ 0),
        ],
    );
}

fn setup_vm_config(context: &mut GenesisContext) {
    let publishing_option = VMPublishingOption::Open;
    context.set_sender(account_config::config_address());

    let option_bytes =
        scs::to_bytes(&publishing_option).expect("Cannot serialize publishing option");
    context.exec(
        "VMConfig",
        "initialize",
        vec![],
        vec![
            Value::transaction_argument_signer_reference(account_config::config_address()),
            Value::vector_u8(option_bytes),
            Value::vector_u8(INITIAL_GAS_SCHEDULE.0.clone()),
            Value::vector_u8(INITIAL_GAS_SCHEDULE.1.clone()),
        ],
    );
}

fn remove_genesis(stdlib_modules: &[VerifiedModule]) -> impl Iterator<Item = &VerifiedModule> {
    stdlib_modules
        .iter()
        .filter(|module| module.self_id().name().as_str() != GENESIS_MODULE_NAME)
}

/// Publish the standard library.
fn publish_stdlib(data_store: &mut dyn DataStore, stdlib: &[VerifiedModule]) {
    for module in remove_genesis(stdlib) {
        assert_ne!(module.self_id().name().as_str(), GENESIS_MODULE_NAME);
        let mut module_vec = vec![];
        module.serialize(&mut module_vec).unwrap();
        data_store
            .publish_module(module.self_id(), module_vec)
            .unwrap_or_else(|_| panic!("Failure publishing module {:?}", module.self_id()));
    }
}

/// Trigger a reconfiguration. This emits an event that will be passed along to the storage layer.
fn reconfigure(context: &mut GenesisContext) {
    context.set_sender(account_config::association_address());
    context.exec("Config", "emit_reconfiguration_event", vec![], vec![]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_config::ChainNetwork;

    #[test]
    fn test_genesis() {
        let change_set = generate_genesis_state_set(ChainNetwork::Dev.get_config()).unwrap();
        assert!(!change_set.write_set().is_empty())
    }
}
