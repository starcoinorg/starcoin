use crate::executor2::execute_block_transactions;
use serde::{Deserialize, Serialize};
use starcoin_types2::{
    error::{BlockExecutorError, ExecutorResult},
    vm_error::KeptVMStatus,
};
use starcoin_vm2::metrics::VMMetrics;
use starcoin_vm2_crypto::HashValue;
use starcoin_vm2_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_vm2_types::{
    contract_event::ContractEvent,
    state_store::table::{TableHandle, TableInfo},
    transaction::{Transaction, TransactionInfo, TransactionStatus},
    write_set::WriteSet,
    StateView,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockExecutedData {
    pub state_root: HashValue,
    pub txn_infos: Vec<TransactionInfo>,
    pub txn_events: Vec<Vec<ContractEvent>>,
    pub txn_table_infos: BTreeMap<TableHandle, TableInfo>,
    pub write_sets: Vec<WriteSet>,
}

impl Default for BlockExecutedData {
    fn default() -> Self {
        Self {
            state_root: HashValue::zero(),
            txn_events: vec![],
            txn_infos: vec![],
            txn_table_infos: BTreeMap::new(),
            write_sets: vec![],
        }
    }
}

pub fn execute_genesis_transaction<S: StateView + ChainStateWriter + Sync>(
    chain_state: &S,
    genesis_txn: Transaction,
) -> ExecutorResult<BlockExecutedData> {
    let txn_hash = genesis_txn.id();
    let txn_outputs = execute_block_transactions(chain_state, vec![genesis_txn], u64::MAX, None)
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

pub fn block_execute<S: ChainStateReader + ChainStateWriter + Sync>(
    chain_state: &S,
    txns: Vec<Transaction>,
    block_gas_limit: u64,
    vm_metrics: Option<VMMetrics>,
) -> ExecutorResult<BlockExecutedData> {
    let txn_outputs = execute_block_transactions(
        chain_state,
        txns.clone(),
        block_gas_limit,
        vm_metrics.clone(),
    )
    .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;

    let mut executed_data = BlockExecutedData::default();
    for (txn, output) in txns
        .iter()
        .take(txn_outputs.len())
        .zip(txn_outputs.into_iter())
    {
        let txn_hash = txn.id();
        let (write_set, events, gas_used, status, _) = output.into_inner();
        match status {
            TransactionStatus::Discard(status) => {
                return Err(BlockExecutorError::BlockTransactionDiscard(
                    status, txn_hash,
                ));
            }
            TransactionStatus::Keep(status) => {
                chain_state
                    .apply_write_set(write_set.clone())
                    .map_err(BlockExecutorError::BlockChainStateErr)?;

                let txn_state_root = chain_state
                    .commit()
                    .map_err(BlockExecutorError::BlockChainStateErr)?;
                #[cfg(testing)]
                info!("txn_hash {} gas_used {}", txn_hash, gas_used);
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
            TransactionStatus::Retry => return Err(BlockExecutorError::BlockExecuteRetryErr),
        };
    }

    executed_data.state_root = chain_state.state_root();
    Ok(executed_data)
}

fn extract_extra_writeset<S: StateView>(_chain_state: &S) -> anyhow::Result<WriteSet> {
    Ok(WriteSet::default())
}
