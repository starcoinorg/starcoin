// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chain_state::StateStore, system_module_names::*, transaction_helper::TransactionHelper,
    transaction_helper::VerifiedTranscationPayload,
};
use config::VMConfig;
use libra_state_view::StateView;
use libra_types::{
    account_address::AccountAddress as LibraAccountAddress,
    transaction::{
        TransactionOutput as LibraTransactionOutput, TransactionStatus as LibraTransactionStatus,
    },
    vm_error::{StatusCode as LibraStatusCode, VMStatus as LibraVMStatus},
    write_set::WriteSet as LibraWriteSet,
};
use logger::prelude::*;
use move_vm_runtime::MoveVM;
use move_vm_state::{
    data_cache::{BlockDataCache, RemoteCache},
    execution_context::{ExecutionContext, SystemExecutionContext, TransactionExecutionContext},
};
use move_vm_types::chain_state::ChainState as LibraChainState;
use move_vm_types::values::Value;
use std::sync::Arc;

use crypto::ed25519::Ed25519Signature;
use once_cell::sync::Lazy;
use std::collections::BTreeMap;
use std::convert::TryInto;
use types::{
    access_path::AccessPath,
    account_config,
    account_config::AccountResource,
    block_metadata::BlockMetadata,
    transaction::{
        SignatureCheckedTransaction, SignedUserTransaction, Transaction, TransactionArgument,
        TransactionOutput, TransactionPayload, TransactionStatus,
    },
    vm_error::{StatusCode, VMStatus},
};
use vm::errors::convert_prologue_runtime_error;
use vm::{
    errors::VMResult,
    gas_schedule::{CostTable, GasAlgebra, GasUnits},
    transaction_metadata::TransactionMetadata,
};

pub static KEEP_STATUS: Lazy<TransactionStatus> =
    Lazy::new(|| TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)));

// We use 10 as the assertion error code for insufficient balance within the Libra coin contract.
pub static DISCARD_STATUS: Lazy<TransactionStatus> = Lazy::new(|| {
    TransactionStatus::Discard(VMStatus::new(StatusCode::ABORTED).with_sub_status(10))
});

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
        let _ctx = SystemExecutionContext::new(data_cache, GasUnits::new(0));
        //        self.gas_schedule = self.move_vm.load_gas_schedule(&mut ctx, data_cache).ok();
        self.gas_schedule = Some(CostTable::zero());
    }

    fn get_gas_schedule(&self) -> Result<&CostTable, VMStatus> {
        self.gas_schedule
            .as_ref()
            .ok_or_else(|| VMStatus::new(StatusCode::VM_STARTUP_FAILURE))
    }

    fn verify_transaction_impl(
        &mut self,
        transaction: &SignatureCheckedTransaction,
        _state_view: &dyn StateView,
        remote_cache: &dyn RemoteCache,
        txn_data: &TransactionMetadata,
    ) -> Result<VerifiedTranscationPayload, VMStatus> {
        info!("very transaction");
        let mut ctx = SystemExecutionContext::new(remote_cache, GasUnits::new(0));
        // ToDo: check gas
        self.load_gas_schedule(remote_cache);
        let gas_schedule = self.get_gas_schedule()?;
        match transaction.payload() {
            TransactionPayload::Script(script) => {
                let result = self.run_prologue(gas_schedule, &mut ctx, &txn_data);
                match result {
                    Ok(_) => Ok(VerifiedTranscationPayload::Script(
                        script.code().to_vec(),
                        script.args().to_vec(),
                    )),
                    Err(e) => return Err(TransactionHelper::to_starcoin_vmstatus(e)),
                }
            }
            TransactionPayload::Module(module) => {
                let result = self.run_prologue(gas_schedule, &mut ctx, &txn_data);
                match result {
                    Ok(_) => Ok(VerifiedTranscationPayload::Module(module.code().to_vec())),
                    Err(e) => return Err(TransactionHelper::to_starcoin_vmstatus(e)),
                }
            }
            _ => Err(VMStatus::new(StatusCode::UNREACHABLE)),
        }
    }

    pub fn verify_transaction(
        &mut self,
        chain_state: &dyn traits::ChainState,
        txn: SignedUserTransaction,
    ) -> Option<VMStatus> {
        let mut state_store = StateStore::new(chain_state);
        let data_cache = BlockDataCache::new(&state_store);
        let mut ctx = SystemExecutionContext::new(&data_cache, GasUnits::new(0));
        let libra_txn = TransactionHelper::to_libra_signed_transaction(&txn);
        let txn_data = TransactionMetadata::new(&libra_txn);
        let signature_verified_txn = match txn.check_signature() {
            Ok(t) => t,
            Err(_) => return Some(VMStatus::new(StatusCode::INVALID_SIGNATURE)),
        };
        match self.verify_transaction_impl(
            &signature_verified_txn,
            &state_store,
            &data_cache,
            &txn_data,
        ) {
            Ok(_) => None,
            Err(err) => {
                if err.major_status == StatusCode::SEQUENCE_NUMBER_TOO_NEW {
                    None
                } else {
                    Some(err)
                }
            }
        }
    }
    fn execute_verified_payload(
        &mut self,
        remote_cache: &mut BlockDataCache<'_>,
        txn_data: &TransactionMetadata,
        payload: VerifiedTranscationPayload,
    ) -> LibraTransactionOutput {
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
                        return discard_libra_error_output(TransactionHelper::to_libra_vmstatus(e))
                    }
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
            failed_gas_left = ctx.remaining_gas();
            err
        })
        .and_then(|_| {
            failed_gas_left = ctx.remaining_gas();
            let mut gas_free_ctx = SystemExecutionContext::from(ctx);
            self.run_epilogue(&mut gas_free_ctx, txn_data);
            get_transaction_output(
                &mut gas_free_ctx,
                txn_data,
                LibraVMStatus::new(LibraStatusCode::EXECUTED),
            )
        })
        .unwrap_or_else(|err| {
            let mut gas_free_ctx = SystemExecutionContext::new(remote_cache, failed_gas_left);
            self.run_epilogue(&mut gas_free_ctx, txn_data);
            failed_transaction_output(&mut gas_free_ctx, txn_data, err)
        });
        // TODO convert to starcoin type
        info!("{:?}", output);
        output
    }

    fn run_prologue<T: LibraChainState>(
        &self,
        gas_schedule: &CostTable,
        chain_state: &mut T,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let txn_sequence_number = txn_data.sequence_number();
        let txn_public_key = txn_data.public_key().to_bytes().to_vec();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let txn_expiration_time = txn_data.expiration_time();
        self.move_vm
            .execute_function(
                &ACCOUNT_MODULE,
                &PROLOGUE_NAME,
                gas_schedule,
                chain_state,
                &txn_data,
                vec![
                    Value::u64(txn_sequence_number),
                    Value::vector_u8(txn_public_key),
                    Value::u64(txn_gas_price),
                    Value::u64(txn_max_gas_units),
                    Value::u64(txn_expiration_time),
                ],
            )
            .map_err(|err| convert_prologue_runtime_error(&err, &txn_data.sender))
    }

    fn run_epilogue<T: LibraChainState>(
        &self,
        chain_state: &mut T,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let txn_sequence_number = txn_data.sequence_number();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let gas_remaining = chain_state.remaining_gas().get();
        let gas_schedule = match self.get_gas_schedule() {
            Ok(s) => s,
            Err(e) => return Err(TransactionHelper::to_libra_vmstatus(e)),
        };
        self.move_vm.execute_function(
            &ACCOUNT_MODULE,
            &EPILOGUE_NAME,
            gas_schedule,
            chain_state,
            &txn_data,
            vec![
                Value::u64(txn_sequence_number),
                Value::u64(txn_gas_price),
                Value::u64(txn_max_gas_units),
                Value::u64(gas_remaining),
            ],
        )
    }

    fn process_block_metadata(
        &self,
        remote_cache: &mut BlockDataCache<'_>,
        block_metadata: BlockMetadata,
    ) -> VMResult<LibraTransactionOutput> {
        let mut txn_data = TransactionMetadata::default();
        txn_data.sender = account_config::mint_address().into();
        txn_data.max_gas_amount = GasUnits::new(std::u64::MAX);

        let mut interpreter_context =
            TransactionExecutionContext::new(txn_data.max_gas_amount(), remote_cache);
        let gas_schedule = CostTable::zero();

        if let Ok((id, timestamp, author, auth)) = block_metadata.into_inner() {
            let previous_vote: BTreeMap<LibraAccountAddress, Ed25519Signature> = BTreeMap::new();
            let vote_maps = scs::to_bytes(&previous_vote).unwrap();
            let args = vec![
                Value::u64(timestamp),
                Value::vector_u8(id.into_inner()),
                Value::vector_u8(vote_maps),
                Value::address(author.into()),
                match auth {
                    Some(prefix) => Value::vector_u8(prefix),
                    None => Value::vector_u8(Vec::new()),
                },
            ];

            self.move_vm.execute_function(
                &LIBRA_BLOCK_MODULE,
                &BLOCK_PROLOGUE,
                &gas_schedule,
                &mut interpreter_context,
                &txn_data,
                args,
            )?
        } else {
            return Err(LibraVMStatus::new(LibraStatusCode::MALFORMED));
        };
        get_transaction_output(
            &mut interpreter_context,
            &txn_data,
            LibraVMStatus::new(LibraStatusCode::EXECUTED),
        )
        .map(|output| {
            remote_cache.push_write_set(output.write_set());
            output
        })
    }

    pub fn execute_transaction(
        &mut self,
        chain_state: &dyn traits::ChainState,
        txn: Transaction,
    ) -> TransactionOutput {
        let mut state_store = StateStore::new(chain_state);
        info!("new remote cache");
        let mut data_cache = BlockDataCache::new(&state_store);
        self.load_gas_schedule(&data_cache);

        match txn {
            Transaction::UserTransaction(txn) => {
                let libra_txn = TransactionHelper::to_libra_signed_transaction(&txn);
                let txn_data = TransactionMetadata::new(&libra_txn);

                // check signature
                let signature_checked_txn = match txn.check_signature() {
                    Ok(t) => Ok(t),
                    Err(_) => Err(VMStatus::new(StatusCode::INVALID_SIGNATURE)),
                };

                let output = match signature_checked_txn {
                    Ok(txn) => {
                        let verified_payload = self.verify_transaction_impl(
                            &txn,
                            &state_store,
                            &data_cache,
                            &txn_data,
                        );

                        let result = match verified_payload {
                            Ok(payload) => {
                                self.execute_verified_payload(&mut data_cache, &txn_data, payload)
                            }
                            Err(e) => {
                                info!("we are here!!!");
                                discard_libra_error_output(TransactionHelper::to_libra_vmstatus(e))
                            }
                        };

                        if let LibraTransactionStatus::Keep(_) = result.status() {
                            state_store.add_write_set(result.write_set())
                        };
                        TransactionHelper::to_starcoin_transaction_output(result)
                    }
                    Err(e) => {
                        info!("we are here!!!");
                        discard_error_output(e)
                    }
                };
                output
            }
            Transaction::BlockMetadata(block_metadata) => {
                // let (_id, _timestamp, author, _auth) = block_metadata.into_inner().unwrap();
                // let access_path = AccessPath::new_for_account(author);
                // let account_resource: AccountResource = state_store
                //     .get_from_statedb(&access_path)
                //     .and_then(|blob| match blob {
                //         Some(blob) => Ok(blob),
                //         None => {
                //             state_store.create_account(author).unwrap();
                //             Ok(state_store
                //                 .get_from_statedb(&access_path)
                //                 .unwrap()
                //                 .expect("account resource must exist."))
                //         }
                //     })
                //     .and_then(|blob| blob.try_into())
                //     .unwrap();
                //
                // let new_account_resource = AccountResource::new(
                //     account_resource.balance() + 50_00000000,
                //     account_resource.sequence_number(),
                //     account_resource.authentication_key().clone(),
                // );
                // state_store
                //     .set(access_path, new_account_resource.try_into().unwrap())
                //     .unwrap();
                // TransactionOutput::new(vec![], 0, KEEP_STATUS.clone())
                let result = self
                    .process_block_metadata(&mut data_cache, block_metadata)
                    .unwrap();
                if let LibraTransactionStatus::Keep(_) = result.status() {
                    state_store.add_write_set(result.write_set())
                };
                TransactionHelper::to_starcoin_transaction_output(result)
            }
            Transaction::StateSet(state_set) => {
                let result_status = match chain_state.apply(state_set) {
                    Ok(_) => KEEP_STATUS.clone(),
                    Err(_) => DISCARD_STATUS.clone(),
                };
                TransactionOutput::new(vec![], 0, result_status)
            }
        }
    }
}

pub(crate) fn discard_error_output(err: VMStatus) -> TransactionOutput {
    info!("discard error output: {:?}", err);
    // Since this transaction will be discarded, no writeset will be included.
    TransactionOutput::new(vec![], 0, TransactionStatus::Discard(err))
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
            TransactionArgument::Address(a) => {
                Value::address(TransactionHelper::to_libra_account_address(a))
            }
            TransactionArgument::Bool(b) => Value::bool(b),
            TransactionArgument::U8Vector(v) => Value::vector_u8(v),
        })
        .collect()
}

fn get_transaction_output(
    ctx: &mut (impl LibraChainState + ExecutionContext),
    txn_data: &TransactionMetadata,
    status: LibraVMStatus,
) -> VMResult<LibraTransactionOutput> {
    let gas_used: u64 = txn_data
        .max_gas_amount()
        .sub(ctx.remaining_gas())
        .mul(txn_data.gas_unit_price())
        .get();
    let write_set = ctx.make_write_set()?;
    Ok(LibraTransactionOutput::new(
        write_set,
        ctx.events().to_vec(),
        gas_used,
        LibraTransactionStatus::Keep(status),
    ))
}

pub fn failed_transaction_output(
    ctx: &mut (impl LibraChainState + ExecutionContext),
    txn_data: &TransactionMetadata,
    error_code: LibraVMStatus,
) -> LibraTransactionOutput {
    match LibraTransactionStatus::from(error_code) {
        LibraTransactionStatus::Keep(status) => {
            get_transaction_output(ctx, txn_data, status).unwrap_or_else(discard_libra_error_output)
        }
        LibraTransactionStatus::Discard(status) => discard_libra_error_output(status),
        _ => unreachable!(),
    }
}
