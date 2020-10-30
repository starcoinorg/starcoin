// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Account;
use crate::Genesis;
use anyhow::{bail, Result};
use starcoin_account_api::AccountPrivateKey;
use starcoin_config::{ChainNetwork, GenesisConfig};
use starcoin_executor::{execute_readonly_function, execute_transactions, DEFAULT_MAX_GAS_AMOUNT};
use starcoin_state_api::{AccountStateReader, ChainState};
use starcoin_statedb::{ChainStateDB, ChainStateWriter};
use starcoin_types::account_config::{association_address, genesis_address};
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_types::genesis_config::ChainId;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_types::transaction::{
    RawUserTransaction, SignedUserTransaction, TransactionOutput, TransactionPayload,
};
use starcoin_types::vm_error::KeptVMStatus;
use starcoin_types::{
    account_address::AccountAddress, transaction::Module, transaction::Transaction,
    transaction::TransactionStatus,
};
use starcoin_vm_types::values::VMValueCast;
use stdlib::stdlib_files;

//TODO warp to A MockTxnExecutor

pub const TEST_MODULE: &str = r#"
    module M {
        struct Foo { a: u8 }
        public fun foo(): u8 { 1 }
    }
    "#;
pub const TEST_MODULE_1: &str = r#"
    module M {
        struct Foo { a: address }
        public fun foo(): u8 { 1 }
    }
    "#;
pub const TEST_MODULE_2: &str = r#"
    module M {
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

pub fn execute_and_apply(chain_state: &ChainStateDB, txn: Transaction) -> TransactionOutput {
    let output = execute_transactions(chain_state, vec![txn])
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

pub fn get_sequence_number(addr: AccountAddress, chain_state: &dyn ChainState) -> u64 {
    let account_reader = AccountStateReader::new(chain_state.as_super());
    account_reader
        .get_account_resource(&addr)
        .expect("read account state should ok")
        .map(|res| res.sequence_number())
        .unwrap_or_default()
}

pub fn get_balance(address: AccountAddress, chain_state: &dyn ChainState) -> u128 {
    let account_reader = AccountStateReader::new(chain_state.as_super());
    account_reader
        .get_balance(&address)
        .expect("read balance resource should ok")
        .unwrap_or_default()
}

pub fn compile_module_with_address(address: AccountAddress, code: &str) -> Module {
    let stdlib_files = stdlib_files();
    let compiled_result =
        starcoin_move_compiler::compile_source_string_no_report(code, &stdlib_files, address)
            .expect("compile fail")
            .1
            .expect("compile fail");
    Module::new(compiled_result.serialize())
}
#[allow(unused)]
pub fn compile_script(code: impl AsRef<str>) -> Vec<u8> {
    let stdlib_files = stdlib_files();
    let compile_unit = starcoin_move_compiler::compile_source_string_no_report(
        code.as_ref(),
        &stdlib_files,
        genesis_address(),
    )
    .expect("compile fail")
    .1
    .expect("compile fail");
    compile_unit.serialize()
}

pub fn association_execute(
    config: &GenesisConfig,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<()> {
    let txn = build_raw_txn(association_address(), state, payload, ChainId::test());
    let txn = config.sign_with_association(txn)?;
    execute_signed_txn(state, txn)
}
pub fn account_execute(
    account: &Account,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<()> {
    user_execute(*account.address(), account.private_key(), state, payload)
}
pub fn blockmeta_execute(state: &ChainStateDB, meta: BlockMetadata) -> Result<()> {
    let txn = Transaction::BlockMetadata(meta);
    let output = execute_and_apply(state, txn);
    if let TransactionStatus::Discard(s) = output.status() {
        bail!("txn discard, status: {:?}", s);
    }

    Ok(())
}

pub fn build_raw_txn(
    user_address: AccountAddress,
    state: &ChainStateDB,
    payload: TransactionPayload,
    chain_id: ChainId,
) -> RawUserTransaction {
    let seq_number = get_sequence_number(user_address, state);

    let now: u64 = {
        let mut ret = execute_readonly_function(
            state,
            &ModuleId::new(genesis_address(), Identifier::new("Timestamp").unwrap()),
            &Identifier::new("now_seconds").unwrap(),
            vec![],
            vec![],
        )
        .unwrap();
        assert_eq!(ret.len(), 1);
        // should never fail
        ret.pop().unwrap().1.cast().unwrap()
    };

    RawUserTransaction::new(
        user_address,
        seq_number,
        payload,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        now + 60 * 60,
        chain_id,
    )
}

fn user_execute(
    user_address: AccountAddress,
    prikey: &AccountPrivateKey,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<()> {
    let txn = build_raw_txn(user_address, state, payload, ChainId::test());
    let signature = prikey.sign(&txn);
    let txn = signature.build_transaction(txn)?;
    execute_signed_txn(state, txn)
}

fn execute_signed_txn(state: &ChainStateDB, txn: SignedUserTransaction) -> Result<()> {
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
    Ok(())
}
