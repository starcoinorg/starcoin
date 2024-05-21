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
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use starcoin_vm_types::write_set::WriteSet;
use std::collections::BTreeMap;

#[cfg(feature = "force-deploy")]
use {
    crate::execute_transactions,
    anyhow::bail,
    log::info,
    starcoin_force_upgrade::ForceUpgrade,
    starcoin_types::account::DEFAULT_EXPIRATION_TIME,
    starcoin_types::identifier::Identifier,
    starcoin_vm_runtime::force_upgrade_management::{
        get_force_upgrade_account, get_force_upgrade_block_number,
    },
    starcoin_vm_types::{
        access_path::AccessPath,
        account_config::{genesis_address, ModuleUpgradeStrategy},
        genesis_config::StdlibVersion,
        move_resource::MoveResource,
        on_chain_config,
        on_chain_config::Version,
        state_store::state_key::StateKey,
        state_view::StateReaderExt,
    },
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockExecutedData {
    pub state_root: HashValue,
    pub txn_infos: Vec<TransactionInfo>,
    pub txn_events: Vec<Vec<ContractEvent>>,
    pub txn_table_infos: BTreeMap<TableHandle, TableInfo>,
    pub write_sets: Vec<WriteSet>,
    #[cfg(feature = "force-deploy")]
    #[serde(skip)]
    pub with_extra_txn: bool,
}

impl Default for BlockExecutedData {
    fn default() -> Self {
        BlockExecutedData {
            state_root: HashValue::zero(),
            txn_events: vec![],
            txn_infos: vec![],
            txn_table_infos: BTreeMap::new(),
            write_sets: vec![],
            #[cfg(feature = "force-deploy")]
            with_extra_txn: false,
        }
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

    #[cfg(feature = "force-deploy")]
    if let Some(extra_txn) = create_force_upgrade_extra_txn(chain_state)
        .map_err(BlockExecutorError::BlockChainStateErr)?
    {
        // !!! commit suicide if any error or exception happens !!!
        execute_extra_txn(chain_state, extra_txn, vm_metrics, &mut executed_data)
            .expect("extra txn must be executed successfully");
        executed_data.with_extra_txn = true;
    }

    executed_data.state_root = chain_state.state_root();
    Ok(executed_data)
}

#[cfg(feature = "force-deploy")]
fn create_force_upgrade_extra_txn<S: ChainStateReader + ChainStateWriter>(
    statedb: &S,
) -> anyhow::Result<Option<Transaction>> {
    // Only execute extra_txn when stdlib version is 11
    if statedb
        .get_on_chain_config::<Version>()?
        .map(|v| v.into_stdlib_version())
        .map(|v| v != StdlibVersion::Version(11))
        .unwrap_or(true)
    {
        return Ok(None);
    }

    let chain_id = statedb.get_chain_id()?;
    let block_timestamp = statedb.get_timestamp()?.seconds();
    let block_number = statedb.get_block_metadata()?.number;

    Ok(
        if block_number == get_force_upgrade_block_number(&chain_id) {
            let account = get_force_upgrade_account(&chain_id)?;
            let sequence_number = statedb.get_sequence_number(*account.address())?;
            let extra_txn = ForceUpgrade::force_deploy_txn(
                account,
                sequence_number,
                block_timestamp + DEFAULT_EXPIRATION_TIME,
                &chain_id,
            )?;
            info!(
                "create_force_upgrade_extra_txn | block_number: ({:?}) extra txn to execute ({:?})",
                block_number,
                extra_txn.id()
            );
            Some(Transaction::UserTransaction(extra_txn))
        } else {
            None
        },
    )
}

// todo: check the execute_extra_txn in OpenedBlock, and merge with it
#[cfg(feature = "force-deploy")]
fn execute_extra_txn<S: ChainStateReader + ChainStateWriter>(
    chain_state: &S,
    txn: Transaction,
    vm_metrics: Option<VMMetrics>,
    executed_data: &mut BlockExecutedData,
) -> anyhow::Result<()> {
    let txn_hash = txn.id();
    let strategy_path =
        AccessPath::resource_access_path(genesis_address(), ModuleUpgradeStrategy::struct_tag());

    // retrieve the original strategy value
    let old_val = chain_state
        .get_state_value(&StateKey::AccessPath(strategy_path.clone()))?
        .expect("module upgrade strategy should exist");
    // Set strategy to 100 upgrade package directly
    chain_state.set(&strategy_path, vec![100])?;

    let output = execute_transactions(&chain_state, vec![txn], vm_metrics)?
        .pop()
        .expect("extra txn must have output");

    // restore strategy to old value
    chain_state.set(&strategy_path, old_val)?;

    let (mut table_infos, write_set, events, _gas_used, status) = output.into_inner();
    match status {
        TransactionStatus::Discard(status) => {
            bail!("extra txn {txn_hash:?} is discarded: {status:?}");
        }
        TransactionStatus::Keep(status) => {
            chain_state
                .apply_write_set(write_set.clone())
                .map_err(BlockExecutorError::BlockChainStateErr)?;
            {
                // update stdlib version to 12 directly
                let version_path = on_chain_config::access_path_for_config(
                    genesis_address(),
                    Identifier::new("Version").unwrap(),
                    Identifier::new("Version").unwrap(),
                    vec![],
                );
                let version = on_chain_config::Version { major: 12 };
                chain_state.set(&version_path, bcs_ext::to_bytes(&version)?)?;
            }

            let txn_state_root = chain_state
                .commit()
                .map_err(BlockExecutorError::BlockChainStateErr)?;
            executed_data.txn_infos.push(TransactionInfo::new(
                txn_hash,
                txn_state_root,
                events.as_slice(),
                // skip the gas_used
                0,
                status,
            ));
            executed_data.txn_events.push(events);
            // Merge more table_infos, and keep the latest TableInfo for a same TableHandle
            executed_data.txn_table_infos.append(&mut table_infos);
            executed_data.write_sets.push(write_set);
        }
        TransactionStatus::Retry => {
            bail!("extra txn {txn_hash:?} must not to retry");
        }
    }

    Ok(())
}
