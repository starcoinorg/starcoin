// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_vm_runtime::MoveVM;
use move_vm_state::{
    data_cache::{BlockDataCache, RemoteCache, RemoteStorage},
    execution_context::{ExecutionContext, SystemExecutionContext, TransactionExecutionContext},
};
use once_cell::sync::Lazy;
use starcoin_logger::prelude::*;
use starcoin_types::{
    account_config,
    block_metadata::BlockMetadata,
    transaction::{
        ChangeSet, SignatureCheckedTransaction, SignedUserTransaction, Transaction,
        TransactionArgument, TransactionOutput, TransactionPayload, TransactionStatus,
    },
    vm_error::{sub_status, StatusCode, VMStatus},
    write_set::WriteSet,
};
use starcoin_vm_types::{
    chain_state::ChainState as MoveChainState,
    errors::{convert_prologue_runtime_error, VMResult},
    gas_schedule::{self, AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, GasUnits},
    language_storage::TypeTag,
    on_chain_config::{OnChainConfig, VMConfig, Version},
    state_view::StateView,
    transaction_metadata::TransactionMetadata,
    values::Value,
};
use std::sync::Arc;

pub static KEEP_STATUS: Lazy<TransactionStatus> =
    Lazy::new(|| TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)));

// We use 10 as the assertion error code for insufficient balance within the Libra coin contract.
pub static DISCARD_STATUS: Lazy<TransactionStatus> = Lazy::new(|| {
    TransactionStatus::Discard(
        VMStatus::new(StatusCode::ABORTED).with_sub_status(StatusCode::REJECTED_WRITE_SET.into()),
    )
});

// The value should be tuned carefully
pub static MAXIMUM_NUMBER_OF_GAS_UNITS: Lazy<GasUnits<GasCarrier>> =
    Lazy::new(|| GasUnits::new(100_000_000));

#[derive(Clone, Default)]
/// Wrapper of MoveVM
pub struct StarcoinVM {
    move_vm: Arc<MoveVM>,
    vm_config: Option<VMConfig>,
    version: Option<Version>,
}

static ZERO_TABLE: Lazy<CostTable> = Lazy::new(gas_schedule::zero_cost_schedule);

//TODO define as argument.
pub static DEFAULT_CURRENCY_TY: Lazy<TypeTag> = Lazy::new(|| {
    account_config::type_tag_for_currency_code(account_config::STC_IDENTIFIER.to_owned())
});

impl StarcoinVM {
    pub fn new() -> Self {
        let inner = MoveVM::new();
        Self {
            move_vm: Arc::new(inner),
            vm_config: None,
            version: None,
        }
    }

    pub fn load_configs(&mut self, state: &dyn StateView) {
        self.load_configs_impl(&RemoteStorage::new(state))
    }

    fn vm_config(&self) -> VMResult<&VMConfig> {
        self.vm_config
            .as_ref()
            .ok_or_else(|| VMStatus::new(StatusCode::VM_STARTUP_FAILURE))
    }

    fn load_configs_impl(&mut self, data_cache: &dyn RemoteCache) {
        self.vm_config = VMConfig::fetch_config(data_cache);
        self.version = Version::fetch_config(data_cache);
    }

    pub fn get_gas_schedule(&self) -> VMResult<&CostTable> {
        self.vm_config
            .as_ref()
            .map(|config| &config.gas_schedule)
            .ok_or_else(|| {
                VMStatus::new(StatusCode::VM_STARTUP_FAILURE)
                    .with_sub_status(sub_status::VSF_GAS_SCHEDULE_NOT_FOUND)
            })
    }

    pub fn get_version(&self) -> VMResult<Version> {
        self.version.clone().ok_or_else(|| {
            VMStatus::new(StatusCode::VM_STARTUP_FAILURE)
                .with_sub_status(sub_status::VSF_LIBRA_VERSION_NOT_FOUND)
        })
    }

    fn check_gas(&self, txn: &SignedUserTransaction) -> Result<(), VMStatus> {
        let gas_constants = &self.get_gas_schedule()?.gas_constants;
        let raw_bytes_len = AbstractMemorySize::new(txn.raw_txn_bytes_len() as GasCarrier);
        // The transaction is too large.
        if txn.raw_txn_bytes_len() > gas_constants.max_transaction_size_in_bytes as usize {
            let error_str = format!(
                "max size: {}, txn size: {}",
                gas_constants.max_transaction_size_in_bytes,
                raw_bytes_len.get()
            );
            warn!(
                "[VM] Transaction size too big {} (max {})",
                raw_bytes_len.get(),
                gas_constants.max_transaction_size_in_bytes
            );
            return Err(
                VMStatus::new(StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE).with_message(error_str)
            );
        }

        // Check is performed on `txn.raw_txn_bytes_len()` which is the same as
        // `raw_bytes_len`
        assert!(raw_bytes_len.get() <= gas_constants.max_transaction_size_in_bytes);

        // The submitted max gas units that the transaction can consume is greater than the
        // maximum number of gas units bound that we have set for any
        // transaction.
        if txn.max_gas_amount() > gas_constants.maximum_number_of_gas_units.get() {
            let error_str = format!(
                "max gas units: {}, gas units submitted: {}",
                gas_constants.maximum_number_of_gas_units.get(),
                txn.max_gas_amount()
            );
            warn!(
                "[VM] Gas unit error; max {}, submitted {}",
                gas_constants.maximum_number_of_gas_units.get(),
                txn.max_gas_amount()
            );
            return Err(
                VMStatus::new(StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND)
                    .with_message(error_str),
            );
        }

        // The submitted transactions max gas units needs to be at least enough to cover the
        // intrinsic cost of the transaction as calculated against the size of the
        // underlying `RawTransaction`
        let min_txn_fee = gas_schedule::calculate_intrinsic_gas(raw_bytes_len, gas_constants);
        if txn.max_gas_amount() < min_txn_fee.get() {
            let error_str = format!(
                "min gas required for txn: {}, gas submitted: {}",
                min_txn_fee.get(),
                txn.max_gas_amount()
            );
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                min_txn_fee.get(),
                txn.max_gas_amount()
            );
            return Err(
                VMStatus::new(StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS)
                    .with_message(error_str),
            );
        }

        // The submitted gas price is less than the minimum gas unit price set by the VM.
        // NB: MIN_PRICE_PER_GAS_UNIT may equal zero, but need not in the future. Hence why
        // we turn off the clippy warning.
        #[allow(clippy::absurd_extreme_comparisons)]
        let below_min_bound = txn.gas_unit_price() < gas_constants.min_price_per_gas_unit.get();
        if below_min_bound {
            let error_str = format!(
                "gas unit min price: {}, submitted price: {}",
                gas_constants.min_price_per_gas_unit.get(),
                txn.gas_unit_price()
            );
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                gas_constants.min_price_per_gas_unit.get(),
                txn.gas_unit_price()
            );
            return Err(
                VMStatus::new(StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND).with_message(error_str)
            );
        }

        // The submitted gas price is greater than the maximum gas unit price set by the VM.
        if txn.gas_unit_price() > gas_constants.max_price_per_gas_unit.get() {
            let error_str = format!(
                "gas unit max price: {}, submitted price: {}",
                gas_constants.max_price_per_gas_unit.get(),
                txn.gas_unit_price()
            );
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                gas_constants.max_price_per_gas_unit.get(),
                txn.gas_unit_price()
            );
            return Err(
                VMStatus::new(StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND).with_message(error_str)
            );
        }
        Ok(())
    }

    fn verify_transaction_impl(
        &mut self,
        transaction: &SignatureCheckedTransaction,
        remote_cache: &dyn RemoteCache,
        txn_data: &TransactionMetadata,
    ) -> Result<VerifiedTranscationPayload, VMStatus> {
        let mut ctx = SystemExecutionContext::new(remote_cache, GasUnits::new(0));
        self.check_gas(transaction)?;
        match transaction.payload() {
            TransactionPayload::Script(script) => {
                if !self
                    .vm_config()?
                    .publishing_option
                    .is_allowed_script(&script.code())
                {
                    warn!("[VM] Custom scripts not allowed: {:?}", &script.code());
                    return Err(VMStatus::new(StatusCode::UNKNOWN_SCRIPT));
                };
                let result = self.run_prologue(&mut ctx, &txn_data);
                match result {
                    Ok(_) => Ok(VerifiedTranscationPayload::Script(
                        script.code().to_vec(),
                        script.ty_args().to_vec(),
                        script.args().to_vec(),
                    )),
                    Err(e) => Err(e),
                }
            }
            TransactionPayload::Module(module) => {
                if !&self.vm_config()?.publishing_option.is_open() {
                    warn!("[VM] Custom modules not allowed");
                    return Err(VMStatus::new(StatusCode::UNKNOWN_MODULE));
                };
                let result = self.run_prologue(&mut ctx, &txn_data);
                match result {
                    Ok(_) => Ok(VerifiedTranscationPayload::Module(module.code().to_vec())),
                    Err(e) => Err(e),
                }
            }
        }
    }

    pub fn verify_transaction(
        &mut self,
        state_view: &dyn StateView,
        txn: SignedUserTransaction,
    ) -> Option<VMStatus> {
        let data_cache = BlockDataCache::new(state_view);
        let txn_data = TransactionMetadata::new(&txn.clone().into());
        let signature_verified_txn = match txn.check_signature() {
            Ok(t) => t,
            Err(_) => return Some(VMStatus::new(StatusCode::INVALID_SIGNATURE)),
        };
        self.load_configs_impl(&data_cache);
        match self.verify_transaction_impl(&signature_verified_txn, &data_cache, &txn_data) {
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
    ) -> TransactionOutput {
        let mut ctx = TransactionExecutionContext::new(txn_data.max_gas_amount(), remote_cache);
        let mut failed_gas_left = GasUnits::new(0);
        let output = match payload {
            VerifiedTranscationPayload::Module(m) => {
                self.move_vm.publish_module(m, &mut ctx, txn_data)
            }
            VerifiedTranscationPayload::Script(s, ty_args, args) => {
                ////////
                let gas_schedule = match self.get_gas_schedule() {
                    Ok(s) => s,
                    Err(e) => return discard_error_output(e),
                };
                self.move_vm.execute_script(
                    s,
                    gas_schedule,
                    &mut ctx,
                    txn_data,
                    ty_args,
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
            self.run_epilogue(&mut gas_free_ctx, txn_data).ok();
            get_transaction_output(
                &mut gas_free_ctx,
                txn_data,
                VMStatus::new(StatusCode::EXECUTED),
            )
        })
        .unwrap_or_else(|err| {
            let mut gas_free_ctx = SystemExecutionContext::new(remote_cache, failed_gas_left);
            self.run_epilogue(&mut gas_free_ctx, txn_data).ok();
            failed_transaction_output(&mut gas_free_ctx, txn_data, err)
        });
        debug!("{:?}", output);
        output
    }

    fn run_prologue<T: MoveChainState>(
        &self,
        chain_state: &mut T,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let txn_sequence_number = txn_data.sequence_number();
        let txn_public_key = txn_data.authentication_key_preimage().to_vec();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let txn_expiration_time = txn_data.expiration_time();
        self.move_vm
            .execute_function(
                &account_config::ACCOUNT_MODULE,
                &account_config::PROLOGUE_NAME,
                self.get_gas_schedule()?,
                chain_state,
                &txn_data,
                vec![DEFAULT_CURRENCY_TY.clone()],
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

    fn run_epilogue<T: MoveChainState>(
        &self,
        chain_state: &mut T,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let txn_sequence_number = txn_data.sequence_number();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let gas_remaining = chain_state.remaining_gas().get();
        self.move_vm.execute_function(
            &account_config::ACCOUNT_MODULE,
            &account_config::EPILOGUE_NAME,
            self.get_gas_schedule()?,
            chain_state,
            &txn_data,
            vec![DEFAULT_CURRENCY_TY.clone()],
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
    ) -> VMResult<TransactionOutput> {
        let mut txn_data = TransactionMetadata::default();
        txn_data.sender = account_config::mint_address();
        txn_data.max_gas_amount = GasUnits::new(std::u64::MAX);

        let mut interpreter_context =
            TransactionExecutionContext::new(txn_data.max_gas_amount(), remote_cache);

        let (parent_id, timestamp, author, auth) = block_metadata.into_inner();
        let vote_maps = vec![];
        let round = 0u64;
        let args = vec![
            Value::u64(round),
            Value::u64(timestamp),
            Value::vector_u8(parent_id.to_vec()),
            Value::vector_address(vote_maps),
            Value::address(author),
            match auth {
                Some(prefix) => Value::vector_u8(prefix),
                None => Value::vector_u8(Vec::new()),
            },
        ];

        self.move_vm.execute_function(
            &account_config::BLOCK_MODULE,
            &account_config::BLOCK_PROLOGUE,
            &ZERO_TABLE,
            &mut interpreter_context,
            &txn_data,
            vec![],
            args,
        )?;

        get_transaction_output(
            &mut interpreter_context,
            &txn_data,
            VMStatus::new(StatusCode::EXECUTED),
        )
        .map(|output| {
            remote_cache.push_write_set(output.write_set());
            output
        })
    }

    fn execute_user_transaction(
        &mut self,
        txn: SignedUserTransaction,
        data_cache: &mut BlockDataCache<'_>,
    ) -> TransactionOutput {
        let txn_data = TransactionMetadata::new(&txn.clone().into());

        // check signature
        let signature_checked_txn = match txn.check_signature() {
            Ok(t) => Ok(t),
            Err(_) => Err(VMStatus::new(StatusCode::INVALID_SIGNATURE)),
        };

        match signature_checked_txn {
            Ok(txn) => {
                let verified_payload = self.verify_transaction_impl(&txn, data_cache, &txn_data);
                match verified_payload {
                    Ok(payload) => self.execute_verified_payload(data_cache, &txn_data, payload),
                    Err(e) => discard_error_output(e),
                }
            }
            Err(e) => discard_error_output(e),
        }
    }

    /// Execute a block transactions with gas_limit,
    /// if gas is used up when executing some txn, only return the outputs of previous succeed txns.
    pub fn execute_block_transactions(
        &mut self,
        state_view: &dyn StateView,
        transactions: Vec<Transaction>,
        block_gas_limit: Option<u64>,
    ) -> Result<Vec<TransactionOutput>> {
        let mut data_cache = BlockDataCache::new(state_view);

        let check_gas = block_gas_limit.is_some();
        // only used when check_gas
        let mut gas_left = block_gas_limit.unwrap_or_default();

        let mut result = vec![];
        let blocks = chunk_block_transactions(transactions);
        'outer: for block in blocks {
            match block {
                TransactionBlock::UserTransaction(txns) => {
                    self.load_configs_impl(&data_cache);
                    for transaction in txns {
                        let output = self.execute_user_transaction(transaction, &mut data_cache);

                        // only need to check for user transactions.
                        if check_gas {
                            match gas_left.checked_sub(output.gas_used()) {
                                Some(l) => gas_left = l,
                                None => break 'outer,
                            }
                        }

                        if let TransactionStatus::Keep(_) = output.status() {
                            data_cache.push_write_set(output.write_set())
                        }

                        result.push(output);
                    }
                }
                TransactionBlock::BlockPrologue(block_metadata) => {
                    self.load_configs_impl(&data_cache);
                    let output = self
                        .process_block_metadata(&mut data_cache, block_metadata)
                        .unwrap_or_else(discard_error_output);
                    if let TransactionStatus::Keep(_) = output.status() {
                        data_cache.push_write_set(output.write_set())
                    }
                    result.push(output);
                }
                TransactionBlock::ChangeSet(change_set) => {
                    //TODO change_set txn verify
                    let (write_set, events) = change_set.into_inner();
                    data_cache.push_write_set(&write_set);
                    result.push(TransactionOutput::new(
                        write_set,
                        events,
                        0,
                        KEEP_STATUS.clone(),
                    ));
                }
            }
        }
        Ok(result)
    }

    pub fn execute_transactions(
        &mut self,
        state_view: &dyn StateView,
        transactions: Vec<Transaction>,
    ) -> Result<Vec<TransactionOutput>> {
        self.execute_block_transactions(state_view, transactions, None)
    }
}

pub enum TransactionBlock {
    UserTransaction(Vec<SignedUserTransaction>),
    ChangeSet(ChangeSet),
    BlockPrologue(BlockMetadata),
}

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
            Transaction::ChangeSet(cs) => {
                if !buf.is_empty() {
                    blocks.push(TransactionBlock::UserTransaction(buf));
                    buf = vec![];
                }
                blocks.push(TransactionBlock::ChangeSet(cs));
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

pub(crate) fn discard_error_output(err: VMStatus) -> TransactionOutput {
    info!("discard error output: {:?}", err);
    // Since this transaction will be discarded, no writeset will be included.
    TransactionOutput::new(
        WriteSet::default(),
        vec![],
        0,
        TransactionStatus::Discard(err),
    )
}

/// Convert the transaction arguments into move values.
fn convert_txn_args(args: Vec<TransactionArgument>) -> Vec<Value> {
    args.into_iter()
        .map(|arg| match arg {
            TransactionArgument::U64(i) => Value::u64(i),
            TransactionArgument::Address(a) => Value::address(a),
            TransactionArgument::Bool(b) => Value::bool(b),
            TransactionArgument::U8Vector(v) => Value::vector_u8(v),
        })
        .collect()
}

fn get_transaction_output(
    ctx: &mut (impl MoveChainState + ExecutionContext),
    txn_data: &TransactionMetadata,
    status: VMStatus,
) -> VMResult<TransactionOutput> {
    let gas_used: u64 = txn_data
        .max_gas_amount()
        .sub(ctx.remaining_gas())
        .mul(txn_data.gas_unit_price())
        .get();
    let write_set = ctx.make_write_set()?;
    Ok(TransactionOutput::new(
        write_set,
        ctx.events().to_vec(),
        gas_used,
        TransactionStatus::Keep(status),
    ))
}

pub fn failed_transaction_output(
    ctx: &mut (impl MoveChainState + ExecutionContext),
    txn_data: &TransactionMetadata,
    error_code: VMStatus,
) -> TransactionOutput {
    match TransactionStatus::from(error_code) {
        TransactionStatus::Keep(status) => {
            get_transaction_output(ctx, txn_data, status).unwrap_or_else(discard_error_output)
        }
        TransactionStatus::Discard(status) => discard_error_output(status),
    }
}

pub enum VerifiedTranscationPayload {
    Script(Vec<u8>, Vec<TypeTag>, Vec<TransactionArgument>),
    Module(Vec<u8>),
}
