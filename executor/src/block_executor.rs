// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::execute_block_transactions;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue;
use starcoin_metrics::metrics::VMMetrics;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_types::error::BlockExecutorError;
use starcoin_types::error::ExecutorResult;
use starcoin_types::transaction::TransactionStatus;
use starcoin_types::transaction::{Transaction, TransactionInfo};
use starcoin_vm_types::account_config::{BalanceEvent, STCUnit, G_STC_TOKEN_CODE};
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use starcoin_vm_types::write_set::WriteSet;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

static LOGGER_BALANCE_AMOUNT: AtomicU64 = AtomicU64::new(1_000_000_u64);

/// Sets LOGGER_BALANCE_AMOUNT when invoked the first time.
pub fn set_logger_balance_amount_once(logger_balance_amount: u64) {
    // Only the first call succeeds, due to OnceCell semantics.
    LOGGER_BALANCE_AMOUNT.store(logger_balance_amount, Ordering::Relaxed);
    info!("LOGGER_BALANCE_AMOUNT set {}", logger_balance_amount);
}

/// Get the LOGGER_BALANCE_AMOUNT if already set, otherwise return default 1_000_000
pub fn get_logger_balance_amount() -> u64 {
    LOGGER_BALANCE_AMOUNT.load(Ordering::Relaxed)
}

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

impl BlockExecutedData {
    pub fn gas_used(&self) -> u64 {
        self.txn_infos
            .iter()
            .map(|txn_info| txn_info.gas_used())
            .sum()
    }
}

pub fn block_execute<S: ChainStateReader + ChainStateWriter>(
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
        let (mut table_infos, write_set, events, gas_used, status) = output.into_inner();
        let balance_amount = get_logger_balance_amount();
        let _ = events.iter().any(|e| {
            if let Ok(balance_event) = BalanceEvent::try_from(e) {
                let res = balance_event.token_code() == &G_STC_TOKEN_CODE.clone()
                    && balance_event.amount()
                        > STCUnit::STC.value_of(balance_amount as u128).scaling();
                if res {
                    warn!("Logging Event: txn_hash {}, {}", txn_hash, balance_event);
                }
                res
            } else {
                false
            }
        });

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
                #[cfg(feature = "testing")]
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

    executed_data.state_root = chain_state.state_root();
    Ok(executed_data)
}
