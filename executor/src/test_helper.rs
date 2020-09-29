// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_config::ChainNetwork;
use starcoin_genesis::Genesis;
use starcoin_state_api::{AccountStateReader, ChainState, ChainStateWriter};
use starcoin_types::transaction::TransactionOutput;
use starcoin_types::{
    account_address::AccountAddress, transaction::Module, transaction::Transaction,
    transaction::TransactionStatus,
};
use starcoin_vm_types::account_config::genesis_address;
use statedb::ChainStateDB;
use stdlib::stdlib_files;

pub fn prepare_genesis() -> (ChainStateDB, &'static ChainNetwork) {
    let net = &ChainNetwork::TEST;
    let chain_state = ChainStateDB::mock();
    let genesis_txn = Genesis::build_genesis_transaction(net).unwrap();
    Genesis::execute_genesis_txn(&chain_state, genesis_txn).unwrap();
    (chain_state, net)
}

pub fn execute_and_apply(chain_state: &ChainStateDB, txn: Transaction) -> TransactionOutput {
    let output = crate::execute_transactions(chain_state, vec![txn])
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
