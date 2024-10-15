// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

/*
use crate::access_path_cache::AccessPathCache;
use crate::data_cache::{AsMoveResolver, RemoteStorage, StateViewCache};
use crate::errors::{
    convert_normal_success_epilogue_error, convert_prologue_runtime_error, error_split,
};
use crate::move_vm_ext::{MoveVmExt, SessionId};
use crate::vm_adapter::{
    discard_error_output, discard_error_vm_status, PreprocessedTransaction,
    PublishModuleBundleOption, SessionAdapter, VMAdapter,
}; */
use crate::{PreprocessedTransaction, VMAdapter, VMExecutor};
use anyhow::{Error, Result};
use num_cpus;
use once_cell::sync::OnceCell;
use starcoin_gas_schedule::{InitialGasSchedule, NativeGasParameters, StarcoinGasParameters};
use starcoin_logger::prelude::*;
use starcoin_metrics::metrics::VMMetrics;
use starcoin_types::{
    block_metadata::BlockMetadata,
    transaction::{
        SignatureCheckedTransaction, SignedUserTransaction, Transaction, TransactionOutput,
    },
};

use starcoin_vm_types::identifier::IdentStr;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::on_chain_config::{FlexiDagConfig, GasSchedule, MoveLanguageVersion};

use starcoin_vm_types::state_store::StateView;
use starcoin_vm_types::transaction::DryRunTransaction;

use starcoin_vm_types::{
    language_storage::TypeTag,
    on_chain_config::{VMConfig, Version},
    vm_status::VMStatus,
};

use std::cmp::min;

static EXECUTION_CONCURRENCY_LEVEL: OnceCell<usize> = OnceCell::new();

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
/// Wrapper of MoveVM
pub struct StarcoinVM {
    vm_config: Option<VMConfig>,
    version: Option<Version>,
    move_version: Option<MoveLanguageVersion>,
    native_params: NativeGasParameters,
    gas_params: Option<StarcoinGasParameters>,
    gas_schedule: Option<GasSchedule>,
    flexi_dag_config: Option<FlexiDagConfig>,
    #[cfg(feature = "metrics")]
    metrics: Option<VMMetrics>,
}

impl StarcoinVM {
    #[cfg(feature = "metrics")]
    pub fn new(metrics: Option<VMMetrics>) -> Self {
        let gas_params = StarcoinGasParameters::initial();
        let native_params = gas_params.natives.clone();
        Self {
            vm_config: None,
            version: None,
            move_version: None,
            native_params,
            gas_params: Some(gas_params),
            gas_schedule: None,
            flexi_dag_config: None,
            metrics,
        }
    }
    #[cfg(not(feature = "metrics"))]
    pub fn new() -> Self {
        let gas_params = StarcoinGasParameters::initial();
        let native_params = gas_params.natives.clone();
        Self {
            move_vm: Arc::new(inner),
            vm_config: None,
            version: None,
            move_version: None,
            native_params,
            gas_params: Some(gas_params),
            gas_schedule: None,
            flexi_dag_config: None,
        }
    }

    pub fn load_configs<S: StateView>(&mut self, _state: &S) -> Result<(), Error> {
        Ok(())
    }

    fn load_configs_impl<S: StateView>(&mut self, _state: &S) -> Result<(), Error> {
        Ok(())
    }

    pub fn get_move_version(&self) -> Option<MoveLanguageVersion> {
        self.move_version
    }

    fn verify_transaction_impl<S: StateView>(
        &mut self,
        _transaction: &SignatureCheckedTransaction,
        _remote_cache: &S,
    ) -> Result<(), VMStatus> {
        Ok(())
    }

    pub fn verify_transaction<S: StateView>(
        &mut self,
        _state_view: &S,
        _txn: SignedUserTransaction,
    ) -> Option<VMStatus> {
        todo!("verify_transaction")
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn execute_user_transaction<S: StateView>(
        &self,
        _storage: &S,
        _txn: SignedUserTransaction,
    ) -> (VMStatus, TransactionOutput) {
        todo!("execute_user_transaction")
    }

    pub fn dry_run_transaction<S: StateView>(
        &mut self,
        _storage: &S,
        _txn: DryRunTransaction,
    ) -> Result<(VMStatus, TransactionOutput)> {
        todo!("dry_run_transaction")
    }

    fn check_reconfigure<S: StateView>(
        &mut self,
        _state_view: &S,
        _output: &TransactionOutput,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// Execute a block transactions with gas_limit,
    /// if gas is used up when executing some txn, only return the outputs of previous succeed txns.
    pub fn execute_block_transactions<S: StateView>(
        &mut self,
        _storage: &S,
        _transactions: Vec<Transaction>,
        _block_gas_limit: Option<u64>,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>, VMStatus> {
        todo!("execute_block_transactions")
    }

    pub fn execute_readonly_function<S: StateView>(
        &mut self,
        _state_view: &S,
        _module: &ModuleId,
        _function_name: &IdentStr,
        _type_params: Vec<TypeTag>,
        _args: Vec<Vec<u8>>,
    ) -> Result<Vec<Vec<u8>>, VMStatus> {
        todo!("execute_readonly_function")
    }

    /// Sets execution concurrency level when invoked the first time.
    pub fn set_concurrency_level_once(mut concurrency_level: usize) {
        concurrency_level = min(concurrency_level, num_cpus::get());
        // Only the first call succeeds, due to OnceCell semantics.
        EXECUTION_CONCURRENCY_LEVEL.set(concurrency_level).ok();
        info!("TurboSTM executor concurrency_level {}", concurrency_level);
    }

    /// Get the concurrency level if already set, otherwise return default 1
    /// (sequential execution).
    pub fn get_concurrency_level() -> usize {
        match EXECUTION_CONCURRENCY_LEVEL.get() {
            Some(concurrency_level) => *concurrency_level,
            None => 1,
        }
    }

    /// Alternate form of 'execute_block' that keeps the vm_status before it goes into the
    /// `TransactionOutput`
    pub fn execute_block_and_keep_vm_status(
        txns: Vec<Transaction>,
        state_view: &impl StateView,
        block_gas_limit: Option<u64>,
        metrics: Option<VMMetrics>,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>, VMStatus> {
        let mut vm = Self::new(metrics);
        vm.execute_block_transactions(state_view, txns, block_gas_limit)
    }
}

#[allow(clippy::large_enum_variant)]
pub enum TransactionBlock {
    UserTransaction(Vec<SignedUserTransaction>),
    BlockPrologue(BlockMetadata),
}

impl TransactionBlock {
    pub fn type_name(&self) -> &str {
        match self {
            Self::UserTransaction(_) => "UserTransaction",
            Self::BlockPrologue(_) => "BlockMetadata",
        }
    }
}

/// TransactionBlock::UserTransaction | TransactionBlock::BlockPrologue | TransactionBlock::UserTransaction
pub fn chunk_block_transactions(txns: Vec<Transaction>) -> Vec<TransactionBlock> {
    let mut blocks = vec![];
    let mut buf = vec![];
    for txn in txns {
        match txn {
            Transaction::BlockMetadata(data) => {
                if !buf.is_empty() {
                    blocks.push(TransactionBlock::UserTransaction(buf));
                    buf = vec![];
                }
                blocks.push(TransactionBlock::BlockPrologue(data));
            }
            Transaction::UserTransaction(txn) => {
                buf.push(txn);
            }
        }
    }
    if !buf.is_empty() {
        blocks.push(TransactionBlock::UserTransaction(buf));
    }
    blocks
}

// Executor external API
impl VMExecutor for StarcoinVM {
    /// Execute a block of `transactions`. The output vector will have the exact same length as the
    /// input vector. The discarded transactions will be marked as `TransactionStatus::Discard` and
    /// have an empty `WriteSet`. Also `state_view` is immutable, and does not have interior
    /// mutability. Writes to be applied to the data view are encoded in the write set part of a
    /// transaction output.
    fn execute_block(
        _transactions: Vec<Transaction>,
        _state_view: &impl StateView,
        _block_gas_limit: Option<u64>,
        _metrics: Option<VMMetrics>,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        todo!("execute_block")
    }
}

impl VMAdapter for StarcoinVM {
    fn should_restart_execution(_output: &TransactionOutput) -> bool {
        false
    }

    fn execute_single_transaction<S: StateView>(
        &self,
        _txn: &PreprocessedTransaction,
        _data_cache: &S,
    ) -> Result<(VMStatus, TransactionOutput, Option<String>), VMStatus> {
        todo!("execute_single_transaction")
    }
}
