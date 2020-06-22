// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::data_cache::{RemoteStorage, StateViewCache};
use crate::metrics::TXN_EXECUTION_GAS_USAGE;
use anyhow::Result;
use bytecode_verifier::VerifiedModule;
use move_vm_runtime::data_cache::TransactionDataCache;
use move_vm_runtime::{data_cache::RemoteCache, move_vm::MoveVM};
use once_cell::sync::Lazy;
use starcoin_logger::prelude::*;
use starcoin_types::language_storage::CORE_CODE_ADDRESS;
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
use starcoin_vm_types::access::ModuleAccess;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::{stc_type_tag, EPILOGUE_NAME, PROLOGUE_NAME};
use starcoin_vm_types::data_store::DataStore;
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::gas_schedule::{zero_cost_schedule, CostStrategy};
use starcoin_vm_types::transaction::{Module, Script, UpgradePackage};
use starcoin_vm_types::{
    errors,
    errors::{convert_prologue_runtime_error, VMResult},
    gas_schedule::{self, AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, GasUnits},
    language_storage::TypeTag,
    on_chain_config::{OnChainConfig, VMConfig, Version},
    state_view::StateView,
    transaction_metadata::TransactionMetadata,
    values::Value,
};
use std::sync::Arc;
use vm::IndexKind;

pub static KEEP_STATUS: Lazy<TransactionStatus> =
    Lazy::new(|| TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)));

// We use 10 as the assertion error code for insufficient balance within the Libra coin contract.
pub static DISCARD_STATUS: Lazy<TransactionStatus> = Lazy::new(|| {
    TransactionStatus::Discard(
        VMStatus::new(StatusCode::ABORTED).with_sub_status(StatusCode::REJECTED_WRITE_SET.into()),
    )
});

pub static ZERO_COST_TABLE: Lazy<CostTable> = Lazy::new(zero_cost_schedule);

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

//TODO define as argument.
pub static DEFAULT_CURRENCY_TY: Lazy<TypeTag> = Lazy::new(stc_type_tag);

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
        if state.is_genesis() {
            return;
        }
        self.load_configs_impl(&RemoteStorage::new(state))
    }

    fn vm_config(&self) -> VMResult<&VMConfig> {
        self.vm_config
            .as_ref()
            .ok_or_else(|| VMStatus::new(StatusCode::VM_STARTUP_FAILURE))
    }

    fn load_configs_impl(&mut self, data_cache: &dyn RemoteCache) {
        if let Some(vm_config) = VMConfig::fetch_config(data_cache) {
            self.vm_config = Some(vm_config);
        }
        if let Some(version) = Version::fetch_config(data_cache) {
            self.version = Some(version);
        }
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
        if let TransactionPayload::Package(_) = txn.payload() {
            //TODO PackageUpgrade txn gas verify.
            return Ok(());
        }
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

    fn verify_script(
        &self,
        remote_cache: &dyn RemoteCache,
        script: &Script,
        txn_data: &TransactionMetadata,
    ) -> VMResult<VerifiedTransactionPayload> {
        let mut cost_strategy = CostStrategy::system(self.get_gas_schedule()?, GasUnits::new(0));
        let mut data_store = TransactionDataCache::new(remote_cache);
        if !self
            .vm_config()?
            .publishing_option
            .is_allowed_script(&script.code())
        {
            warn!("[VM] Custom scripts not allowed: {:?}", &script.code());
            return Err(VMStatus::new(StatusCode::UNKNOWN_SCRIPT));
        };
        self.run_prologue(&mut data_store, &mut cost_strategy, &txn_data)?;
        Ok(VerifiedTransactionPayload::Script(
            script.code().to_vec(),
            script.ty_args().to_vec(),
            convert_txn_args(script.args()),
        ))
    }

    fn verify_module(
        &self,
        remote_cache: &dyn RemoteCache,
        module: &Module,
        txn_data: &TransactionMetadata,
    ) -> VMResult<VerifiedTransactionPayload> {
        let mut cost_strategy = CostStrategy::system(self.get_gas_schedule()?, GasUnits::new(0));
        let mut data_store = TransactionDataCache::new(remote_cache);
        if !&self.vm_config()?.publishing_option.is_open() {
            warn!("[VM] Custom modules not allowed");
            return Err(VMStatus::new(StatusCode::UNKNOWN_MODULE));
        };
        self.run_prologue(&mut data_store, &mut cost_strategy, &txn_data)?;
        Ok(VerifiedTransactionPayload::Module(module.code().to_vec()))
    }

    fn verify_upgrade(
        &self,
        _remote_cache: &dyn RemoteCache,
        upgrade: &UpgradePackage,
        _txn_data: &TransactionMetadata,
    ) -> VMResult<VerifiedTransactionPayload> {
        //TODO custom prologue, check sender in prologue
        //self.run_prologue(&mut data_store, &mut cost_strategy, &txn_data)?;
        Ok(VerifiedTransactionPayload::Package(upgrade.clone()))
    }

    fn verify_transaction_impl(
        &mut self,
        transaction: &SignatureCheckedTransaction,
        remote_cache: &dyn RemoteCache,
        txn_data: &TransactionMetadata,
    ) -> Result<VerifiedTransactionPayload, VMStatus> {
        self.check_gas(transaction)?;
        match transaction.payload() {
            TransactionPayload::Script(script) => {
                self.verify_script(remote_cache, script, txn_data)
            }
            TransactionPayload::Module(module) => {
                self.verify_module(remote_cache, module, txn_data)
            }
            TransactionPayload::Package(upgrade) => {
                self.verify_upgrade(remote_cache, upgrade, txn_data)
            }
        }
    }

    pub fn verify_transaction(
        &mut self,
        state_view: &dyn StateView,
        txn: SignedUserTransaction,
    ) -> Option<VMStatus> {
        let data_cache = StateViewCache::new(state_view);
        let txn_data = TransactionMetadata::new(&txn);
        let signature_verified_txn = match txn.check_signature() {
            Ok(t) => t,
            Err(_) => return Some(VMStatus::new(StatusCode::INVALID_SIGNATURE)),
        };
        self.load_configs(state_view);
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
        remote_cache: &mut StateViewCache<'_>,
        txn_data: &TransactionMetadata,
        payload: VerifiedTransactionPayload,
    ) -> TransactionOutput {
        //TODO handle genesis transaction space case.
        let gas_schedule = if remote_cache.is_genesis() {
            &ZERO_COST_TABLE
        } else {
            match self.get_gas_schedule() {
                Ok(s) => s,
                Err(e) => return discard_error_output(e),
            }
        };
        let mut cost_strategy = CostStrategy::transaction(gas_schedule, txn_data.max_gas_amount());
        let mut data_store = TransactionDataCache::new(remote_cache);
        // TODO: The logic for handling falied transaction fee is pretty ugly right now. Fix it later.
        let mut failed_gas_left = GasUnits::new(0);
        match payload {
            VerifiedTransactionPayload::Module(m) => {
                self.move_vm
                    .publish_module(m, txn_data.sender(), &mut data_store)
            }
            VerifiedTransactionPayload::Script(s, ty_args, args) => {
                let ret = self.move_vm.execute_script(
                    s,
                    ty_args,
                    args,
                    txn_data.sender(),
                    &mut data_store,
                    &mut cost_strategy,
                );
                let gas_usage = txn_data
                    .max_gas_amount()
                    .sub(cost_strategy.remaining_gas())
                    .get();
                TXN_EXECUTION_GAS_USAGE.observe(gas_usage as f64);
                ret
            }
            VerifiedTransactionPayload::Package(package) => {
                cost_strategy = CostStrategy::system(gas_schedule, GasUnits::new(0));
                let (modules, scripts) = package.into_inner();
                let mut ret: VMResult<Vec<_>> = modules
                    .iter()
                    .map(|module| Self::update_module(txn_data.sender, module, &mut data_store))
                    .collect();
                if ret.is_ok() {
                    ret = scripts
                        .iter()
                        .map(|init_script| {
                            let sender = init_script
                                .su_account()
                                .unwrap_or_else(|| txn_data.sender());
                            let ty_args = init_script.script().ty_args().to_vec();
                            let args = convert_txn_args(init_script.script().args());
                            let s = init_script.script().code().to_vec();
                            debug!("execute init script by account {:?}", sender);
                            self.move_vm.execute_script(
                                s,
                                ty_args,
                                args,
                                sender,
                                &mut data_store,
                                &mut cost_strategy,
                            )
                        })
                        .collect();
                }
                ret.map(|_| ())
            }
        }
        .map_err(|err| {
            failed_gas_left = cost_strategy.remaining_gas();
            debug!("execute payload error: {:?}", err);
            err
        })
        .and_then(|_| {
            failed_gas_left = cost_strategy.remaining_gas();
            let mut cost_strategy = CostStrategy::system(gas_schedule, failed_gas_left);
            //TODO handle genesis txn's epilogue.
            if txn_data.sender != CORE_CODE_ADDRESS {
                self.run_epilogue(&mut data_store, &mut cost_strategy, txn_data)
                    .and_then(|_| {
                        get_transaction_output(
                            &mut data_store,
                            &cost_strategy,
                            txn_data,
                            VMStatus::new(StatusCode::EXECUTED),
                        )
                    })
            } else {
                get_transaction_output(
                    &mut data_store,
                    &cost_strategy,
                    txn_data,
                    VMStatus::new(StatusCode::EXECUTED),
                )
            }
        })
        .unwrap_or_else(|err| {
            self.failed_transaction_cleanup(
                err,
                gas_schedule,
                failed_gas_left,
                txn_data,
                remote_cache,
            )
        })
    }

    fn update_module(
        sender: AccountAddress,
        module: &Module,
        data_store: &mut dyn DataStore,
    ) -> VMResult<()> {
        //TODO move_vm should support method verify and update a module.
        let code = module.code().to_vec();
        let compiled_module = match CompiledModule::deserialize(code.as_slice()) {
            Ok(module) => module,
            Err(err) => {
                warn!("[VM] module deserialization failed {:?}", err);
                return Err(err);
            }
        };
        //TODO supported authorize to deploy other address for genesis.
        // Make sure the module's self address matches the transaction sender. The self address is
        // where the module will actually be published. If we did not check this, the sender could
        // publish a module under anyone's account.
        if compiled_module.address() != &sender {
            return Err(errors::verification_error(
                IndexKind::AddressIdentifier,
                compiled_module.self_handle_idx().0 as usize,
                StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER,
            ));
        }

        let module_id = compiled_module.self_id();
        //TODO verify module compatibility
        let _verified_module = VerifiedModule::new(compiled_module).map_err(|(_, e)| e)?;
        //TODO check native implement.
        //Loader::check_natives(&verified_module)?;
        data_store.publish_module(module_id, code)
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_prologue(
        &self,
        data_store: &mut TransactionDataCache,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let gas_currency_ty = DEFAULT_CURRENCY_TY.clone();
        let txn_sequence_number = txn_data.sequence_number();
        let txn_public_key = txn_data.authentication_key_preimage().to_vec();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let txn_expiration_time = txn_data.expiration_time();
        self.move_vm
            .execute_function(
                &account_config::ACCOUNT_MODULE,
                &PROLOGUE_NAME,
                vec![gas_currency_ty],
                vec![
                    Value::transaction_argument_signer_reference(txn_data.sender),
                    Value::u64(txn_sequence_number),
                    Value::vector_u8(txn_public_key),
                    Value::u64(txn_gas_price),
                    Value::u64(txn_max_gas_units),
                    Value::u64(txn_expiration_time),
                ],
                txn_data.sender(),
                data_store,
                cost_strategy,
            )
            .map_err(|err| convert_prologue_runtime_error(&err, &txn_data.sender))
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_epilogue(
        &self,
        data_store: &mut TransactionDataCache,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let gas_currency_ty = DEFAULT_CURRENCY_TY.clone();
        let txn_sequence_number = txn_data.sequence_number();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let gas_remaining = cost_strategy.remaining_gas().get();
        self.move_vm.execute_function(
            &account_config::ACCOUNT_MODULE,
            &EPILOGUE_NAME,
            vec![gas_currency_ty],
            vec![
                Value::transaction_argument_signer_reference(txn_data.sender),
                Value::u64(txn_sequence_number),
                Value::u64(txn_gas_price),
                Value::u64(txn_max_gas_units),
                Value::u64(gas_remaining),
            ],
            txn_data.sender(),
            data_store,
            cost_strategy,
        )
    }

    fn process_block_metadata(
        &self,
        remote_cache: &mut StateViewCache<'_>,
        block_metadata: BlockMetadata,
    ) -> VMResult<TransactionOutput> {
        let mut txn_data = TransactionMetadata::default();
        //TODO reconsider sender address
        txn_data.sender = account_config::mint_address();
        txn_data.max_gas_amount = GasUnits::new(std::u64::MAX);

        let gas_schedule = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::transaction(&gas_schedule, txn_data.max_gas_amount());
        let mut data_store = TransactionDataCache::new(remote_cache);

        let (parent_id, timestamp, author, auth) = block_metadata.into_inner();
        let vote_maps = vec![];
        let round = 0u64;
        let args = vec![
            Value::transaction_argument_signer_reference(account_config::mint_address()),
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
            vec![],
            args,
            txn_data.sender(),
            &mut data_store,
            &mut cost_strategy,
        )?;

        get_transaction_output(
            &mut data_store,
            &cost_strategy,
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
        remote_cache: &mut StateViewCache<'_>,
    ) -> TransactionOutput {
        let txn_data = TransactionMetadata::new(&txn);

        // check signature
        let signature_checked_txn = match txn.check_signature() {
            Ok(t) => Ok(t),
            Err(_) => Err(VMStatus::new(StatusCode::INVALID_SIGNATURE)),
        };

        match signature_checked_txn {
            Ok(txn) => {
                let verified_payload = self.verify_transaction_impl(&txn, remote_cache, &txn_data);
                match verified_payload {
                    Ok(payload) => self.execute_verified_payload(remote_cache, &txn_data, payload),
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
        let mut data_cache = StateViewCache::new(state_view);

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

    /// Generates a transaction output for a transaction that encountered errors during the
    /// execution process. This is public for now only for tests.
    pub fn failed_transaction_cleanup(
        &self,
        error_code: VMStatus,
        gas_schedule: &CostTable,
        gas_left: GasUnits<GasCarrier>,
        txn_data: &TransactionMetadata,
        remote_cache: &mut StateViewCache<'_>,
    ) -> TransactionOutput {
        let mut cost_strategy = CostStrategy::system(gas_schedule, gas_left);
        let mut data_store = TransactionDataCache::new(remote_cache);
        match TransactionStatus::from(error_code) {
            TransactionStatus::Keep(status) => self
                .run_epilogue(&mut data_store, &mut cost_strategy, txn_data)
                .and_then(|_| {
                    get_transaction_output(&mut data_store, &cost_strategy, txn_data, status)
                })
                .unwrap_or_else(discard_error_output),
            TransactionStatus::Discard(status) => discard_error_output(status),
        }
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

fn convert_txn_args(args: &[TransactionArgument]) -> Vec<Value> {
    args.iter()
        .map(|arg| match arg {
            TransactionArgument::U8(i) => Value::u8(*i),
            TransactionArgument::U64(i) => Value::u64(*i),
            TransactionArgument::U128(i) => Value::u128(*i),
            TransactionArgument::Address(a) => Value::address(*a),
            TransactionArgument::Bool(b) => Value::bool(*b),
            TransactionArgument::U8Vector(v) => Value::vector_u8(v.clone()),
        })
        .collect()
}

fn get_transaction_output(
    data_store: &mut TransactionDataCache,
    cost_strategy: &CostStrategy,
    txn_data: &TransactionMetadata,
    status: VMStatus,
) -> VMResult<TransactionOutput> {
    let gas_used: u64 = txn_data
        .max_gas_amount()
        .sub(cost_strategy.remaining_gas())
        .get();
    let write_set = data_store.make_write_set()?;
    //TODO add gas usage metrics.
    Ok(TransactionOutput::new(
        write_set,
        data_store.event_data().to_vec(),
        gas_used,
        TransactionStatus::Keep(status),
    ))
}

pub enum VerifiedTransactionPayload {
    Script(Vec<u8>, Vec<TypeTag>, Vec<Value>),
    Module(Vec<u8>),
    Package(UpgradePackage),
}
