// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_types::error::BlockExecutorError;
use starcoin_types::error::ExecutorResult;
use starcoin_types::transaction::TransactionStatus;
use starcoin_types::transaction::{Transaction, TransactionInfo};
use starcoin_vm_runtime::metrics::VMMetrics;
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockExecutedData {
    pub state_root: HashValue,
    pub txn_infos: Vec<TransactionInfo>,
    pub txn_events: Vec<Vec<ContractEvent>>,
    pub txn_table_infos: BTreeMap<TableHandle, TableInfo>,
}

impl Default for BlockExecutedData {
    fn default() -> Self {
        BlockExecutedData {
            state_root: HashValue::zero(),
            txn_events: vec![],
            txn_infos: vec![],
            txn_table_infos: BTreeMap::new(),
        }
    }
}

pub fn block_execute<S: ChainStateReader + ChainStateWriter>(
    chain_state: &S,
    txns: Vec<Transaction>,
    block_gas_limit: u64,
    vm_metrics: Option<VMMetrics>,
) -> ExecutorResult<BlockExecutedData> {
    let txn_outputs =
        crate::execute_block_transactions(chain_state, txns.clone(), block_gas_limit, vm_metrics)
            .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;

    let mut executed_data = BlockExecutedData::default();
    for (txn, output) in txns
        .iter()
        .take(txn_outputs.len())
        .zip(txn_outputs.into_iter())
    {
        let txn_hash = txn.id();
        let (mut table_infos, write_set, events, gas_used, status) = output.into_inner();
        match status {
            TransactionStatus::Discard(status) => {
                return Err(BlockExecutorError::BlockTransactionDiscard(
                    status, txn_hash,
                ));
            }
            TransactionStatus::Keep(status) => {
                chain_state
                    .apply_write_set(write_set)
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
                // Merge more table_infos, and keep the latest TableInfo for a same TableHandle
                executed_data.txn_table_infos.append(&mut table_infos);
            }
            TransactionStatus::Retry => return Err(BlockExecutorError::BlockExecuteRetryErr),
        };
    }

    executed_data.state_root = chain_state.state_root();
    Ok(executed_data)
}
