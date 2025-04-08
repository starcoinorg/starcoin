use starcoin_config::genesis_config::ChainNetwork;
use starcoin_crypto2::ed25519::genesis_key_pair;
use starcoin_types2::{
    account_config::CORE_CODE_ADDRESS,
    error::{BlockExecutorError, ExecutorResult},
    vm_error::KeptVMStatus,
};
use starcoin_vm2_executor::{
    block_executor2::BlockExecutedData, executor2::do_execute_block_transactions,
};
use starcoin_vm2_state_api::ChainStateWriter;
use starcoin_vm2_transaction_builder::{
    build_stdlib_package as build_stdlib_package_2, build_stdlib_package_for_test,
};
use starcoin_vm2_types::{
    transaction::{
        Package, RawUserTransaction, SignedUserTransaction, Transaction, TransactionInfo,
        TransactionPayload, TransactionStatus,
    },
    write_set::WriteSet,
    StateView,
};

pub fn build_genesis_transaction_with_package(
    net: &ChainNetwork,
    package: Package,
) -> anyhow::Result<SignedUserTransaction> {
    let txn = RawUserTransaction::new_with_default_gas_token(
        CORE_CODE_ADDRESS,
        0,
        TransactionPayload::Package(package),
        0,
        0,
        1, // init to 1 to pass time check
        net.chain_id().id().into(),
    );
    let (genesis_private_key, genesis_public_key) = genesis_key_pair();
    let sign_txn = txn.sign(&genesis_private_key, genesis_public_key)?;
    Ok(sign_txn.into_inner())
}

pub fn build_genesis_transaction(net: &ChainNetwork) -> anyhow::Result<SignedUserTransaction> {
    let entry_func =
        build_stdlib_package_2(net.chain_id().id().into(), net.genesis_config2(), None);
    let package = build_stdlib_package_for_test(None, Some(entry_func))?;
    build_genesis_transaction_with_package(&net, package)
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
    let extra_write_set =
        extract_extra_writeset(chain_state).expect("extract extra write set failed");
    let write_set = write_set
        .into_mut()
        .squash(extra_write_set.into_mut())
        .expect("failed to squash write set")
        .freeze()
        .expect("failed to freeze write set");
    match status {
        TransactionStatus::Keep(status) => {
            assert_eq!(status, KeptVMStatus::Executed);
            chain_state
                .apply_write_set(write_set.clone())
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
            executed_data.write_sets.push(write_set);
        }
        _ => unreachable!("genesis txn output must be keep"),
    }

    Ok(executed_data)
}

fn extract_extra_writeset<S: StateView>(_chain_state: &S) -> anyhow::Result<WriteSet> {
    Ok(WriteSet::default())
}
