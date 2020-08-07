// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path_cache::AccessPathCache;
use crate::data_cache::{RemoteStorage, StateViewCache};
use crate::metrics::TXN_EXECUTION_GAS_USAGE;
use anyhow::{format_err, Error, Result};
use move_vm_runtime::data_cache::TransactionEffects;
use move_vm_runtime::session::Session;
use move_vm_runtime::{data_cache::RemoteCache, move_vm::MoveVM};
use once_cell::sync::Lazy;
use starcoin_logger::prelude::*;
use starcoin_move_compiler::check_compat_and_verify_module;
use starcoin_types::{
    account_config,
    block_metadata::BlockMetadata,
    transaction::{
        SignatureCheckedTransaction, SignedUserTransaction, Transaction, TransactionArgument,
        TransactionOutput, TransactionPayload, TransactionStatus,
    },
    write_set::WriteSet,
};
use starcoin_vm_types::access::ModuleAccess;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::{
    genesis_address, stc_type_tag, EPILOGUE_NAME, PROLOGUE_NAME,
};
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::gas_schedule::{zero_cost_schedule, CostStrategy};
use starcoin_vm_types::on_chain_config::{VMPublishingOption, INITIAL_GAS_SCHEDULE};
use starcoin_vm_types::transaction::{Package, Script, TransactionPayloadType};
use starcoin_vm_types::transaction_metadata::TransactionPayloadMetadata;
use starcoin_vm_types::vm_status::KeptVMStatus;
use starcoin_vm_types::write_set::{WriteOp, WriteSetMut};
use starcoin_vm_types::{
    errors::{self, IndexKind, Location},
    event::EventKey,
    gas_schedule::{self, CostTable, GasAlgebra, GasCarrier, GasUnits},
    language_storage::TypeTag,
    on_chain_config::{OnChainConfig, VMConfig, Version},
    state_view::StateView,
    transaction_metadata::TransactionMetadata,
    values::Value,
    vm_status::{StatusCode, VMStatus},
};
use std::convert::TryFrom;
use std::sync::Arc;

//// The value should be tuned carefully
//pub static MAXIMUM_NUMBER_OF_GAS_UNITS: Lazy<GasUnits<GasCarrier>> =
//    Lazy::new(|| GasUnits::new(100_000_000));

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

    fn load_configs(&mut self, state: &dyn StateView) -> Result<(), Error> {
        if state.is_genesis() {
            self.vm_config = Some(VMConfig {
                publishing_option: VMPublishingOption::Open,
                gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
                block_gas_limit: u64::MAX, //no gas limitation on genesis
            });
            self.version = Some(Version { major: 0 });
            Ok(())
        } else {
            self.load_configs_impl(state)
        }
    }

    fn vm_config(&self) -> Result<&VMConfig, VMStatus> {
        self.vm_config
            .as_ref()
            .ok_or_else(|| VMStatus::Error(StatusCode::VM_STARTUP_FAILURE))
    }

    fn load_configs_impl(&mut self, state: &dyn StateView) -> Result<(), Error> {
        self.vm_config = Some(
            VMConfig::fetch_config(RemoteStorage::new(state))?
                .ok_or_else(|| format_err!("Load VMConfig fail, VMConfig resource not exist."))?,
        );

        self.version = Some(
            Version::fetch_config(RemoteStorage::new(state))?
                .ok_or_else(|| format_err!("Load Version fail, Version resource not exist."))?,
        );

        Ok(())
    }

    pub fn get_gas_schedule(&self) -> Result<&CostTable, VMStatus> {
        self.vm_config
            .as_ref()
            .map(|config| &config.gas_schedule)
            .ok_or_else(|| VMStatus::Error(StatusCode::VM_STARTUP_FAILURE))
    }

    pub fn get_version(&self) -> Result<Version, VMStatus> {
        self.version
            .clone()
            .ok_or_else(|| VMStatus::Error(StatusCode::VM_STARTUP_FAILURE))
    }

    pub fn get_block_gas_limit(&self) -> Result<&u64, VMStatus> {
        self.vm_config
            .as_ref()
            .map(|config| &config.block_gas_limit)
            .ok_or_else(|| VMStatus::Error(StatusCode::VM_STARTUP_FAILURE))
    }

    fn check_gas(&self, txn_data: &TransactionMetadata) -> Result<(), VMStatus> {
        let gas_constants = &self.get_gas_schedule()?.gas_constants;
        let raw_bytes_len = txn_data.transaction_size;
        // The transaction is too large.
        if raw_bytes_len.get() > gas_constants.max_transaction_size_in_bytes {
            warn!(
                "[VM] Transaction size too big {} (max {})",
                raw_bytes_len.get(),
                gas_constants.max_transaction_size_in_bytes
            );
            return Err(VMStatus::Error(StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE));
        }

        // Check is performed on `txn.raw_txn_bytes_len()` which is the same as
        // `raw_bytes_len`
        assert!(raw_bytes_len.get() <= gas_constants.max_transaction_size_in_bytes);

        // The submitted max gas units that the transaction can consume is greater than the
        // maximum number of gas units bound that we have set for any
        // transaction.
        if txn_data.max_gas_amount().get() > gas_constants.maximum_number_of_gas_units.get() {
            warn!(
                "[VM] Gas unit error; max {}, submitted {}",
                gas_constants.maximum_number_of_gas_units.get(),
                txn_data.max_gas_amount().get()
            );
            return Err(VMStatus::Error(
                StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND,
            ));
        }

        // The submitted transactions max gas units needs to be at least enough to cover the
        // intrinsic cost of the transaction as calculated against the size of the
        // underlying `RawTransaction`
        let min_txn_fee = gas_schedule::calculate_intrinsic_gas(raw_bytes_len, gas_constants);
        if txn_data.max_gas_amount().get() < min_txn_fee.get() {
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                min_txn_fee.get(),
                txn_data.max_gas_amount().get()
            );
            return Err(VMStatus::Error(
                StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS,
            ));
        }

        // The submitted gas price is less than the minimum gas unit price set by the VM.
        // NB: MIN_PRICE_PER_GAS_UNIT may equal zero, but need not in the future. Hence why
        // we turn off the clippy warning.
        #[allow(clippy::absurd_extreme_comparisons)]
        let below_min_bound =
            txn_data.gas_unit_price().get() < gas_constants.min_price_per_gas_unit.get();
        if below_min_bound {
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                gas_constants.min_price_per_gas_unit.get(),
                txn_data.gas_unit_price().get()
            );
            return Err(VMStatus::Error(StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND));
        }

        // The submitted gas price is greater than the maximum gas unit price set by the VM.
        if txn_data.gas_unit_price().get() > gas_constants.max_price_per_gas_unit.get() {
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                gas_constants.max_price_per_gas_unit.get(),
                txn_data.gas_unit_price().get()
            );
            return Err(VMStatus::Error(StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND));
        }
        Ok(())
    }

    fn verify_transaction_impl(
        &mut self,
        transaction: &SignatureCheckedTransaction,
        remote_cache: &StateViewCache,
    ) -> Result<(), VMStatus> {
        let txn_data = TransactionMetadata::new(transaction);
        let mut session = self.move_vm.new_session(remote_cache);
        let mut cost_strategy = CostStrategy::system(self.get_gas_schedule()?, GasUnits::new(0));
        self.check_gas(&txn_data)?;
        match transaction.payload() {
            TransactionPayload::Script(script) => {
                self.is_allowed_script(script)?;
            }
            TransactionPayload::Package(_package) => {
                //TODO move to prologue
                if !&self.vm_config()?.publishing_option.is_open() {
                    warn!("[VM] Custom modules not allowed");
                    return Err(VMStatus::Error(StatusCode::UNKNOWN_MODULE));
                };
                //TODO verify module compat.
            }
        }
        self.run_prologue(&mut session, &mut cost_strategy, &txn_data)
    }

    pub fn verify_transaction(
        &mut self,
        state_view: &dyn StateView,
        txn: SignedUserTransaction,
    ) -> Option<VMStatus> {
        let data_cache = StateViewCache::new(state_view);
        let signature_verified_txn = match txn.check_signature() {
            Ok(t) => t,
            Err(_) => return Some(VMStatus::Error(StatusCode::INVALID_SIGNATURE)),
        };
        if let Err(err) = self.load_configs(state_view) {
            warn!("Load config error at verify_transaction: {}", err);
            return Some(VMStatus::Error(StatusCode::VM_STARTUP_FAILURE));
        }
        match self.verify_transaction_impl(&signature_verified_txn, &data_cache) {
            Ok(_) => None,
            Err(err) => {
                if err.status_code() == StatusCode::SEQUENCE_NUMBER_TOO_NEW {
                    None
                } else {
                    Some(err)
                }
            }
        }
    }

    fn execute_package(
        &self,
        remote_cache: &StateViewCache<'_>,
        gas_schedule: &CostTable,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
        package: &Package,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let mut session = self.move_vm.new_session(remote_cache);

        {
            // Run the validation logic
            cost_strategy.disable_metering();
            // genesis txn skip check gas and txn prologue.
            if !remote_cache.is_genesis() {
                //let _timer = TXN_VERIFICATION_SECONDS.start_timer();
                self.check_gas(txn_data)?;
                self.run_prologue(&mut session, cost_strategy, &txn_data)?;
            }
        }
        {
            // Genesis txn not enable gas charge.
            if !remote_cache.is_genesis() {
                cost_strategy.enable_metering();
            }
            cost_strategy
                .charge_intrinsic_gas(txn_data.transaction_size())
                .map_err(|e| e.into_vm_status())?;

            let package_address = package.package_address();
            for module in package.modules() {
                let compiled_module = match CompiledModule::deserialize(module.code()) {
                    Ok(module) => module,
                    Err(err) => {
                        warn!("[VM] module deserialization failed {:?}", err);
                        return Err(err.finish(Location::Undefined).into_vm_status());
                    }
                };

                let module_id = compiled_module.self_id();
                if module_id.address() != &package_address {
                    return Err(errors::verification_error(
                        //TODO define new error code.
                        StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER,
                        IndexKind::AddressIdentifier,
                        compiled_module.self_handle_idx().0,
                    )
                    .finish(Location::Undefined)
                    .into_vm_status());
                }

                //verify module compatibility
                let _new_module = if session
                    .exists_module(&module_id)
                    .map_err(|e| e.into_vm_status())?
                {
                    let pre_version = session
                        .load_module(&module_id)
                        .map_err(|e| e.into_vm_status())?;
                    check_compat_and_verify_module(pre_version.as_slice(), module.code()).map_err(
                        |e| {
                            {
                                warn!("Check module compat error: {:?}", e);
                                errors::verification_error(
                                    //TODO define error code for compat.
                                    StatusCode::VERIFICATION_ERROR,
                                    IndexKind::ModuleHandle,
                                    compiled_module.self_handle_idx().0,
                                )
                            }
                            .finish(Location::Undefined)
                            .into_vm_status()
                        },
                    )?
                } else {
                    session
                        .verify_module(module.code())
                        .map_err(|e| e.into_vm_status())?
                };
                session
                    .publish_module(module.code().to_vec(), txn_data.sender, cost_strategy)
                    .map_err(|e| e.into_vm_status())?;
            }
            if let Some(init_script) = package.init_script() {
                let sender = txn_data.sender;
                let ty_args = init_script.ty_args().to_vec();
                let args = convert_txn_args(init_script.args());
                let s = init_script.code().to_vec();
                debug!("execute init script by account {:?}", sender);
                session
                    .execute_script(s, ty_args, args, sender, cost_strategy)
                    .map_err(|e| e.into_vm_status())?
            }
            charge_global_write_gas_usage(cost_strategy, &session)?;

            cost_strategy.disable_metering();
            self.success_transaction_cleanup(
                session,
                gas_schedule,
                cost_strategy.remaining_gas(),
                txn_data,
            )
        }
    }

    fn execute_script(
        &self,
        remote_cache: &StateViewCache<'_>,
        gas_schedule: &CostTable,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
        script: &Script,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let mut session = self.move_vm.new_session(remote_cache);

        // Run the validation logic
        {
            cost_strategy.disable_metering();
            //let _timer = TXN_VERIFICATION_SECONDS.start_timer();
            self.check_gas(txn_data)?;
            self.is_allowed_script(script)?;
            self.run_prologue(&mut session, cost_strategy, &txn_data)?;
        }

        // Run the execution logic
        {
            //let _timer = TXN_EXECUTION_SECONDS.start_timer();
            cost_strategy.enable_metering();
            cost_strategy
                .charge_intrinsic_gas(txn_data.transaction_size())
                .map_err(|e| e.into_vm_status())?;
            session
                .execute_script(
                    script.code().to_vec(),
                    script.ty_args().to_vec(),
                    convert_txn_args(script.args()),
                    txn_data.sender(),
                    cost_strategy,
                )
                .map_err(|e| e.into_vm_status())?;

            charge_global_write_gas_usage(cost_strategy, &session)?;

            cost_strategy.disable_metering();
            self.success_transaction_cleanup(
                session,
                gas_schedule,
                cost_strategy.remaining_gas(),
                txn_data,
            )
        }
    }

    fn is_allowed_script(&self, script: &Script) -> Result<(), VMStatus> {
        if !self
            .vm_config()?
            .publishing_option
            .is_allowed_script(&script.code())
        {
            warn!("[VM] Custom scripts not allowed: {:?}", &script.code());
            Err(VMStatus::Error(StatusCode::UNKNOWN_SCRIPT))
        } else {
            Ok(())
        }
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_prologue<R: RemoteCache>(
        &self,
        session: &mut Session<R>,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
    ) -> Result<(), VMStatus> {
        let genesis_address = genesis_address();
        let gas_token_ty = txn_data.gas_token_code().into();
        let txn_sequence_number = txn_data.sequence_number();
        let txn_public_key = txn_data.authentication_key_preimage().to_vec();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_amount = txn_data.max_gas_amount().get();
        let txn_expiration_time = txn_data.expiration_time_secs();
        let chain_id = txn_data.chain_id().id();
        let (payload_type, script_or_package_hash, package_address) = match txn_data.payload() {
            TransactionPayloadMetadata::Script(hash) => {
                (TransactionPayloadType::Script, *hash, AccountAddress::ZERO)
            }
            TransactionPayloadMetadata::Package(hash, package_address) => {
                (TransactionPayloadType::Package, *hash, *package_address)
            }
        };

        // Run prologue by genesis account
        session
            .execute_function(
                &account_config::TRANSACTION_MANAGER_MODULE,
                &PROLOGUE_NAME,
                vec![gas_token_ty],
                vec![
                    Value::transaction_argument_signer_reference(genesis_address),
                    Value::address(txn_data.sender),
                    Value::u64(txn_sequence_number),
                    Value::vector_u8(txn_public_key),
                    Value::u64(txn_gas_price),
                    Value::u64(txn_max_gas_amount),
                    Value::u64(txn_expiration_time),
                    Value::u8(chain_id),
                    Value::u8(payload_type.into()),
                    Value::vector_u8(script_or_package_hash.to_vec()),
                    Value::address(package_address),
                ],
                genesis_address,
                cost_strategy,
            )
            .map_err(|err| convert_prologue_runtime_error(err.into_vm_status()))
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_epilogue<R: RemoteCache>(
        &self,
        session: &mut Session<R>,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
        success: bool,
    ) -> Result<(), VMStatus> {
        let genesis_address = genesis_address();
        let gas_token_ty = txn_data.gas_token_code().into();
        let txn_sequence_number = txn_data.sequence_number();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_amount = txn_data.max_gas_amount().get();
        let gas_remaining = cost_strategy.remaining_gas().get();
        let data_size = 0i64; //data_store.get_size(txn_data.sender);
        let cost_is_negative = data_size.is_negative();
        let state_cost_amount = data_size.wrapping_abs() as u64;
        let (payload_type, script_or_package_hash, package_address) = match txn_data.payload() {
            TransactionPayloadMetadata::Script(hash) => {
                (TransactionPayloadType::Script, *hash, AccountAddress::ZERO)
            }
            TransactionPayloadMetadata::Package(hash, package_address) => {
                (TransactionPayloadType::Package, *hash, *package_address)
            }
        };
        // Run epilogue by genesis account
        session
            .execute_function(
                &account_config::TRANSACTION_MANAGER_MODULE,
                &EPILOGUE_NAME,
                vec![gas_token_ty],
                vec![
                    Value::transaction_argument_signer_reference(genesis_address),
                    Value::address(txn_data.sender),
                    Value::u64(txn_sequence_number),
                    Value::u64(txn_gas_price),
                    Value::u64(txn_max_gas_amount),
                    Value::u64(gas_remaining),
                    Value::u64(state_cost_amount),
                    Value::bool(cost_is_negative),
                    Value::u8(payload_type.into()),
                    Value::vector_u8(script_or_package_hash.to_vec()),
                    Value::address(package_address),
                    Value::bool(success),
                ],
                genesis_address,
                cost_strategy,
            )
            .map_err(|err| err.into_vm_status())
    }

    fn process_block_metadata(
        &self,
        remote_cache: &mut StateViewCache<'_>,
        block_metadata: BlockMetadata,
    ) -> Result<TransactionOutput, VMStatus> {
        let mut txn_data = TransactionMetadata::default();
        //process block metadata by genesis.
        txn_data.sender = account_config::genesis_address();

        let gas_schedule = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&gas_schedule, txn_data.max_gas_amount());

        let (parent_id, timestamp, author, auth, uncles, number) = block_metadata.into_inner();
        let args = vec![
            Value::transaction_argument_signer_reference(txn_data.sender),
            Value::vector_u8(parent_id.to_vec()),
            Value::u64(timestamp),
            Value::address(author),
            match auth {
                Some(prefix) => Value::vector_u8(prefix),
                None => Value::vector_u8(Vec::new()),
            },
            Value::u64(uncles),
            Value::u64(number),
        ];
        let mut session = self.move_vm.new_session(remote_cache);
        session
            .execute_function(
                &account_config::TRANSACTION_MANAGER_MODULE,
                &account_config::BLOCK_PROLOGUE_NAME,
                vec![],
                args,
                txn_data.sender(),
                &mut cost_strategy,
            )
            .map_err(|err| convert_prologue_runtime_error(err.into_vm_status()))?;
        Ok(get_transaction_output(
            &mut (),
            session,
            &cost_strategy,
            &txn_data,
            KeptVMStatus::Executed,
        )?)
    }

    fn execute_user_transaction(
        &mut self,
        txn: SignedUserTransaction,
        remote_cache: &mut StateViewCache<'_>,
    ) -> (VMStatus, TransactionOutput) {
        let gas_schedule = match self.get_gas_schedule() {
            Ok(gas_schedule) => gas_schedule,
            Err(e) => {
                if remote_cache.is_genesis() {
                    &INITIAL_GAS_SCHEDULE
                } else {
                    return discard_error_vm_status(e);
                }
            }
        };
        let txn_data = TransactionMetadata::new(&txn);
        let mut cost_strategy = CostStrategy::system(gas_schedule, txn_data.max_gas_amount());
        // check signature
        let signature_checked_txn = match txn.check_signature() {
            Ok(t) => Ok(t),
            Err(_) => Err(VMStatus::Error(StatusCode::INVALID_SIGNATURE)),
        };

        match signature_checked_txn {
            Ok(txn) => {
                let result = match txn.payload() {
                    TransactionPayload::Script(s) => self.execute_script(
                        remote_cache,
                        gas_schedule,
                        &mut cost_strategy,
                        &txn_data,
                        s,
                    ),
                    TransactionPayload::Package(p) => self.execute_package(
                        remote_cache,
                        gas_schedule,
                        &mut cost_strategy,
                        &txn_data,
                        p,
                    ),
                };
                match result {
                    Ok(status_and_output) => status_and_output,
                    Err(err) => {
                        let txn_status = TransactionStatus::from(err.clone());
                        if txn_status.is_discarded() {
                            discard_error_vm_status(err)
                        } else {
                            self.failed_transaction_cleanup(
                                err,
                                gas_schedule,
                                cost_strategy.remaining_gas(),
                                &txn_data,
                                remote_cache,
                            )
                        }
                    }
                }
            }
            Err(e) => discard_error_vm_status(e),
        }
    }

    /// Execute a block transactions with gas_limit,
    /// if gas is used up when executing some txn, only return the outputs of previous succeed txns.
    pub fn execute_block_transactions(
        &mut self,
        state_view: &dyn StateView,
        transactions: Vec<Transaction>,
        block_gas_limit: Option<u64>,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>> {
        let mut data_cache = StateViewCache::new(state_view);
        let mut result = vec![];
        //TODO load config by config change event.
        self.load_configs(&data_cache)?;

        // get_block_gas_limit should be called after load_configs()
        let default_block_gas_limit = *self.get_block_gas_limit()?;
        let mut gas_left = default_block_gas_limit;
        if block_gas_limit.is_some() {
            gas_left = std::cmp::min(default_block_gas_limit, block_gas_limit.unwrap_or_default());
        }

        let blocks = chunk_block_transactions(transactions);
        'outer: for block in blocks {
            match block {
                TransactionBlock::UserTransaction(txns) => {
                    for transaction in txns {
                        let gas_unit_price = transaction.gas_unit_price();
                        let (status, output) =
                            self.execute_user_transaction(transaction, &mut data_cache);
                        // only need to check for user transactions.
                        match gas_left.checked_sub(output.gas_used()) {
                            Some(l) => gas_left = l,
                            None => break 'outer,
                        }

                        if let TransactionStatus::Keep(_) = output.status() {
                            if gas_unit_price > 0 {
                                debug_assert_ne!(
                                    output.gas_used(),
                                    0,
                                    "Keep transaction gas used must not be zero"
                                );
                            }
                            data_cache.push_write_set(output.write_set())
                        }
                        result.push((status, output));
                    }
                }
                TransactionBlock::BlockPrologue(block_metadata) => {
                    let (status, output) =
                        match self.process_block_metadata(&mut data_cache, block_metadata) {
                            Ok(output) => (VMStatus::Executed, output),
                            Err(vm_status) => discard_error_vm_status(vm_status),
                        };
                    debug_assert_eq!(
                        output.gas_used(),
                        0,
                        "Block metadata transaction gas_used must be zero."
                    );
                    if let TransactionStatus::Keep(status) = output.status() {
                        debug_assert_eq!(
                            status,
                            &KeptVMStatus::Executed,
                            "Block metadata transaction keep status must been Executed."
                        );
                        data_cache.push_write_set(output.write_set())
                    }
                    result.push((status, output));
                }
            }
        }
        Ok(result)
    }

    pub fn execute_transactions(
        &mut self,
        state_view: &dyn StateView,
        transactions: Vec<Transaction>,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>> {
        self.execute_block_transactions(state_view, transactions, None)
    }

    fn success_transaction_cleanup<R: RemoteCache>(
        &self,
        mut session: Session<R>,
        gas_schedule: &CostTable,
        gas_left: GasUnits<GasCarrier>,
        txn_data: &TransactionMetadata,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let mut cost_strategy = CostStrategy::system(gas_schedule, gas_left);
        self.run_epilogue(&mut session, &mut cost_strategy, txn_data, true)?;

        Ok((
            VMStatus::Executed,
            get_transaction_output(
                &mut (),
                session,
                &cost_strategy,
                txn_data,
                KeptVMStatus::Executed,
            )?,
        ))
    }

    fn failed_transaction_cleanup(
        &self,
        error_code: VMStatus,
        gas_schedule: &CostTable,
        gas_left: GasUnits<GasCarrier>,
        txn_data: &TransactionMetadata,
        remote_cache: &StateViewCache<'_>,
    ) -> (VMStatus, TransactionOutput) {
        let mut cost_strategy = CostStrategy::system(gas_schedule, gas_left);
        let mut session = self.move_vm.new_session(remote_cache);

        // init_script doesn't need run epilogue
        if remote_cache.is_genesis() {
            return discard_error_vm_status(error_code);
        }

        match TransactionStatus::from(error_code.clone()) {
            TransactionStatus::Keep(status) => {
                if let Err(e) = self.run_epilogue(&mut session, &mut cost_strategy, txn_data, false)
                {
                    return discard_error_vm_status(e);
                }
                let txn_output =
                    get_transaction_output(&mut (), session, &cost_strategy, txn_data, status)
                        .unwrap_or_else(|e| discard_error_vm_status(e).1);
                (error_code, txn_output)
            }
            TransactionStatus::Discard(status) => {
                (VMStatus::Error(status), discard_error_output(status))
            }
        }
    }
}

pub enum TransactionBlock {
    UserTransaction(Vec<SignedUserTransaction>),
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

pub(crate) fn charge_global_write_gas_usage<R: RemoteCache>(
    cost_strategy: &mut CostStrategy,
    session: &Session<R>,
) -> Result<(), VMStatus> {
    let total_cost = session.num_mutated_accounts()
        * cost_strategy
            .cost_table()
            .gas_constants
            .global_memory_per_byte_write_cost
            .mul(
                cost_strategy
                    .cost_table()
                    .gas_constants
                    .default_account_size,
            )
            .get();
    cost_strategy
        .deduct_gas(GasUnits::new(total_cost))
        .map_err(|p_err| p_err.finish(Location::Undefined).into_vm_status())
}

pub(crate) fn discard_error_vm_status(err: VMStatus) -> (VMStatus, TransactionOutput) {
    let vm_status = err.clone();
    let error_code = match err.keep_or_discard() {
        Ok(_) => {
            debug_assert!(false, "discarding non-discardable error: {:?}", vm_status);
            vm_status.status_code()
        }
        Err(code) => code,
    };
    (vm_status, discard_error_output(error_code))
}

pub(crate) fn discard_error_output(err: StatusCode) -> TransactionOutput {
    info!("discard error output: {:?}", err);
    // Since this transaction will be discarded, no writeset will be included.
    TransactionOutput::new(
        WriteSet::default(),
        vec![],
        0,
        0,
        TransactionStatus::Discard(err),
    )
}

pub fn convert_txn_args(args: &[TransactionArgument]) -> Vec<Value> {
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

pub fn txn_effects_to_writeset_and_events_cached<C: AccessPathCache>(
    ap_cache: &mut C,
    effects: TransactionEffects,
) -> Result<(WriteSet, Vec<ContractEvent>), VMStatus> {
    // TODO: Cache access path computations if necessary.
    let mut ops = vec![];

    for (addr, vals) in effects.resources {
        for (ty_tag, val_opt) in vals {
            let struct_tag = match ty_tag {
                TypeTag::Struct(struct_tag) => struct_tag,
                _ => return Err(VMStatus::Error(StatusCode::VALUE_SERIALIZATION_ERROR)),
            };
            let ap = ap_cache.get_resource_path(addr, struct_tag);
            let op = match val_opt {
                None => WriteOp::Deletion,
                Some((ty_layout, val)) => {
                    let blob = val
                        .simple_serialize(&ty_layout)
                        .ok_or_else(|| VMStatus::Error(StatusCode::VALUE_SERIALIZATION_ERROR))?;

                    WriteOp::Value(blob)
                }
            };
            ops.push((ap, op))
        }
    }

    for (module_id, blob) in effects.modules {
        ops.push((ap_cache.get_module_path(module_id), WriteOp::Value(blob)))
    }

    let ws = WriteSetMut::new(ops)
        .freeze()
        .map_err(|_| VMStatus::Error(StatusCode::DATA_FORMAT_ERROR))?;

    let events = effects
        .events
        .into_iter()
        .map(|(guid, seq_num, ty_tag, ty_layout, val)| {
            let msg = val
                .simple_serialize(&ty_layout)
                .ok_or_else(|| VMStatus::Error(StatusCode::DATA_FORMAT_ERROR))?;
            let key = EventKey::try_from(guid.as_slice())
                .map_err(|_| VMStatus::Error(StatusCode::EVENT_KEY_MISMATCH))?;
            Ok(ContractEvent::new(key, seq_num, ty_tag, msg))
        })
        .collect::<Result<Vec<_>, VMStatus>>()?;

    Ok((ws, events))
}

pub(crate) fn get_transaction_output<A: AccessPathCache, R: RemoteCache>(
    ap_cache: &mut A,
    session: Session<R>,
    cost_strategy: &CostStrategy,
    txn_data: &TransactionMetadata,
    status: KeptVMStatus,
) -> Result<TransactionOutput, VMStatus> {
    let gas_used: u64 = txn_data
        .max_gas_amount()
        .sub(cost_strategy.remaining_gas())
        .get();

    let effects = session.finish().map_err(|e| e.into_vm_status())?;
    let (write_set, events) = txn_effects_to_writeset_and_events_cached(ap_cache, effects)?;

    TXN_EXECUTION_GAS_USAGE.observe(gas_used as f64);

    Ok(TransactionOutput::new(
        write_set,
        events,
        gas_used,
        0,
        TransactionStatus::Keep(status),
    ))
}

pub fn txn_effects_to_writeset_and_events(
    effects: TransactionEffects,
) -> Result<(WriteSet, Vec<ContractEvent>), VMStatus> {
    txn_effects_to_writeset_and_events_cached(&mut (), effects)
}

pub enum VerifiedTransactionPayload {
    Script(Vec<u8>, Vec<TypeTag>, Vec<Value>),
    Package(Package),
}

//should be consistent with ErrorCode.move
const PROLOGUE_ACCOUNT_DOES_NOT_EXIST: u64 = 0;
const PROLOGUE_INVALID_ACCOUNT_AUTH_KEY: u64 = 1;
const PROLOGUE_SEQUENCE_NUMBER_TOO_OLD: u64 = 2;
const PROLOGUE_SEQUENCE_NUMBER_TOO_NEW: u64 = 3;
const PROLOGUE_CANT_PAY_GAS_DEPOSIT: u64 = 4;
const PROLOGUE_TRANSACTION_EXPIRED: u64 = 5;
const PROLOGUE_BAD_CHAIN_ID: u64 = 6;

const ENOT_GENESIS_ACCOUNT: u64 = 11;
// Todo: haven't found proper StatusCode for below Error Code
const ENOT_GENESIS: u64 = 12;
const ECONFIG_VALUE_DOES_NOT_EXIST: u64 = 13;
const EINVALID_TIMESTAMP: u64 = 14;
const ECOIN_DEPOSIT_IS_ZERO: u64 = 15;
const EDESTORY_TOKEN_NON_ZERO: u64 = 16;
const EBLOCK_NUMBER_MISMATCH: u64 = 17;

fn convert_prologue_runtime_error(status: VMStatus) -> VMStatus {
    match status {
        VMStatus::MoveAbort(_location, code) => {
            warn!("prologue error: {:?}", code);
            let new_major_status = match code {
                PROLOGUE_ACCOUNT_DOES_NOT_EXIST => StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST,
                PROLOGUE_INVALID_ACCOUNT_AUTH_KEY => StatusCode::INVALID_AUTH_KEY,
                PROLOGUE_SEQUENCE_NUMBER_TOO_OLD => StatusCode::SEQUENCE_NUMBER_TOO_OLD,
                PROLOGUE_SEQUENCE_NUMBER_TOO_NEW => StatusCode::SEQUENCE_NUMBER_TOO_NEW,
                PROLOGUE_CANT_PAY_GAS_DEPOSIT => {
                    StatusCode::INSUFFICIENT_BALANCE_FOR_TRANSACTION_FEE
                }
                PROLOGUE_TRANSACTION_EXPIRED => StatusCode::TRANSACTION_EXPIRED,
                PROLOGUE_BAD_CHAIN_ID => StatusCode::BAD_CHAIN_ID,
                ENOT_GENESIS_ACCOUNT => StatusCode::NO_ACCOUNT_ROLE,
                ENOT_GENESIS => StatusCode::UNKNOWN_STATUS,
                ECONFIG_VALUE_DOES_NOT_EXIST => StatusCode::UNKNOWN_STATUS,
                EINVALID_TIMESTAMP => StatusCode::UNKNOWN_STATUS,
                ECOIN_DEPOSIT_IS_ZERO => StatusCode::UNKNOWN_STATUS,
                EDESTORY_TOKEN_NON_ZERO => StatusCode::UNKNOWN_STATUS,
                EBLOCK_NUMBER_MISMATCH => StatusCode::UNKNOWN_STATUS,
                // ToDo add corresponding error code into StatusCode
                _ => StatusCode::UNKNOWN_STATUS,
            };
            VMStatus::Error(new_major_status)
        }
        status @ VMStatus::ExecutionFailure { .. } | status @ VMStatus::Executed => status,
        VMStatus::Error(code) => {
            warn!("[libra_vm] Unexpected prologue error: {:?}", code);
            VMStatus::Error(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR) //ToDo: replace with UNEXPECTED_ERROR_FROM_KNOWN_MOVE_FUNCTION
        }
    }
}
