// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Account;
use crate::Genesis;
use anyhow::{bail, Result};
use serde::de::DeserializeOwned;
use starcoin_account_api::AccountPrivateKey;
use starcoin_config::ChainNetwork;
use starcoin_executor::{execute_readonly_function, execute_transactions, DEFAULT_MAX_GAS_AMOUNT};
use starcoin_state_api::{ChainState, StateReaderExt, StateView};
use starcoin_statedb::{ChainStateDB, ChainStateWriter};
use starcoin_types::account_config::{association_address, genesis_address};
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_types::transaction::{
    RawUserTransaction, SignedUserTransaction, TransactionOutput, TransactionPayload,
};
use starcoin_types::{
    account_address::AccountAddress, transaction::Module, transaction::Transaction,
    transaction::TransactionStatus,
};
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::vm_status::KeptVMStatus;
use stdlib::stdlib_files;

//TODO warp to A MockTxnExecutor

pub const TEST_MODULE: &str = r#"
    module {{sender}}::M {
        struct Foo { a: u8 }
        public fun foo(): u8 { 1 }
    }
    "#;
pub const TEST_MODULE_1: &str = r#"
    module {{sender}}::M {
        struct Foo { a: address }
        public fun foo(): u8 { 2 }
    }
    "#;
pub const TEST_MODULE_2: &str = r#"
    module {{sender}}::M {
        struct Foo { a: u8 }
        public fun foo(): u8 { 1 }
        public fun bar(): u8 { 2 }
    }
    "#;

pub fn prepare_genesis() -> (ChainStateDB, ChainNetwork) {
    let net = ChainNetwork::new_test();
    let chain_state = ChainStateDB::mock();
    let genesis_txn = Genesis::build_genesis_transaction(&net).unwrap();
    Genesis::execute_genesis_txn(&chain_state, genesis_txn).unwrap();
    (chain_state, net)
}

pub fn prepare_customized_genesis(net: &ChainNetwork) -> ChainStateDB {
    let chain_state = ChainStateDB::mock();
    let genesis_txn = Genesis::build_genesis_transaction(net).unwrap();
    Genesis::execute_genesis_txn(&chain_state, genesis_txn).unwrap();
    chain_state
}

pub fn execute_and_apply(chain_state: &ChainStateDB, txn: Transaction) -> TransactionOutput {
    let output = execute_transactions(chain_state, vec![txn], None)
        .unwrap()
        .pop()
        .expect("Output must exist.");
    if let TransactionStatus::Keep(_) = output.status() {
        chain_state
            .apply_write_set(output.write_set().clone())
            .expect("apply write_set should success.");
    }

    output
}
pub fn current_block_number(state_view: &dyn StateView) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Block").unwrap()),
        &Identifier::new("get_current_block_number").unwrap(),
        vec![],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

pub fn get_sequence_number(addr: AccountAddress, chain_state: &dyn ChainState) -> u64 {
    chain_state
        .get_account_resource(addr)
        .expect("read account state should ok")
        .map(|res| res.sequence_number())
        .unwrap_or_default()
}

pub fn get_balance(address: AccountAddress, chain_state: &dyn ChainState) -> u128 {
    chain_state
        .get_balance(address)
        .expect("read balance resource should ok")
        .unwrap_or_default()
}

pub fn compile_modules_with_address(address: AccountAddress, code: &str) -> Vec<Module> {
    let (_, compiled_result) =
        starcoin_move_compiler::compile_source_string(code, &stdlib_files(), address)
            .expect("compile fail");

    compiled_result
        .into_iter()
        .map(|m| Module::new(m.serialize()))
        .collect()
}
#[allow(unused)]
pub fn compile_script(code: impl AsRef<str>) -> Vec<u8> {
    let mut compile_unit = starcoin_move_compiler::compile_source_string_no_report(
        code.as_ref(),
        &stdlib_files(),
        genesis_address(),
    )
    .expect("compile fail")
    .1
    .expect("compile fail");
    compile_unit
        .0
        .pop()
        .expect("at least contain one script")
        .into_compiled_unit()
        .serialize()
}

pub fn association_execute(
    net: &ChainNetwork,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<TransactionOutput> {
    let txn = build_raw_txn(association_address(), state, payload, None);
    let txn = net.genesis_config().sign_with_association(txn)?;
    execute_signed_txn(state, txn)
}

pub fn association_execute_should_success(
    net: &ChainNetwork,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<TransactionOutput> {
    let txn = build_raw_txn(association_address(), state, payload, None);
    let txn = net.genesis_config().sign_with_association(txn)?;
    execute_signed_txn_should_success(state, txn)
}

pub fn account_execute(
    account: &Account,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<TransactionOutput> {
    user_execute(*account.address(), account.private_key(), state, payload)
}
pub fn account_execute_should_success(
    account: &Account,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<TransactionOutput> {
    user_execute_should_success(*account.address(), account.private_key(), state, payload)
}
pub fn account_execute_with_output(
    account: &Account,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> TransactionOutput {
    let txn = build_signed_txn(*account.address(), account.private_key(), state, payload);
    execute_and_apply(state, Transaction::UserTransaction(txn))
}

pub fn blockmeta_execute(state: &ChainStateDB, meta: BlockMetadata) -> Result<TransactionOutput> {
    let txn = Transaction::BlockMetadata(meta);
    let output = execute_and_apply(state, txn);
    if let TransactionStatus::Discard(s) = output.status() {
        bail!("txn discard, status: {:?}", s);
    }

    Ok(output)
}

pub fn build_raw_txn(
    user_address: AccountAddress,
    state: &ChainStateDB,
    payload: TransactionPayload,
    expiration_timestamp_secs: Option<u64>,
) -> RawUserTransaction {
    let chain_id = state.get_chain_id().unwrap();
    let seq_number = get_sequence_number(user_address, state);

    let now_seconds: u64 = state.get_timestamp().unwrap().milliseconds / 1000;
    let expiration_timestamp_secs = expiration_timestamp_secs.unwrap_or(now_seconds + 60 * 60);
    RawUserTransaction::new_with_default_gas_token(
        user_address,
        seq_number,
        payload,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        expiration_timestamp_secs,
        chain_id,
    )
}

fn user_execute(
    user_address: AccountAddress,
    prikey: &AccountPrivateKey,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<TransactionOutput> {
    let txn = build_signed_txn(user_address, prikey, state, payload);
    execute_signed_txn(state, txn)
}

fn user_execute_should_success(
    user_address: AccountAddress,
    prikey: &AccountPrivateKey,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<TransactionOutput> {
    let txn = build_signed_txn(user_address, prikey, state, payload);
    execute_signed_txn_should_success(state, txn)
}

fn build_signed_txn(
    user_address: AccountAddress,
    prikey: &AccountPrivateKey,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> SignedUserTransaction {
    let txn = build_raw_txn(user_address, state, payload, None);
    let signature = prikey.sign(&txn);
    SignedUserTransaction::new(txn, signature)
}

#[allow(clippy::unnecessary_wraps)]
fn execute_signed_txn(
    state: &ChainStateDB,
    txn: SignedUserTransaction,
) -> Result<TransactionOutput> {
    let txn = Transaction::UserTransaction(txn);
    Ok(execute_and_apply(state, txn))
}

fn execute_signed_txn_should_success(
    state: &ChainStateDB,
    txn: SignedUserTransaction,
) -> Result<TransactionOutput> {
    let txn = Transaction::UserTransaction(txn);
    let output = execute_and_apply(state, txn);

    match output.status() {
        TransactionStatus::Discard(s) => {
            bail!("txn discard, status: {:?}", s);
        }
        TransactionStatus::Keep(s) => {
            if s != &KeptVMStatus::Executed {
                bail!("txn executing error, {:?}", s)
            }
        }
    }
    Ok(output)
}

pub fn move_abort_code(status: KeptVMStatus) -> Option<u64> {
    match status {
        KeptVMStatus::MoveAbort(_, code) => Some(code),
        _ => None,
    }
}

pub fn expect_event<Event: MoveResource>(output: &TransactionOutput) -> ContractEvent {
    output
        .events()
        .iter()
        .filter(|event| event.is::<Event>())
        .cloned()
        .last()
        .unwrap_or_else(|| panic!("Expect event: {}", Event::struct_tag()))
}

pub fn expect_decode_event<Event: MoveResource + DeserializeOwned>(
    output: &TransactionOutput,
) -> Event {
    output
        .events()
        .iter()
        .filter(|event| event.is::<Event>())
        .cloned()
        .last()
        .and_then(|event| event.decode_event::<Event>().ok())
        .unwrap_or_else(|| panic!("Expect event: {}", Event::struct_tag()))
}
