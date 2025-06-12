// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_vm2_crypto::ed25519::genesis_key_pair;
use starcoin_vm2_executor::{
    block_executor::BlockExecutedData, executor::do_execute_block_transactions,
};
use starcoin_vm2_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_storage::{storage::StorageInstance, Storage};
use starcoin_vm2_transaction_builder::build_stdlib_package;
use starcoin_vm2_types::{
    account_config::CORE_CODE_ADDRESS,
    error::{BlockExecutorError, ExecutorResult},
    vm_error::KeptVMStatus,
};

use move_vm2_core_types::move_resource::MoveStructType;
use starcoin_vm2_crypto::HashValue;
use starcoin_vm2_vm_types::{
    access_path::AccessPath,
    account_config::genesis_address,
    genesis_config::GenesisConfig,
    on_chain_resource::BlockMetadata,
    state_view::StateReaderExt,
    transaction::{
        Package, RawUserTransaction, SignedUserTransaction, Transaction, TransactionInfo,
        TransactionPayload, TransactionStatus,
    },
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
    block_number: Option<u64>,
) -> (SignedUserTransaction, TransactionInfo) {
    let user_txn = build_genesis_transaction(chain_id, genesis_config).unwrap();

    let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance()).unwrap());
    let chain_state = ChainStateDB::new(storage.clone(), None);

    let txn = Transaction::UserTransaction(user_txn.clone());

    let mut executed_data = execute_genesis_transaction(&chain_state, txn).unwrap();

    let mut txn_info = executed_data
        .txn_infos
        .pop()
        .expect("Execute output must exist.");

    if let Some(number) = block_number {
        // todo: handle event_root_hash?
        txn_info.state_root_hash = set_block_meta_number(&chain_state, number);
    }

    (user_txn, txn_info)
}

pub fn set_block_meta_number<S: ChainStateWriter + ChainStateReader + Sync>(
    chain_state: &S,
    number: u64,
) -> HashValue {
    let mut block_meta = chain_state.get_block_metadata().unwrap();
    block_meta.number = number;

    let access_path =
        AccessPath::resource_access_path(genesis_address(), BlockMetadata::struct_tag());
    chain_state
        .set(&access_path, bcs_ext::to_bytes(&block_meta).unwrap())
        .unwrap();

    let block_meta_1 = chain_state.get_block_metadata().unwrap();
    assert_eq!(block_meta.number, block_meta_1.number);
    assert_eq!(block_meta.parent_hash, block_meta_1.parent_hash);
    assert_eq!(block_meta.author, block_meta_1.author);
    assert_eq!(block_meta.uncles, block_meta_1.uncles);
    assert_eq!(block_meta.parents_hash, block_meta_1.parents_hash);
    assert_eq!(block_meta.new_block_events, block_meta_1.new_block_events);

    chain_state.commit().unwrap()
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

#[cfg(test)]
mod tests {
    use crate::{build_genesis_transaction, execute_genesis_transaction, set_block_meta_number};
    use starcoin_vm2_statedb::ChainStateDB;
    use starcoin_vm2_storage::storage::StorageInstance;
    use starcoin_vm2_storage::Storage;
    use starcoin_vm2_vm_types::genesis_config::G_DEV_CONFIG;
    use starcoin_vm2_vm_types::transaction::Transaction;
    use std::sync::Arc;

    #[test]
    fn test_set_block_meta() {
        let genesis_config = G_DEV_CONFIG.clone();
        let chain_id = 255; // for test
        let user_txn = build_genesis_transaction(chain_id, &genesis_config).unwrap();

        let storage = Arc::new(Storage::new(StorageInstance::new_cache_instance()).unwrap());
        let chain_state = ChainStateDB::new(storage.clone(), None);

        let txn = Transaction::UserTransaction(user_txn.clone());

        let _executed_data = execute_genesis_transaction(&chain_state, txn).unwrap();

        set_block_meta_number(&chain_state, 100);
    }
}
