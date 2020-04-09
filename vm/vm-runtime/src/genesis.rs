// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::gas::initial_gas_schedule;
use crate::{chain_state::StateStore, system_module_names::*};
use anyhow::Result;
use bytecode_verifier::VerifiedModule;
use crypto::ed25519::*;
use crypto::HashValue;
use crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    PrivateKey, Uniform, ValidKey,
};
use libra_state_view::StateView;
use libra_types::access_path::AccessPath;
use logger::prelude::*;
use move_core_types::identifier::Identifier;
use move_vm_runtime::MoveVM;
use move_vm_state::{
    data_cache::BlockDataCache,
    execution_context::{ExecutionContext, TransactionExecutionContext},
};
use move_vm_types::{chain_state::ChainState as LibraChainState, values::Value};
use once_cell::sync::Lazy;
use rand::{rngs::StdRng, SeedableRng};
use starcoin_state_api::ChainState;
use stdlib::{stdlib_modules, StdLibOptions};
use types::transaction::authenticator::AuthenticationKey;
use types::{account_config, state_set::ChainStateSet};
use vm::{
    access::ModuleAccess,
    gas_schedule::{CostTable, GasAlgebra, GasUnits},
    transaction_metadata::TransactionMetadata,
};

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

static INITIALIZE: Lazy<Identifier> = Lazy::new(|| Identifier::new("initialize").unwrap());
static INITIALIZE_BLOCK: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("initialize_block_metadata").unwrap());
static MINT_TO_ADDRESS: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("mint_to_address").unwrap());
static EPILOGUE: Lazy<Identifier> = Lazy::new(|| Identifier::new("epilogue").unwrap());
static ROTATE_AUTHENTICATION_KEY: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("rotate_authentication_key").unwrap());

static SUBSIDY_CONF: Lazy<Identifier> = Lazy::new(|| Identifier::new("subsidy").unwrap());
static SUBSIDY_INIT: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("initialize_subsidy_info").unwrap());

pub fn generate_genesis_state_set(
    _private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    chain_state: &dyn ChainState,
) -> Result<(HashValue, ChainStateSet)> {
    // Compile the needed stdlib modules.
    let modules = stdlib_modules(StdLibOptions::Staged);

    // create a MoveVM
    let move_vm = MoveVM::new();

    // create a data view for move_vm
    let state_view = GenesisStateView;
    let gas_schedule = CostTable::zero();
    let data_cache = BlockDataCache::new(&state_view);

    // create an execution context for the move_vm.
    // It will contain the genesis WriteSet after execution
    let mut interpreter_context =
        TransactionExecutionContext::new(GasUnits::new(100_000_000), &data_cache);

    // initialize the VM with stdlib modules.
    // This step is needed because we are creating the main accounts and we are calling
    // code to create those. However, code lives under an account but we have none.
    // So we are pushing code into the VM blindly in order to create the main accounts.
    for module in modules {
        move_vm.cache_module(module.clone());
    }

    let mut state_store = StateStore::new(chain_state);

    create_and_initialize_main_accounts(
        &move_vm,
        &gas_schedule,
        &mut interpreter_context,
        &public_key,
        initial_gas_schedule(&move_vm, &data_cache),
    );
    publish_stdlib(&mut interpreter_context, modules);

    let write_set = interpreter_context.make_write_set()?;
    state_store.add_write_set(&write_set);
    state_store.commit()?;
    state_store.flush()?;

    let dump = state_store.state().dump()?;
    Ok((state_store.state().state_root(), dump))
}

/// Create and initialize Transaction Fee and Core Code accounts.
fn create_and_initialize_main_accounts(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
    public_key: &Ed25519PublicKey,
    initial_gas_schedule: Value,
) {
    let association_addr: libra_types::account_address::AccountAddress =
        account_config::association_address().into();
    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = association_addr;
    // create  the LBR module
    move_vm
        .execute_function(
            &LBR_MODULE,
            &INITIALIZE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure initializing LBR");
    // create the association account
    move_vm
        .execute_function(
            &ACCOUNT_MODULE,
            &CREATE_ACCOUNT_NAME,
            gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![
                Value::address(association_addr),
                Value::vector_u8(association_addr.to_vec()),
            ],
        )
        .unwrap_or_else(|e| {
            panic!(
                "Failure creating association account {:?}: {}",
                association_addr, e
            )
        });

    // create the transaction fee account
    let transaction_fee_address: libra_types::account_address::AccountAddress =
        account_config::transaction_fee_address().into();
    move_vm
        .execute_function(
            &ACCOUNT_MODULE,
            &CREATE_ACCOUNT_NAME,
            gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![
                Value::address(transaction_fee_address),
                Value::vector_u8(transaction_fee_address.to_vec()),
            ],
        )
        .unwrap_or_else(|e| {
            panic!(
                "Failure creating transaction fee account {:?}: {}",
                transaction_fee_address, e
            )
        });

    // create the mint account
    let mint_address: libra_types::account_address::AccountAddress =
        account_config::mint_address().into();
    move_vm
        .execute_function(
            &ACCOUNT_MODULE,
            &CREATE_ACCOUNT_NAME,
            gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![
                Value::address(mint_address),
                Value::vector_u8(mint_address.to_vec()),
            ],
        )
        .unwrap_or_else(|e| panic!("Failure creating mint account {:?}: {}", mint_address, e));

    // init subsidy config
    txn_data.sender = mint_address;
    move_vm
        .execute_function(
            &SUBSIDY_CONF_MODULE,
            &INITIALIZE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure initializing SubsidyConfig");

    move_vm
        .execute_function(
            &SUBSIDY_CONF_MODULE,
            &SUBSIDY_CONF,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![
                Value::u64(1000 as u64),
                Value::u64(50_000_000 as u64),
                Value::u64(2 as u64),
            ],
        )
        .unwrap();
    txn_data.sender = association_addr;

    move_vm
        .execute_function(
            &LIBRA_TRANSACTION_TIMEOUT,
            &INITIALIZE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure initializing LibraTransactionTimeout");

    move_vm
        .execute_function(
            &LIBRA_BLOCK_MODULE,
            &INITIALIZE_BLOCK,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![],
        )
        .expect("Failure initializing block metadata");

    move_vm
        .execute_function(
            &GAS_SCHEDULE_MODULE,
            &INITIALIZE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![initial_gas_schedule],
        )
        .expect("Failure initializing gas module");

    move_vm
        .execute_function(
            &ACCOUNT_MODULE,
            &MINT_TO_ADDRESS,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![
                Value::address(association_addr),
                Value::vector_u8(association_addr.to_vec()),
                Value::u64(ASSOCIATION_INIT_BALANCE),
            ],
        )
        .expect("Failure minting to association");

    // mint coins to mint address
    move_vm
        .execute_function(
            &ACCOUNT_MODULE,
            &MINT_TO_ADDRESS,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![
                Value::address(mint_address),
                Value::vector_u8(mint_address.to_vec()),
                Value::u64(SUBSIDY_BALANCE),
            ],
        )
        .unwrap();

    //    // init subsidy.
    //    txn_data.sender = mint_address;
    //    move_vm
    //        .execute_function(
    //            &LIBRA_BLOCK_MODULE,
    //            &SUBSIDY_INIT,
    //            &gas_schedule,
    //            interpreter_context,
    //            &txn_data,
    //            vec![],
    //            vec![],
    //        )
    //        .unwrap();
    //    txn_data.sender = association_addr;

    let genesis_auth_key = AuthenticationKey::ed25519(public_key).to_vec();
    move_vm
        .execute_function(
            &ACCOUNT_MODULE,
            &ROTATE_AUTHENTICATION_KEY,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![Value::vector_u8(genesis_auth_key)],
        )
        .expect("Failure rotating association key");

    // Bump the sequence number for the Association account. If we don't do this and a
    // subsequent transaction (e.g., minting) is sent from the Assocation account, a problem
    // arises: both the genesis transaction and the subsequent transaction have sequence
    // number 0
    move_vm
        .execute_function(
            &ACCOUNT_MODULE,
            &EPILOGUE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
            vec![
                Value::u64(/* txn_sequence_number */ 0),
                Value::u64(/* txn_gas_price */ 0),
                Value::u64(/* txn_max_gas_units */ 0),
                Value::u64(/* gas_units_remaining */ 0),
            ],
        )
        .expect("Failure running epilogue for association account");
}

/// Publish the standard library.
fn publish_stdlib(interpreter_context: &mut dyn LibraChainState, stdlib: &[VerifiedModule]) {
    for module in stdlib {
        let mut module_vec = vec![];
        module.serialize(&mut module_vec).unwrap();
        interpreter_context
            .publish_module(module.self_id(), module_vec)
            .unwrap_or_else(|_| panic!("Failure publishing module {:?}", module.self_id()));
    }
}

// `StateView` has no data given we are creating the genesis
struct GenesisStateView;

impl StateView for GenesisStateView {
    fn get(&self, _access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        Ok(None)
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        true
    }
}
