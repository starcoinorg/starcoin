// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path_cache::AccessPathCache;
use crate::data_cache::{RemoteStorage, StateViewCache};
use crate::errors::{
    convert_normal_success_epilogue_error, convert_prologue_runtime_error, error_split,
};
use crate::metrics::{BLOCK_UNCLES, TXN_EXECUTION_GAS_USAGE};
use anyhow::{format_err, Error, Result};
use crypto::HashValue;
use move_vm_runtime::data_cache::RemoteCache;
use move_vm_runtime::move_vm_adapter::{MoveVMAdapter, SessionAdapter};
use starcoin_config::INITIAL_GAS_SCHEDULE;
use starcoin_logger::prelude::*;
use starcoin_move_compiler::check_module_compat;
use starcoin_types::account_config::{
    access_path_for_module_upgrade_strategy, access_path_for_two_phase_upgrade_v2,
};
use starcoin_types::{
    account_config,
    block_metadata::BlockMetadata,
    transaction::{
        SignatureCheckedTransaction, SignedUserTransaction, Transaction, TransactionOutput,
        TransactionPayload, TransactionStatus,
    },
    write_set::WriteSet,
};
use starcoin_vm_types::access::ModuleAccess;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::{
    genesis_address, ModuleUpgradeStrategy, TwoPhaseUpgradeV2Resource, EPILOGUE_NAME,
    EPILOGUE_V2_NAME, PROLOGUE_NAME,
};
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::file_format::CompiledModule;
use starcoin_vm_types::gas_schedule::{zero_cost_schedule, CostStrategy};
use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::identifier::IdentStr;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::transaction::{DryRunTransaction, Module, Package, TransactionPayloadType};
use starcoin_vm_types::transaction_metadata::TransactionPayloadMetadata;
use starcoin_vm_types::value::{serialize_values, MoveValue};
use starcoin_vm_types::vm_status::KeptVMStatus;
use starcoin_vm_types::write_set::{WriteOp, WriteSetMut};
use starcoin_vm_types::{
    effects::{ChangeSet as MoveChangeSet, Event as MoveEvent},
    errors::{self, IndexKind, Location},
    event::EventKey,
    gas_schedule::{self, CostTable, GasAlgebra, GasCarrier, GasUnits, InternalGasUnits},
    language_storage::TypeTag,
    on_chain_config::{OnChainConfig, VMConfig, Version},
    state_view::StateView,
    transaction_metadata::TransactionMetadata,
    values::Value,
    vm_status::{StatusCode, VMStatus},
};
use std::convert::TryFrom;
use std::sync::Arc;

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
/// Wrapper of MoveVM
pub struct StarcoinVM {
    move_vm: Arc<MoveVMAdapter>,
    vm_config: Option<VMConfig>,
    version: Option<Version>,
}

impl Default for StarcoinVM {
    fn default() -> Self {
        Self::new()
    }
}

impl StarcoinVM {
    pub fn new() -> Self {
        let inner = MoveVMAdapter::new();
        Self {
            move_vm: Arc::new(inner),
            vm_config: None,
            version: None,
        }
    }

    fn load_configs(&mut self, state: &dyn StateView) -> Result<(), Error> {
        if state.is_genesis() {
            self.vm_config = Some(VMConfig {
                gas_schedule: INITIAL_GAS_SCHEDULE.clone(),
            });
            self.version = Some(Version { major: 1 });
            Ok(())
        } else {
            self.load_configs_impl(state)
        }
    }

    fn load_configs_impl(&mut self, state: &dyn StateView) -> Result<(), Error> {
        let remote_storage = RemoteStorage::new(state);
        self.vm_config = Some(
            VMConfig::fetch_config(&remote_storage)?
                .ok_or_else(|| format_err!("Load VMConfig fail, VMConfig resource not exist."))?,
        );

        self.version = Some(
            Version::fetch_config(&remote_storage)?
                .ok_or_else(|| format_err!("Load Version fail, Version resource not exist."))?,
        );

        Ok(())
    }

    pub fn get_gas_schedule(&self) -> Result<&CostTable, VMStatus> {
        self.vm_config
            .as_ref()
            .map(|config| &config.gas_schedule)
            .ok_or(VMStatus::Error(StatusCode::VM_STARTUP_FAILURE))
    }

    pub fn get_version(&self) -> Result<Version, VMStatus> {
        self.version
            .clone()
            .ok_or(VMStatus::Error(StatusCode::VM_STARTUP_FAILURE))
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

        // The submitted max gas units that the transaction can consume is greater than the
        // maximum number of gas units bound that we have set for any
        // transaction.
        if txn_data.max_gas_amount().get() > gas_constants.maximum_number_of_gas_units.get() {
            warn!(
                "[VM] Gas unit error; max {}, submitted {}, with scaling_factor {}",
                gas_constants.maximum_number_of_gas_units.get(),
                txn_data.max_gas_amount().get(),
                gas_constants.gas_unit_scaling_factor
            );
            return Err(VMStatus::Error(
                StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND,
            ));
        }

        // The submitted transactions max gas units needs to be at least enough to cover the
        // intrinsic cost of the transaction as calculated against the size of the
        // underlying `RawTransaction`
        let min_txn_fee = gas_schedule::calculate_intrinsic_gas(raw_bytes_len, gas_constants);
        if gas_constants
            .to_internal_units(txn_data.max_gas_amount())
            .get()
            < min_txn_fee.get()
        {
            warn!(
                "[VM] Gas unit error; min {}, submitted {}, with scaling_factor {}",
                min_txn_fee.get(),
                txn_data.max_gas_amount().get(),
                gas_constants.gas_unit_scaling_factor
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
        let txn_data = TransactionMetadata::new(transaction)?;
        let mut session = self.move_vm.new_session(remote_cache);
        let mut cost_strategy = CostStrategy::system(self.get_gas_schedule()?, GasUnits::new(0));
        self.check_gas(&txn_data)?;
        match transaction.payload() {
            TransactionPayload::Package(package) => {
                let enforced = match Self::is_enforced(remote_cache, package.package_address()) {
                    Ok(is_enforced) => is_enforced,
                    _ => false,
                };
                match Self::only_new_module_strategy(remote_cache, package.package_address()) {
                    Err(e) => {
                        warn!("[VM]Update module strategy deserialize err : {:?}", e);
                        return Err(VMStatus::Error(StatusCode::FAILED_TO_DESERIALIZE_RESOURCE));
                    }
                    Ok(only_new_module) => {
                        for module in package.modules() {
                            self.check_compatibility_if_exist(
                                &session,
                                module,
                                only_new_module,
                                enforced,
                            )?;
                        }
                    }
                }
            }
            TransactionPayload::Script(_) => {}
            TransactionPayload::ScriptFunction(_) => {}
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

    fn check_compatibility_if_exist<R: RemoteCache>(
        &self,
        session: &SessionAdapter<R>,
        module: &Module,
        only_new_module: bool,
        enforced: bool,
    ) -> Result<(), VMStatus> {
        let compiled_module = match CompiledModule::deserialize(module.code()) {
            Ok(module) => module,
            Err(err) => {
                warn!("[VM] module deserialization failed {:?}", err);
                return Err(err.finish(Location::Undefined).into_vm_status());
            }
        };

        let module_id = compiled_module.self_id();
        if session
            .exists_module(&module_id)
            .map_err(|e| e.into_vm_status())?
        {
            if only_new_module {
                warn!("only new module for {:?}", module_id);
                return Err(VMStatus::Error(StatusCode::INVALID_MODULE_PUBLISHER));
            }
            let pre_version = session
                .load_module(&module_id)
                .map_err(|e| e.into_vm_status())?;
            let compatible = check_module_compat(pre_version.as_slice(), module.code())
                .map_err(|e| e.into_vm_status())?;
            if !compatible && !enforced {
                warn!("Check module compat error: {:?}", module_id);
                return Err(errors::verification_error(
                    StatusCode::BACKWARD_INCOMPATIBLE_MODULE_UPDATE,
                    IndexKind::ModuleHandle,
                    compiled_module.self_handle_idx().0,
                )
                .finish(Location::Undefined)
                .into_vm_status());
            }
        }
        Ok(())
    }

    fn only_new_module_strategy(
        remote_cache: &StateViewCache,
        package_address: AccountAddress,
    ) -> Result<bool> {
        let strategy_access_path = access_path_for_module_upgrade_strategy(package_address);
        if let Some(data) = remote_cache.get(&strategy_access_path)? {
            Ok(bcs_ext::from_bytes::<ModuleUpgradeStrategy>(&data)?.only_new_module())
        } else {
            Ok(false)
        }
    }

    fn is_enforced(remote_cache: &StateViewCache, package_address: AccountAddress) -> Result<bool> {
        let two_phase_upgrade_v2_path = access_path_for_two_phase_upgrade_v2(package_address);
        if let Some(data) = remote_cache.get(&two_phase_upgrade_v2_path)? {
            Ok(bcs_ext::from_bytes::<TwoPhaseUpgradeV2Resource>(&data)?.enforced())
        } else {
            Ok(false)
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
            let enforced = match Self::is_enforced(remote_cache, package_address) {
                Ok(is_enforced) => is_enforced,
                _ => false,
            };
            match Self::only_new_module_strategy(remote_cache, package_address) {
                Err(e) => {
                    warn!("[VM]Update module strategy deserialize err : {:?}", e);
                    return Err(VMStatus::Error(StatusCode::FAILED_TO_DESERIALIZE_RESOURCE));
                }
                Ok(only_new_module) => {
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
                                StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER,
                                IndexKind::AddressIdentifier,
                                compiled_module.self_handle_idx().0,
                            )
                            .finish(Location::Undefined)
                            .into_vm_status());
                        }
                        self.check_compatibility_if_exist(
                            &session,
                            module,
                            only_new_module,
                            enforced,
                        )?;

                        session
                            .verify_module(module.code())
                            .map_err(|e| e.into_vm_status())?;

                        session
                            .publish_module(module.code().to_vec(), txn_data.sender, cost_strategy)
                            .map_err(|e| e.into_vm_status())?;
                    }
                }
            }
            if let Some(init_script) = package.init_script() {
                let genesis_address = genesis_address();
                // If package owner is genesis, then init_script will run using the genesis address
                // instead of the txn sender address. It provides the opportunity to add new resource
                // under the genesis address through DAO.
                let sender = if package_address == genesis_address {
                    cost_strategy.disable_metering();
                    genesis_address
                } else {
                    txn_data.sender
                };
                debug!(
                    "execute init script({}::{}) by account {:?}",
                    init_script.module(),
                    init_script.function(),
                    sender
                );
                session
                    .execute_script_function(
                        init_script.module(),
                        init_script.function(),
                        init_script.ty_args().to_vec(),
                        init_script.args().to_vec(),
                        vec![sender],
                        cost_strategy,
                    )
                    .map_err(|e| e.into_vm_status())?;
            }
            charge_global_write_gas_usage(cost_strategy, &session, &txn_data.sender())?;

            cost_strategy.disable_metering();
            self.success_transaction_cleanup(
                session,
                gas_schedule,
                cost_strategy.remaining_gas(),
                txn_data,
            )
        }
    }

    fn execute_script_or_script_function(
        &self,
        remote_cache: &StateViewCache<'_>,
        gas_schedule: &CostTable,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
        payload: &TransactionPayload,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let mut session = self.move_vm.new_session(remote_cache);

        // Run the validation logic
        {
            cost_strategy.disable_metering();
            //let _timer = TXN_VERIFICATION_SECONDS.start_timer();
            self.check_gas(txn_data)?;
            self.run_prologue(&mut session, cost_strategy, &txn_data)?;
        }

        // Run the execution logic
        {
            //let _timer = TXN_EXECUTION_SECONDS.start_timer();
            cost_strategy.enable_metering();
            cost_strategy
                .charge_intrinsic_gas(txn_data.transaction_size())
                .map_err(|e| e.into_vm_status())?;
            match payload {
                TransactionPayload::Script(script) => session.execute_script(
                    script.code().to_vec(),
                    script.ty_args().to_vec(),
                    script.args().to_vec(),
                    vec![txn_data.sender()],
                    cost_strategy,
                ),
                TransactionPayload::ScriptFunction(script_function) => session
                    .execute_script_function(
                        script_function.module(),
                        script_function.function(),
                        script_function.ty_args().to_vec(),
                        script_function.args().to_vec(),
                        vec![txn_data.sender()],
                        cost_strategy,
                    ),
                TransactionPayload::Package(_) => {
                    return Err(VMStatus::Error(StatusCode::UNREACHABLE));
                }
            }
            .map_err(|e| e.into_vm_status())?;

            charge_global_write_gas_usage(cost_strategy, &session, &txn_data.sender())?;

            cost_strategy.disable_metering();
            self.success_transaction_cleanup(
                session,
                gas_schedule,
                cost_strategy.remaining_gas(),
                txn_data,
            )
        }
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_prologue<R: RemoteCache>(
        &self,
        session: &mut SessionAdapter<R>,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
    ) -> Result<(), VMStatus> {
        let genesis_address = genesis_address();
        let gas_token_ty = txn_data.gas_token_code().into();
        let txn_sequence_number = txn_data.sequence_number();
        let authentication_key_preimage = txn_data.authentication_key_preimage().to_vec();
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
            TransactionPayloadMetadata::ScriptFunction => (
                TransactionPayloadType::ScriptFunction,
                HashValue::zero(),
                AccountAddress::ZERO,
            ),
        };

        // Run prologue by genesis account
        session
            .execute_function(
                &account_config::TRANSACTION_MANAGER_MODULE,
                &PROLOGUE_NAME,
                vec![gas_token_ty],
                serialize_values(&vec![
                    MoveValue::Signer(genesis_address),
                    MoveValue::Address(txn_data.sender),
                    MoveValue::U64(txn_sequence_number),
                    MoveValue::vector_u8(authentication_key_preimage),
                    MoveValue::U64(txn_gas_price),
                    MoveValue::U64(txn_max_gas_amount),
                    MoveValue::U64(txn_expiration_time),
                    MoveValue::U8(chain_id),
                    MoveValue::U8(payload_type.into()),
                    MoveValue::vector_u8(script_or_package_hash.to_vec()),
                    MoveValue::Address(package_address),
                ]),
                cost_strategy,
            )
            .map(|_return_vals| ())
            .or_else(convert_prologue_runtime_error)
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_epilogue<R: RemoteCache>(
        &self,
        session: &mut SessionAdapter<R>,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
        success: bool,
    ) -> Result<(), VMStatus> {
        let genesis_address = genesis_address();
        let gas_token_ty = txn_data.gas_token_code().into();
        let txn_sequence_number = txn_data.sequence_number();
        let txn_authentication_key_preimage = txn_data.authentication_key_preimage().to_vec();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_amount = txn_data.max_gas_amount().get();
        let gas_remaining = cost_strategy.remaining_gas().get();
        let (payload_type, script_or_package_hash, package_address) = match txn_data.payload() {
            TransactionPayloadMetadata::Script(hash) => {
                (TransactionPayloadType::Script, *hash, AccountAddress::ZERO)
            }
            TransactionPayloadMetadata::Package(hash, package_address) => {
                (TransactionPayloadType::Package, *hash, *package_address)
            }
            TransactionPayloadMetadata::ScriptFunction => (
                TransactionPayloadType::ScriptFunction,
                HashValue::zero(),
                AccountAddress::ZERO,
            ),
        };
        let stdlib_version = self.get_version()?.into_stdlib_version();
        // Run epilogue by genesis account, second arg is txn sender.
        // From stdlib v5, the epilogue function add `txn_authentication_key_preimage` argument, change to epilogue_v2
        let (function_name, args) = if stdlib_version > StdlibVersion::Version(4) {
            (
                &EPILOGUE_V2_NAME,
                serialize_values(&vec![
                    MoveValue::Signer(genesis_address),
                    MoveValue::Address(txn_data.sender),
                    MoveValue::U64(txn_sequence_number),
                    MoveValue::vector_u8(txn_authentication_key_preimage),
                    MoveValue::U64(txn_gas_price),
                    MoveValue::U64(txn_max_gas_amount),
                    MoveValue::U64(gas_remaining),
                    MoveValue::U8(payload_type.into()),
                    MoveValue::vector_u8(script_or_package_hash.to_vec()),
                    MoveValue::Address(package_address),
                    MoveValue::Bool(success),
                ]),
            )
        } else {
            (
                &EPILOGUE_NAME,
                serialize_values(&vec![
                    MoveValue::Signer(genesis_address),
                    MoveValue::Address(txn_data.sender),
                    MoveValue::U64(txn_sequence_number),
                    MoveValue::U64(txn_gas_price),
                    MoveValue::U64(txn_max_gas_amount),
                    MoveValue::U64(gas_remaining),
                    MoveValue::U8(payload_type.into()),
                    MoveValue::vector_u8(script_or_package_hash.to_vec()),
                    MoveValue::Address(package_address),
                    MoveValue::Bool(success),
                ]),
            )
        };
        session
            .execute_function(
                &account_config::TRANSACTION_MANAGER_MODULE,
                function_name,
                vec![gas_token_ty],
                args,
                cost_strategy,
            )
            .map(|_return_vals| ())
            .or_else(convert_normal_success_epilogue_error)
    }

    fn process_block_metadata(
        &self,
        remote_cache: &mut StateViewCache<'_>,
        block_metadata: BlockMetadata,
    ) -> Result<TransactionOutput, VMStatus> {
        let txn_sender = account_config::genesis_address();
        // always use 0 gas for system.
        let max_gas_amount = GasUnits::new(0);
        let gas_schedule = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&gas_schedule, max_gas_amount);

        let (
            parent_id,
            timestamp,
            author,
            author_auth_key,
            uncles,
            number,
            chain_id,
            parent_gas_used,
        ) = block_metadata.into_inner();
        let args = serialize_values(&vec![
            MoveValue::Signer(txn_sender),
            MoveValue::vector_u8(parent_id.to_vec()),
            MoveValue::U64(timestamp),
            MoveValue::Address(author),
            match author_auth_key {
                Some(author_auth_key) => MoveValue::vector_u8(author_auth_key.to_vec()),
                None => MoveValue::vector_u8(Vec::new()),
            },
            MoveValue::U64(uncles),
            MoveValue::U64(number),
            MoveValue::U8(chain_id.id()),
            MoveValue::U64(parent_gas_used),
        ]);
        let mut session = self.move_vm.new_session(remote_cache);
        session
            .execute_function(
                &account_config::TRANSACTION_MANAGER_MODULE,
                &account_config::BLOCK_PROLOGUE_NAME,
                vec![],
                args,
                &mut cost_strategy,
            )
            .map(|_return_vals| ())
            .or_else(convert_prologue_runtime_error)?;
        BLOCK_UNCLES.observe(uncles as f64);
        get_transaction_output(
            &mut (),
            session,
            &cost_strategy,
            max_gas_amount,
            KeptVMStatus::Executed,
        )
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
        let txn_id = txn.id();
        let txn_data = match TransactionMetadata::new(&txn) {
            Ok(txn_data) => txn_data,
            Err(e) => {
                return discard_error_vm_status(e);
            }
        };
        let mut cost_strategy = CostStrategy::system(gas_schedule, txn_data.max_gas_amount());
        // check signature
        let signature_checked_txn = match txn.check_signature() {
            Ok(t) => Ok(t),
            Err(_) => Err(VMStatus::Error(StatusCode::INVALID_SIGNATURE)),
        };

        match signature_checked_txn {
            Ok(txn) => {
                let result = match txn.payload() {
                    payload @ TransactionPayload::Script(_)
                    | payload @ TransactionPayload::ScriptFunction(_) => self
                        .execute_script_or_script_function(
                            remote_cache,
                            gas_schedule,
                            &mut cost_strategy,
                            &txn_data,
                            payload,
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
                    Ok(status_and_output) => {
                        log_vm_status(
                            txn_id,
                            &txn_data,
                            &status_and_output.0,
                            Some(&status_and_output.1),
                        );
                        status_and_output
                    }
                    Err(err) => {
                        let txn_status = TransactionStatus::from(err.clone());
                        log_vm_status(txn_id, &txn_data, &err, None);
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

    pub fn dry_run_transaction(
        &mut self,

        state_view: &dyn StateView,
        txn: DryRunTransaction,
    ) -> Result<(VMStatus, TransactionOutput)> {
        let remote_cache = StateViewCache::new(state_view);
        //TODO load config by config change event.
        self.load_configs(&remote_cache)?;

        let gas_schedule = match self.get_gas_schedule() {
            Ok(gas_schedule) => gas_schedule,
            Err(e) => {
                if remote_cache.is_genesis() {
                    &INITIAL_GAS_SCHEDULE
                } else {
                    return Ok(discard_error_vm_status(e));
                }
            }
        };
        let txn_data = match TransactionMetadata::from_raw_txn_and_preimage(
            &txn.raw_txn,
            txn.public_key.authentication_key_preimage(),
        ) {
            Ok(txn_data) => txn_data,
            Err(e) => return Ok(discard_error_vm_status(e)),
        };
        let mut cost_strategy = CostStrategy::system(gas_schedule, txn_data.max_gas_amount());
        let result = match txn.raw_txn.payload() {
            payload @ TransactionPayload::Script(_)
            | payload @ TransactionPayload::ScriptFunction(_) => self
                .execute_script_or_script_function(
                    &remote_cache,
                    gas_schedule,
                    &mut cost_strategy,
                    &txn_data,
                    payload,
                ),
            TransactionPayload::Package(p) => self.execute_package(
                &remote_cache,
                gas_schedule,
                &mut cost_strategy,
                &txn_data,
                p,
            ),
        };
        Ok(match result {
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
                        &remote_cache,
                    )
                }
            }
        })
    }

    fn check_reconfigure(
        &mut self,
        _state_view: &dyn StateView,
        _output: &TransactionOutput,
    ) -> Result<(), Error> {
        //TODO this vm is need to reconfigure by the check the output event
        //if need reconfigure, do load_config
        Ok(())
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

        let mut gas_left = block_gas_limit.unwrap_or(u64::MAX);

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
                        self.check_reconfigure(&data_cache, &output)?;
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

    pub fn execute_readonly_function(
        &mut self,
        state_view: &dyn StateView,
        module: &ModuleId,
        function_name: &IdentStr,
        type_params: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<(TypeTag, Value)>, VMStatus> {
        let data_cache = StateViewCache::new(state_view);
        if let Err(err) = self.load_configs(&data_cache) {
            warn!("Load config error at verify_transaction: {}", err);
            return Err(VMStatus::Error(StatusCode::VM_STARTUP_FAILURE));
        }

        let cost_table = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&cost_table, GasUnits::new(0));
        let mut session = self.move_vm.new_session(&data_cache);
        let result = session
            .execute_readonly_function(module, function_name, type_params, args, &mut cost_strategy)
            .map_err(|e| e.into_vm_status())?;

        let (changeset, events) = session.finish().map_err(|e| e.into_vm_status())?;
        let (writeset, _events) = convert_changeset_and_events(changeset, events)?;
        if !writeset.is_empty() {
            warn!("Readonly function {} changes state", function_name);
            return Err(VMStatus::Error(StatusCode::REJECTED_WRITE_SET));
        }
        Ok(result)
    }

    fn success_transaction_cleanup<R: RemoteCache>(
        &self,
        mut session: SessionAdapter<R>,
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
                txn_data.max_gas_amount,
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
                let txn_output = get_transaction_output(
                    &mut (),
                    session,
                    &cost_strategy,
                    txn_data.max_gas_amount,
                    status,
                )
                .unwrap_or_else(|e| discard_error_vm_status(e).1);
                (error_code, txn_output)
            }
            TransactionStatus::Discard(status) => {
                (VMStatus::Error(status), discard_error_output(status))
            }
        }
    }
}

#[allow(clippy::large_enum_variant)]
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
    session: &SessionAdapter<R>,
    sender: &AccountAddress,
) -> Result<(), VMStatus> {
    let total_cost = session.num_mutated_accounts(sender)
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
        .deduct_gas(InternalGasUnits::new(total_cost))
        .map_err(|p_err| p_err.finish(Location::Undefined).into_vm_status())
}

pub(crate) fn discard_error_vm_status(err: VMStatus) -> (VMStatus, TransactionOutput) {
    info!("discard error vm_status output: {:?}", err);
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
        TransactionStatus::Discard(err),
    )
}

pub fn convert_changeset_and_events_cached<C: AccessPathCache>(
    ap_cache: &mut C,
    changeset: MoveChangeSet,
    events: Vec<MoveEvent>,
) -> Result<(WriteSet, Vec<ContractEvent>), VMStatus> {
    // TODO: Cache access path computations if necessary.
    let mut ops = vec![];

    for (addr, account_changeset) in changeset.accounts {
        for (struct_tag, blob_opt) in account_changeset.resources {
            let ap = ap_cache.get_resource_path(addr, struct_tag);
            let op = match blob_opt {
                None => WriteOp::Deletion,
                Some(blob) => WriteOp::Value(blob),
            };
            ops.push((ap, op))
        }

        for (name, blob_opt) in account_changeset.modules {
            let ap = ap_cache.get_module_path(ModuleId::new(addr, name));
            let op = match blob_opt {
                None => WriteOp::Deletion,
                Some(blob) => WriteOp::Value(blob),
            };

            ops.push((ap, op))
        }
    }

    let ws = WriteSetMut::new(ops)
        .freeze()
        .map_err(|_| VMStatus::Error(StatusCode::DATA_FORMAT_ERROR))?;

    let events = events
        .into_iter()
        .map(|(guid, seq_num, ty_tag, blob)| {
            let key = EventKey::try_from(guid.as_slice())
                .map_err(|_| VMStatus::Error(StatusCode::EVENT_KEY_MISMATCH))?;
            Ok(ContractEvent::new(key, seq_num, ty_tag, blob))
        })
        .collect::<Result<Vec<_>, VMStatus>>()?;

    Ok((ws, events))
}

pub fn convert_changeset_and_events(
    changeset: MoveChangeSet,
    events: Vec<MoveEvent>,
) -> Result<(WriteSet, Vec<ContractEvent>), VMStatus> {
    convert_changeset_and_events_cached(&mut (), changeset, events)
}

pub(crate) fn get_transaction_output<A: AccessPathCache, R: RemoteCache>(
    ap_cache: &mut A,
    session: SessionAdapter<R>,
    cost_strategy: &CostStrategy,
    max_gas_amount: GasUnits<GasCarrier>,
    status: KeptVMStatus,
) -> Result<TransactionOutput, VMStatus> {
    let gas_used: u64 = max_gas_amount.sub(cost_strategy.remaining_gas()).get();

    let (changeset, events) = session.finish().map_err(|e| e.into_vm_status())?;
    let (write_set, events) = convert_changeset_and_events_cached(ap_cache, changeset, events)?;

    TXN_EXECUTION_GAS_USAGE.observe(gas_used as f64);

    Ok(TransactionOutput::new(
        write_set,
        events,
        gas_used,
        TransactionStatus::Keep(status),
    ))
}

pub enum VerifiedTransactionPayload {
    Script(Vec<u8>, Vec<TypeTag>, Vec<Value>),
    Package(Package),
}

pub fn log_vm_status(
    txn_id: HashValue,
    txn_data: &TransactionMetadata,
    status: &VMStatus,
    txn_output: Option<&TransactionOutput>,
) {
    let msg = match status {
        VMStatus::Executed => "Executed".to_string(),
        VMStatus::MoveAbort(location, code) => {
            let (category, reason) = error_split(*code);
            format!(
                "MoveAbort code: {}, (Category: {:?} Reason: {:?}), location: {:?}",
                code, category, reason, location
            )
        }
        status => format!("{:?}", status),
    };

    match txn_output {
        Some(output) => {
            //TODO change log level after main network launch.
            info!(
                "[starcoin-vm] Executed txn: {:?} (sender: {:?}, sequence_number: {:?}) txn_status: {:?}, gas_used: {}, vm_status: {}",
                txn_id, txn_data.sender, txn_data.sequence_number, output.status(), output.gas_used(), msg,
            );
        }
        None => {
            let txn_status = TransactionStatus::from(status.clone());
            info!(
                "[starcoin-vm] Executed txn: {:?} (sender: {:?}, sequence_number: {:?}) txn_status: {:?}, vm_status: {}",
                txn_id, txn_data.sender, txn_data.sequence_number, txn_status, msg,
            );
        }
    }
}
