// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_vm2_crypto::{ed25519::genesis_key_pair, HashValue};
use starcoin_vm2_executor::{
    block_executor::BlockExecutedData, executor::do_execute_block_transactions,
};
use starcoin_vm2_state_api::ChainStateWriter;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_storage::{storage::StorageInstance, Storage};
use starcoin_vm2_transaction_builder::build_stdlib_package;
use starcoin_vm2_types::{
    account_config::CORE_CODE_ADDRESS,
    error::{BlockExecutorError, ExecutorResult},
    vm_error::KeptVMStatus,
};

use starcoin_vm2_vm_types::{
    genesis_config::GenesisConfig,
    transaction::TransactionInfo,
    transaction::{Package, RawUserTransaction, SignedUserTransaction, TransactionPayload},
    transaction::{Transaction, TransactionStatus},
    StateView,
};
use std::sync::Arc;

fn build_genesis_transaction_with_package(
    chain_id: u8,
    package: Package,
) -> anyhow::Result<SignedUserTransaction> {
    let txn = RawUserTransaction::new_with_default_gas_token(
        CORE_CODE_ADDRESS,
        0,
        TransactionPayload::Package(package),
        0,
        0,
        1, // init to 1 to pass time check
        chain_id.into(),
    );
    let (genesis_private_key, genesis_public_key) = genesis_key_pair();
    let sign_txn = txn.sign(&genesis_private_key, genesis_public_key)?;
    Ok(sign_txn.into_inner())
}

fn build_genesis_transaction(
    chain_id: u8,
    genesis_config: &GenesisConfig,
) -> anyhow::Result<SignedUserTransaction> {
    let package = build_stdlib_package(chain_id.into(), genesis_config, None)?;
    build_genesis_transaction_with_package(chain_id, package)
}

pub fn build_and_execute_genesis_transaction(
    chain_id: u8,
    genesis_config: &GenesisConfig,
) -> (SignedUserTransaction, HashValue) {
    let user_txn = build_genesis_transaction(chain_id, genesis_config).unwrap();

    let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance()).unwrap());
    let chain_state = ChainStateDB::new(storage.clone(), None);

    let txn = Transaction::UserTransaction(user_txn.clone());

    (
        user_txn,
        execute_genesis_transaction(&chain_state, txn)
            .unwrap()
            .txn_infos
            .pop()
            .expect("Execute output must exist.")
            .id(),
    )
}

pub fn execute_genesis_transaction<S: StateView + ChainStateWriter + Sync>(
    chain_state: &S,
    genesis_txn: Transaction,
) -> ExecutorResult<BlockExecutedData> {
    let txn_hash = genesis_txn.id();
    let txn_outputs = do_execute_block_transactions(chain_state, vec![genesis_txn], None, None)
        .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
    assert_eq!(
        txn_outputs.len(),
        1,
        "genesis txn output count must be 1, but got {}",
        txn_outputs.len()
    );
    let mut executed_data = BlockExecutedData::default();
    // this expect will never fail, as we have checked the output count is 1.
    let (write_set, events, gas_used, status, _) = txn_outputs
        .first()
        .expect("genesis txn must have one output")
        .clone()
        .into_inner();
    match status {
        TransactionStatus::Keep(status) => {
            assert_eq!(status, KeptVMStatus::Executed);
            chain_state
                .apply_write_set(write_set)
                .map_err(BlockExecutorError::BlockChainStateErr)?;

            let txn_state_root = chain_state
                .commit()
                .map_err(BlockExecutorError::BlockChainStateErr)?;
            executed_data.state_root = txn_state_root;
            executed_data.txn_infos.push(TransactionInfo::new(
                txn_hash,
                txn_state_root,
                events.as_slice(),
                gas_used,
                status,
            ));
            executed_data.txn_events.push(events);
        }
        _ => unreachable!("genesis txn output must be keep"),
    }

    Ok(executed_data)
}
