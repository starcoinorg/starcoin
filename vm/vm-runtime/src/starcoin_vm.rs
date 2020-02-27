// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chain_state::StateStore, transaction_helper::TransactionHelper,
    transaction_helper::VerifiedTranscationPayload,
};
use config::VMConfig;
use libra_state_view::StateView;
use libra_types::{
    transaction::{
        Module as LibraModule, Script as LibraScript,
        SignatureCheckedTransaction as LibraSignatureCheckedTransaction,
        SignedTransaction as LibraSignedTransaction,
        TransactionArgument as LibraTransactionArgument,
        TransactionOutput as LibraTransactionOutput, TransactionPayload as LibraTransactionPayload,
        TransactionStatus as LibraTransactionStatus,
    },
    vm_error::{StatusCode as LibraStatusCode, VMStatus as LibraVMStatus},
    write_set::{
        WriteOp as LibraWriteOp, WriteSet as LibraWriteSet, WriteSetMut as LibraMutWriteSetMut,
    },
};
use std::sync::Arc;
use traits::ChainState;
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
    account_state::AccountState,
    transaction::{
        RawUserTransaction, Script, SignatureCheckedTransaction, SignedUserTransaction,
        Transaction, TransactionArgument, TransactionOutput, TransactionPayload, TransactionStatus,
    },
    vm_error::{StatusCode, VMStatus},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use vm::{
    errors::VMResult,
    gas_schedule::{self, AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, GasUnits},
    transaction_metadata::TransactionMetadata,
};
use vm_runtime::{
    chain_state::{
        ChainState as LibraChainState, SystemExecutionContext, TransactionExecutionContext,
    },
    data_cache::{BlockDataCache, RemoteCache},
    move_vm::MoveVM,
};
use vm_runtime_types::value::Value;

#[derive(Clone)]
/// Wrapper of MoveVM
pub struct StarcoinVM {
    move_vm: Arc<MoveVM>,
    gas_schedule: Option<CostTable>,
    config: VMConfig,
}

impl StarcoinVM {
    pub fn new(config: &VMConfig) -> Self {
        let inner = MoveVM::new();
        Self {
            move_vm: Arc::new(inner),
            gas_schedule: None,
            config: config.clone(),
        }
    }

    fn load_gas_schedule(&mut self, data_cache: &dyn RemoteCache) {
        let mut ctx = SystemExecutionContext::new(data_cache, GasUnits::new(0));
        self.gas_schedule = self.move_vm.load_gas_schedule(&mut ctx, data_cache).ok();
    }

    fn get_gas_schedule(&self) -> VMResult<&CostTable> {
        self.gas_schedule
            .as_ref()
            .ok_or_else(|| LibraVMStatus::new(LibraStatusCode::VM_STARTUP_FAILURE))
    }

    fn verify_transaction(
        &self,
        transaction: &LibraSignatureCheckedTransaction,
        state_view: &dyn StateView,
        remote_cache: &dyn RemoteCache,
    ) -> VMResult<VerifiedTranscationPayload> {
        let mut ctx = SystemExecutionContext::new(remote_cache, GasUnits::new(0));
        // ToDo: check gas
        //let txn = TransactionHelper::to_libra_SignedTransaction(&transaction);
        let txn_data = TransactionMetadata::new(&transaction);
        match transaction.payload() {
            LibraTransactionPayload::Script(script) => {
                // ToDo: run prologue?
                Ok(VerifiedTranscationPayload::LibraScript(
                    script.code().to_vec(),
                    script.args().to_vec(),
                ))
            }
            LibraTransactionPayload::Module(module) => {
                // ToDo: run prologue?
                Ok(VerifiedTranscationPayload::LibraModule(
                    module.code().to_vec(),
                ))
            }
            _ => Err(LibraVMStatus::new(LibraStatusCode::UNREACHABLE)),
        }
    }

    fn execute_verified_payload(
        &mut self,
        remote_cache: &mut BlockDataCache<'_>,
        txn_data: &TransactionMetadata,
        payload: VerifiedTranscationPayload,
    ) -> libra_types::transaction::TransactionOutput {
        let mut ctx = TransactionExecutionContext::new(txn_data.max_gas_amount(), remote_cache);
        let mut failed_gas_left = GasUnits::new(0);
        match payload {
            VerifiedTranscationPayload::LibraModule(m) => {
                self.move_vm.publish_module(m, &mut ctx, txn_data)
            }
            VerifiedTranscationPayload::LibraScript(s, args) => {
                let gas_schedule = match self.get_gas_schedule() {
                    Ok(s) => s,
                    Err(e) => return discard_error_output(e),
                };
                self.move_vm.execute_script(
                    s,
                    gas_schedule,
                    &mut ctx,
                    txn_data,
                    convert_txn_args(args),
                )
            }
        }
        .map_err(|err| {
            failed_gas_left = ctx.gas_left();
            err
        })
        .and_then(|_| {
            failed_gas_left = ctx.gas_left();
            let mut gas_free_ctx = SystemExecutionContext::from(ctx);
            gas_free_ctx.get_transaction_output(txn_data, Ok(()))
        })
        .unwrap_or_else(|err| {
            let mut gas_free_ctx = SystemExecutionContext::new(remote_cache, failed_gas_left);
            gas_free_ctx
                .get_transaction_output(txn_data, Err(err))
                .unwrap_or_else(discard_error_output)
        })
    }

    pub fn execute_transaction(
        &mut self,
        chain_state: &dyn traits::ChainState,
        txn: Transaction,
    ) -> TransactionOutput {
        let state_store = StateStore::new(chain_state);
        let mut data_cache = BlockDataCache::new(&state_store);
        self.load_gas_schedule(&data_cache);

        match txn {
            Transaction::UserTransaction(txn) => {
                let txn = TransactionHelper::to_libra_SignedTransaction(&txn);
                let txn_data = TransactionMetadata::new(&txn);
                // check signature
                let signature_checked_txn = txn.check_signature().unwrap();

                let verified_payload =
                    self.verify_transaction(&signature_checked_txn, &state_store, &data_cache);

                let result = verified_payload
                    .and_then(|verified_payload| {
                        Ok(self.execute_verified_payload(
                            &mut data_cache,
                            &txn_data,
                            verified_payload,
                        ))
                    })
                    .unwrap_or_else(discard_error_output);

                if let libra_types::transaction::TransactionStatus::Keep(_) = result.status() {
                    data_cache.push_write_set(result.write_set())
                };
                // TODO convert to starcoin type
                TransactionHelper::to_starcoin_TransactionOutput(result)
            }
            _ => TransactionHelper::fake_starcoin_TransactionOutput(),
        }
    }
}

pub(crate) fn discard_error_output(err: LibraVMStatus) -> LibraTransactionOutput {
    // Since this transaction will be discarded, no writeset will be included.
    LibraTransactionOutput::new(
        LibraWriteSet::default(),
        vec![],
        0,
        LibraTransactionStatus::Discard(err),
    )
}

/// Convert the transaction arguments into move values.
fn convert_txn_args(args: Vec<LibraTransactionArgument>) -> Vec<Value> {
    args.into_iter()
        .map(|arg| match arg {
            LibraTransactionArgument::U64(i) => Value::u64(i),
            LibraTransactionArgument::Address(a) => Value::address(a),
            LibraTransactionArgument::Bool(b) => Value::bool(b),
            LibraTransactionArgument::ByteArray(b) => Value::byte_array(b),
        })
        .collect()
}
