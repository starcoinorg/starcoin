// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::executor::do_execute_block_transactions;
use anyhow::anyhow;
use log::{info, log_enabled, Level};
use serde::{Deserialize, Serialize};
pub use starcoin_metrics::metrics::VMMetrics;
use starcoin_vm2_crypto::HashValue;
use starcoin_vm2_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_vm2_types::contract_event::ContractEvent;
use starcoin_vm2_types::{
    error::{BlockExecutorError, ExecutorResult},
    transaction::{Transaction, TransactionInfo, TransactionOutput, TransactionStatus},
};
use starcoin_vm2_vm_types::state_store::table::{TableHandle, TableInfo};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockExecutedData {
    pub state_root: HashValue,
    pub txn_infos: Vec<TransactionInfo>,
    pub txn_events: Vec<Vec<ContractEvent>>,
    pub txn_table_infos: BTreeMap<TableHandle, TableInfo>,
}

impl Default for BlockExecutedData {
    fn default() -> Self {
        Self {
            state_root: HashValue::zero(),
            txn_events: vec![],
            txn_infos: vec![],
            txn_table_infos: BTreeMap::new(),
        }
    }
}

pub fn block_execute<S: ChainStateReader + ChainStateWriter + Sync>(
    chain_state: &S,
    txns: Vec<Transaction>,
    block_gas_limit: u64,
    vm_metrics: Option<VMMetrics>,
) -> ExecutorResult<BlockExecutedData> {
    let stats_enabled = log_enabled!(target: "vm2-blockbench", Level::Info);
    let total_start = if stats_enabled {
        Some(Instant::now())
    } else {
        None
    };

    if txns
        .iter()
        .any(|txn| matches!(txn, Transaction::BlockEpilogue(..)))
    {
        return Err(BlockExecutorError::OtherError(
            anyhow!("block_execute expects no BlockEpilogue in input").into(),
        ));
    }

    debug_assert!(
        !txns
            .iter()
            .any(|txn| matches!(txn, Transaction::BlockEpilogue(..))),
        "block_execute expects no BlockEpilogue in input"
    );

    let exec_start = if stats_enabled {
        Some(Instant::now())
    } else {
        None
    };
    let block_meta = txns.iter().find_map(|txn| match txn {
        Transaction::BlockMetadata(meta) => Some(meta.clone()),
        _ => None,
    });
    let has_user_txns = txns
        .iter()
        .any(|txn| matches!(txn, Transaction::UserTransaction(_)));
    let should_execute_epilogue = block_meta.is_some() && has_user_txns;
    let txn_outputs = do_execute_block_transactions(
        chain_state,
        txns.clone(),
        Some(block_gas_limit),
        vm_metrics.clone(),
    )
    .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
    let exec_ms = exec_start
        .map(|start| start.elapsed().as_secs_f64() * 1000.0)
        .unwrap_or(0.0);
    let mut executed_data = BlockExecutedData::default();
    let last_index = txn_outputs
        .len()
        .checked_sub(1)
        .ok_or_else(|| BlockExecutorError::BlockTransactionZero)?;
    let mut total_fee: u128 = 0;
    let mut total_gas_used: u64 = 0;
    let mut apply_elapsed = Duration::from_secs(0);
    let mut commit_elapsed = Duration::from_secs(0);
    for (index, (txn, output)) in txns
        .iter()
        .take(txn_outputs.len())
        .zip(txn_outputs.into_iter())
        .enumerate()
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
                if let Transaction::UserTransaction(user_txn) = txn {
                    total_gas_used = total_gas_used.saturating_add(gas_used);
                    total_fee = total_fee.saturating_add(
                        u128::from(gas_used) * u128::from(user_txn.gas_unit_price()),
                    );
                }
                let apply_start = if stats_enabled {
                    Some(Instant::now())
                } else {
                    None
                };
                chain_state
                    .apply_write_set(write_set)
                    .map_err(BlockExecutorError::BlockChainStateErr)?;
                if let Some(start) = apply_start {
                    apply_elapsed += start.elapsed();
                }
                let txn_state_root =
                    if index == 0 || (!should_execute_epilogue && index == last_index) {
                        let commit_start = if stats_enabled {
                            Some(Instant::now())
                        } else {
                            None
                        };
                        let root = chain_state
                            .commit()
                            .map_err(BlockExecutorError::BlockChainStateErr)?;
                        if let Some(start) = commit_start {
                            commit_elapsed += start.elapsed();
                        }
                        Some(root)
                    } else {
                        None
                    };
                let t = TransactionInfo::new(
                    txn_hash,
                    txn_state_root,
                    events.as_slice(),
                    gas_used,
                    status,
                );
                executed_data.txn_infos.push(t);
                executed_data.txn_events.push(events);
            }
            TransactionStatus::Retry => return Err(BlockExecutorError::BlockExecuteRetryErr),
        };
    }

    let mut epilogue_exec_ms = 0.0;
    let mut epilogue_apply_ms = 0.0;
    let mut epilogue_commit_ms = 0.0;
    if should_execute_epilogue {
        let total_fee = u64::try_from(total_fee)
            .map_err(|_| BlockExecutorError::OtherError(anyhow!("total fee overflow").into()))?;
        let epilogue_txn =
            Transaction::BlockEpilogue(block_meta.expect("block meta is present"), total_fee);
        let epilogue_exec_start = if stats_enabled {
            Some(Instant::now())
        } else {
            None
        };
        let mut outputs = do_execute_block_transactions(
            chain_state,
            vec![epilogue_txn.clone()],
            Some(block_gas_limit.saturating_sub(total_gas_used)),
            vm_metrics,
        )
        .map_err(BlockExecutorError::BlockTransactionExecuteErr)?;
        if let Some(start) = epilogue_exec_start {
            epilogue_exec_ms = start.elapsed().as_secs_f64() * 1000.0;
        }
        let output = outputs
            .pop()
            .ok_or_else(|| BlockExecutorError::BlockTransactionZero)?;

        let txn_hash = epilogue_txn.id();
        let (write_set, events, gas_used, status, _) = output.into_inner();
        match status {
            TransactionStatus::Discard(status) => {
                return Err(BlockExecutorError::BlockTransactionDiscard(
                    status, txn_hash,
                ));
            }
            TransactionStatus::Keep(status) => {
                let epilogue_apply_start = if stats_enabled {
                    Some(Instant::now())
                } else {
                    None
                };
                chain_state
                    .apply_write_set(write_set)
                    .map_err(BlockExecutorError::BlockChainStateErr)?;
                if let Some(start) = epilogue_apply_start {
                    epilogue_apply_ms = start.elapsed().as_secs_f64() * 1000.0;
                }
                let epilogue_commit_start = if stats_enabled {
                    Some(Instant::now())
                } else {
                    None
                };
                let txn_state_root = Some(
                    chain_state
                        .commit()
                        .map_err(BlockExecutorError::BlockChainStateErr)?,
                );
                if let Some(start) = epilogue_commit_start {
                    epilogue_commit_ms = start.elapsed().as_secs_f64() * 1000.0;
                }
                let t = TransactionInfo::new(
                    txn_hash,
                    txn_state_root,
                    events.as_slice(),
                    gas_used,
                    status,
                );
                executed_data.txn_infos.push(t);
                executed_data.txn_events.push(events);
            }
            TransactionStatus::Retry => return Err(BlockExecutorError::BlockExecuteRetryErr),
        };
    }
    executed_data.state_root = chain_state.state_root();
    if stats_enabled {
        info!(
            target: "vm2-blockbench",
            "vm2_exec.kernel_ms={:.3}",
            exec_ms
        );
        info!(
            target: "vm2-blockbench",
            "vm2_exec.apply_ms={:.3}",
            apply_elapsed.as_secs_f64() * 1000.0
        );
        info!(
            target: "vm2-blockbench",
            "vm2_exec.commit_ms={:.3}",
            commit_elapsed.as_secs_f64() * 1000.0
        );
        if should_execute_epilogue {
            info!(
                target: "vm2-blockbench",
                "vm2_exec.epilogue_exec_ms={:.3}",
                epilogue_exec_ms
            );
            info!(
                target: "vm2-blockbench",
                "vm2_exec.epilogue_apply_ms={:.3}",
                epilogue_apply_ms
            );
            info!(
                target: "vm2-blockbench",
                "vm2_exec.epilogue_commit_ms={:.3}",
                epilogue_commit_ms
            );
        }
        if let Some(start) = total_start {
            info!(
                target: "vm2-blockbench",
                "vm2_exec.total_ms={:.3}",
                start.elapsed().as_secs_f64() * 1000.0
            );
        }
    }
    Ok(executed_data)
}

/// Execute block with pre-computed outputs (from cache)
pub fn block_execute_with_outputs<S: ChainStateReader + ChainStateWriter + Sync>(
    chain_state: &S,
    txns: Vec<Transaction>,
    outputs: Vec<TransactionOutput>,
) -> ExecutorResult<BlockExecutedData> {
    let mut executed_data = BlockExecutedData::default();
    let last_index = outputs
        .len()
        .checked_sub(1)
        .ok_or_else(|| BlockExecutorError::BlockTransactionZero)?;
    if txns.len() != outputs.len() {
        return Err(BlockExecutorError::InvalidOutputCount {
            expected: txns.len(),
            actual: outputs.len(),
        });
    }
    for (index, (txn, output)) in txns
        .iter()
        .take(outputs.len())
        .zip(outputs.into_iter())
        .enumerate()
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
                    .apply_write_set(write_set)
                    .map_err(BlockExecutorError::BlockChainStateErr)?;
                let txn_state_root = if index == last_index || index == 0 {
                    Some(
                        chain_state
                            .commit()
                            .map_err(BlockExecutorError::BlockChainStateErr)?,
                    )
                } else {
                    None
                };
                let t = TransactionInfo::new(
                    txn_hash,
                    txn_state_root,
                    events.as_slice(),
                    gas_used,
                    status,
                );
                executed_data.txn_infos.push(t);
                executed_data.txn_events.push(events);
            }
            TransactionStatus::Retry => return Err(BlockExecutorError::BlockExecuteRetryErr),
        };
    }
    executed_data.state_root = chain_state.state_root();
    Ok(executed_data)
}
