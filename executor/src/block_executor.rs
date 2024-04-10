// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::execute_block_transactions;
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_types::error::BlockExecutorError;
use starcoin_types::error::ExecutorResult;
use starcoin_types::transaction::TransactionStatus;
use starcoin_types::transaction::{Transaction, TransactionInfo};
use starcoin_vm_runtime::metrics::VMMetrics;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::{genesis_address, ModuleUpgradeStrategy};
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::move_resource::MoveResource;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use starcoin_vm_types::write_set::WriteSet;
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
        BlockExecutedData {
            state_root: HashValue::zero(),
            txn_events: vec![],
            txn_infos: vec![],
            txn_table_infos: BTreeMap::new(),
            write_sets: vec![],
        }
    }
}

pub fn block_execute<S: ChainStateReader + ChainStateWriter>(
    chain_state: &S,
    txns: Vec<Transaction>,
    block_gas_limit: u64,
    vm_metrics: Option<VMMetrics>,
    extra_txn: Option<Transaction>,
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
        let (mut table_infos, write_set, events, gas_used, status) = output.into_inner();
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
                // Merge more table_infos, and keep the latest TableInfo for a same TableHandle
                executed_data.txn_table_infos.append(&mut table_infos);
                executed_data.write_sets.push(write_set);
            }
            TransactionStatus::Retry => return Err(BlockExecutorError::BlockExecuteRetryErr),
        };
    }

    if extra_txn.is_some() {
        execute_extra_txn(chain_state, extra_txn.unwrap(), block_gas_limit, vm_metrics)
            .map_err(BlockExecutorError::BlockChainStateErr)?;
    }

    executed_data.state_root = chain_state.state_root();
    Ok(executed_data)
}

fn execute_extra_txn<S: ChainStateReader + ChainStateWriter>(
    chain_state: &S,
    txn: Transaction,
    block_gas_limit: u64,
    vm_metrics: Option<VMMetrics>,
) -> anyhow::Result<()> {
    let strategy_path =
        AccessPath::resource_access_path(genesis_address(), ModuleUpgradeStrategy::struct_tag());
    // Set strategy to 100
    chain_state.set(&strategy_path, vec![100])?;

    execute_block_transactions(&chain_state, vec![txn], block_gas_limit, vm_metrics)?;

    // Set strategy to 1
    chain_state.set(&strategy_path, vec![1])?;

    Ok(())
}
