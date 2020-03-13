// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use crate::genesis_gas_schedule::initial_gas_schedule;
use crate::transaction_helper::TransactionHelper;
use stdlib::{stdlib_modules, StdLibOptions};
use once_cell::sync::Lazy;
use bytecode_verifier::VerifiedModule;
use crypto::HashValue;
use crypto::{
    ed25519::*,
};
use libra_types::{
    access_path::AccessPath,
    transaction::{ChangeSet, RawTransaction,},
    byte_array::ByteArray,
    account_address::AccountAddress,

};
use move_core_types::identifier::Identifier;
use move_vm_runtime::MoveVM;
use move_vm_state::{
    data_cache::BlockDataCache,
    execution_context::{ExecutionContext, TransactionExecutionContext},
};
use types::{
    transaction::{RawUserTransaction, SignatureCheckedTransaction},
    state_set::ChainStateSet,
    account_config,
};
use vm::{
    access::ModuleAccess,
    gas_schedule::{CostTable, GasAlgebra, GasUnits},
    transaction_metadata::TransactionMetadata,
};
use crate::{
    system_module_names::*, chain_state::StateStore,
};
use libra_state_view::StateView;
use move_vm_types::{chain_state::ChainState as LibraChainState, values::Value};
use rand::{rngs::StdRng, SeedableRng};
use traits::ChainState;

//use std::str::FromStr;

const GENESIS_SEED: [u8; 32] = [42; 32];

/// The initial balance of the association account.
pub const ASSOCIATION_INIT_BALANCE: u64 = 1_000_000_000_000_000;

pub static GENESIS_KEYPAIR: Lazy<(Ed25519PrivateKey, Ed25519PublicKey)> = Lazy::new(|| {
    let mut rng = StdRng::from_seed(GENESIS_SEED);
    compat::generate_keypair(&mut rng)
});

static INITIALIZE: Lazy<Identifier> = Lazy::new(|| Identifier::new("initialize").unwrap());
static INITIALIZE_BLOCK: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("initialize_block_metadata").unwrap());
static MINT_TO_ADDRESS: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("mint_to_address").unwrap());
static EPILOGUE: Lazy<Identifier> = Lazy::new(|| Identifier::new("epilogue").unwrap());
static ROTATE_AUTHENTICATION_KEY: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("rotate_authentication_key").unwrap());

pub fn generate_genesis_state_set(
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    chain_state: &dyn ChainState,
) -> Result<(HashValue, ChainStateSet)> {
    // Compile the needed stdlib modules.
    let modules = stdlib_modules(StdLibOptions::Staged);

    // create a MoveVM
    let mut move_vm = MoveVM::new();

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

    create_and_initialize_main_accounts(
        &move_vm,
        &gas_schedule,
        &mut interpreter_context,
        &public_key,
        initial_gas_schedule(&move_vm, &data_cache),
    );
    publish_stdlib(&mut interpreter_context, modules);

    let write_set = interpreter_context.make_write_set()?;
    let mut state_store = StateStore::new(chain_state);
    state_store.add_write_set(&write_set);

    Ok((state_store.state().state_root(), state_store.state().dump()?))
}


/// Create and initialize Transaction Fee and Core Code accounts.
fn create_and_initialize_main_accounts(
    move_vm: &MoveVM,
    gas_schedule: &CostTable,
    interpreter_context: &mut TransactionExecutionContext,
    public_key: &Ed25519PublicKey,
    initial_gas_schedule: Value,
) {
    let association_addr = TransactionHelper::to_libra_AccountAddress(account_config::association_address());
    let mut txn_data = TransactionMetadata::default();
    txn_data.sender = association_addr;

    // create the association account
    move_vm
        .execute_function(
            &ACCOUNT_MODULE,
            &CREATE_ACCOUNT_NAME,
            gas_schedule,
            interpreter_context,
            &txn_data,
            vec![Value::address(association_addr)],
        )
        .unwrap_or_else(|_| {
            panic!(
                "Failure creating association account {:?}",
                association_addr
            )
        });

    move_vm
        .execute_function(
            &COIN_MODULE,
            &INITIALIZE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
        )
        .expect("Failure initializing LibraCoin");

    move_vm
        .execute_function(
            &LIBRA_TRANSACTION_TIMEOUT,
            &INITIALIZE,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![],
        )
        .expect("Failure initializing LibraTransactionTimeout");

    move_vm
        .execute_function(
            &LIBRA_SYSTEM_MODULE,
            &INITIALIZE_BLOCK,
            &gas_schedule,
            interpreter_context,
            &txn_data,
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
            vec![
                Value::address(association_addr),
                Value::u64(ASSOCIATION_INIT_BALANCE),
            ],
        )
        .expect("Failure minting to association");

    let genesis_auth_key = ByteArray::new(AccountAddress::from_public_key(public_key).to_vec());
    move_vm
        .execute_function(
            &ACCOUNT_MODULE,
            &ROTATE_AUTHENTICATION_KEY,
            &gas_schedule,
            interpreter_context,
            &txn_data,
            vec![Value::byte_array(genesis_auth_key)],
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