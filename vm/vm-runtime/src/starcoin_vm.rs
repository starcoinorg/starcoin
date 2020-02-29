// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chain_state::StateStore,
    transaction_helper::TransactionHelper,
    transaction_helper::VerifiedTranscationPayload
};
use vm_runtime::{
    chain_state::{ChainState as LibraChainState, SystemExecutionContext, TransactionExecutionContext},
    data_cache::{BlockDataCache, RemoteCache},
    move_vm::MoveVM,
};
use config::VMConfig;
use libra_state_view::StateView;
use std::sync::Arc;
use vm::{
    errors::VMResult,
    gas_schedule::{self, AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, GasUnits},
    transaction_metadata::TransactionMetadata,
};
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
    account_state::AccountState,
    transaction::{
        RawUserTransaction, Script, SignatureCheckedTransaction, SignedUserTransaction, Transaction, TransactionArgument,
        TransactionOutput, TransactionPayload, TransactionStatus,
    },
    vm_error::{StatusCode, VMStatus},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use vm_runtime_types::value::Value;
use libra_types::{
    transaction::{
        SignedTransaction as LibraSignedTransaction, TransactionOutput as LibraTransactionOutput,
        TransactionStatus as LibraTransactionStatus, SignatureCheckedTransaction as LibraSignatureCheckedTransaction,
        TransactionPayload as LibraTransactionPayload,
        TransactionArgument as LibraTransactionArgument,
        Script as LibraScript,
        Module as LibraModule,
    },
    vm_error::{StatusCode as LibraStatusCode, VMStatus as LibraVMStatus},
    write_set::{WriteOp as LibraWriteOp, WriteSet as LibraWriteSet, WriteSetMut as LibraMutWriteSetMut},
    byte_array::ByteArray as LibraByteArray,
};
use traits::{ChainState};
use logger::prelude::*;

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
        info!("load gas schedule");
        let mut ctx = SystemExecutionContext::new(data_cache, GasUnits::new(0));
        self.gas_schedule = self.move_vm.load_gas_schedule(&mut ctx, data_cache).ok();
    }

    fn get_gas_schedule(&self) -> Result<&CostTable, VMStatus> {
        self.gas_schedule.as_ref().ok_or_else(|| {
            VMStatus::new(StatusCode::VM_STARTUP_FAILURE)
        })
    }

    fn verify_transaction(
        &self,
        transaction: &SignatureCheckedTransaction,
        state_view: &dyn StateView,
        remote_cache: &dyn RemoteCache,
        txn_data: &TransactionMetadata,
    ) -> Result<VerifiedTranscationPayload, VMStatus> {
        info!("very transaction");
        let mut ctx = SystemExecutionContext::new(remote_cache, GasUnits::new(0));
        // ToDo: check gas
        match transaction.payload() {
            TransactionPayload::Script(script) => {
                // ToDo: run prologue?
                Ok(VerifiedTranscationPayload::Script(
                    script.code().to_vec(),
                    script.args().to_vec(),
                ))
            }
            TransactionPayload::Module(module) => {
                // ToDo: run prologue?
                Ok(VerifiedTranscationPayload::Module(module.code().to_vec()))
            }
            _ => Err(VMStatus::new(StatusCode::UNREACHABLE)),
        }
    }

    fn execute_verified_payload(
        &mut self,
        remote_cache: &mut BlockDataCache<'_>,
        txn_data: &TransactionMetadata,
        payload: VerifiedTranscationPayload,
    ) -> TransactionOutput {
        info!("execute verified payload");
        let mut ctx = TransactionExecutionContext::new(txn_data.max_gas_amount(), remote_cache);
        let mut failed_gas_left = GasUnits::new(0);
        let output = match payload {
            VerifiedTranscationPayload::Module(m) => {
                self.move_vm.publish_module(m, &mut ctx, txn_data)
            }
            VerifiedTranscationPayload::Script(s, args) => {
                ////////
                let gas_schedule = match self.get_gas_schedule() {
                    Ok(s) => s,
                    Err(e) => {
                        return discard_error_output(e)
                    },
                };
                info!("invoke MoveVM::execute_script()");
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
                gas_free_ctx.get_transaction_output(txn_data, Err(err))
                    .unwrap_or_else(discard_libra_error_output)
            });
        // TODO convert to starcoin type
        TransactionHelper::to_starcoin_TransactionOutput(output)
    }

    pub fn execute_transaction(
        &mut self,
        chain_state: &dyn traits::ChainState,
        txn: Transaction,
    ) -> TransactionOutput {
        let state_store = StateStore::new(chain_state);
        info!("new remote cache");
        let mut data_cache = BlockDataCache::new(&state_store);
        self.load_gas_schedule(&data_cache);

        match txn {
            Transaction::UserTransaction(txn) => {
                let libra_txn = TransactionHelper::to_libra_SignedTransaction(&txn);
                let txn_data = TransactionMetadata::new(&libra_txn);

                // check signature
                let signature_checked_txn = match txn.check_signature() {
                    Ok(t) => Ok(t),
                    Err(_) => Err(VMStatus::new(StatusCode::INVALID_SIGNATURE)),
                };

                let output = match signature_checked_txn {
                    Ok(txn) => {

                        let verified_payload =
                            self.verify_transaction(&txn, &state_store, &data_cache, &txn_data);

                        let result = verified_payload
                            .and_then(|verified_payload| {
                                Ok(self.execute_verified_payload(
                                    &mut data_cache,
                                    &txn_data,
                                    verified_payload,
                                ))
                            })
                            .unwrap_or_else(discard_error_output);

                        if let TransactionStatus::Keep(_) = result.status() {
                            //ToDo: when to write back the state changes?
//                            data_cache.push_write_set(result.write_set())
                        };
                        result
                    }
                    Err(e) => {
                        info!("we are here!!!");
                        discard_error_output(e)
                    },
                };
                output
            }
            _ => TransactionHelper::fake_starcoin_TransactionOutput()

        }
    }
}

pub(crate) fn discard_error_output(err: VMStatus) -> TransactionOutput {
    info!("discard error output: {:?}", err);
    // Since this transaction will be discarded, no writeset will be included.
    TransactionOutput::new(
        vec![],
        0,
        TransactionStatus::Discard(err),
    )
}

pub(crate) fn discard_libra_error_output(err: LibraVMStatus) -> LibraTransactionOutput {
    info!("discard error output: {:?}", err);
    // Since this transaction will be discarded, no writeset will be included.
    LibraTransactionOutput::new(
        LibraWriteSet::default(),
        vec![],
        0,
        LibraTransactionStatus::Discard(err),
    )
}

/// Convert the transaction arguments into move values.
fn convert_txn_args(args: Vec<TransactionArgument>) -> Vec<Value> {
    args.into_iter()
        .map(|arg| match arg {
            TransactionArgument::U64(i) => Value::u64(i),
            TransactionArgument::Address(a) => Value::address(TransactionHelper::to_libra_AccountAddress(a)),
            TransactionArgument::Bool(b) => Value::bool(b),
            TransactionArgument::ByteArray(b) => Value::byte_array(LibraByteArray::new((b.clone()).into_inner())),
        })
        .collect()
}

