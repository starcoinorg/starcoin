// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path_cache::AccessPathCache;
use crate::data_cache::{RemoteStorage, StateViewCache};
use crate::errors::{
    convert_normal_success_epilogue_error, convert_prologue_runtime_error, error_split,
};
use crate::metrics::VMMetrics;
use anyhow::{format_err, Error, Result};
use crypto::HashValue;
use move_core_types::resolver::MoveResolver;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_runtime::move_vm_adapter::{PublishModuleBundleOption, SessionAdapter};
use move_vm_runtime::session::Session;
use once_cell::sync::Lazy;
use starcoin_config::LATEST_GAS_SCHEDULE;
use starcoin_logger::prelude::*;
use starcoin_types::account_config::config_change::ConfigChangeEvent;
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
use starcoin_vm_types::access::{ModuleAccess, ScriptAccess};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::upgrade::UpgradeEvent;
use starcoin_vm_types::account_config::{
    core_code_address, genesis_address, ModuleUpgradeStrategy, TwoPhaseUpgradeV2Resource,
    EPILOGUE_NAME, EPILOGUE_V2_NAME, PROLOGUE_NAME,
};
use starcoin_vm_types::contract_event::ContractEvent;
use starcoin_vm_types::file_format::{CompiledModule, CompiledScript};
use starcoin_vm_types::gas_schedule::NativeCostIndex;
use starcoin_vm_types::gas_schedule::{zero_cost_schedule, GasConstants, GasCost, GasStatus};
use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::identifier::IdentStr;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::on_chain_config::{
    MoveLanguageVersion, GAS_CONSTANTS_IDENTIFIER, INSTRUCTION_SCHEDULE_IDENTIFIER,
    NATIVE_SCHEDULE_IDENTIFIER, VM_CONFIG_IDENTIFIER,
};
use starcoin_vm_types::transaction::{DryRunTransaction, Package, TransactionPayloadType};
use starcoin_vm_types::transaction_metadata::TransactionPayloadMetadata;
use starcoin_vm_types::value::{serialize_values, MoveValue};
use starcoin_vm_types::vm_status::KeptVMStatus;
use starcoin_vm_types::write_set::{WriteOp, WriteSetMut};
use starcoin_vm_types::{
    effects::{ChangeSet as MoveChangeSet, Event as MoveEvent},
    errors::Location,
    event::EventKey,
    gas_schedule::{self, CostTable, GasAlgebra, GasCarrier, GasUnits, InternalGasUnits},
    language_storage::TypeTag,
    on_chain_config::{OnChainConfig, VMConfig, Version},
    state_view::StateView,
    transaction_metadata::TransactionMetadata,
    values::Value,
    vm_status::{StatusCode, VMStatus},
};
use std::convert::{TryFrom, TryInto};
use std::sync::Arc;

static ZERO_COST_SCHEDULE: Lazy<CostTable> =
    Lazy::new(|| zero_cost_schedule(NativeCostIndex::NUMBER_OF_NATIVE_FUNCTIONS));

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
/// Wrapper of MoveVM
pub struct StarcoinVM {
    move_vm: Arc<MoveVM>,
    vm_config: Option<VMConfig>,
    version: Option<Version>,
    move_version: Option<MoveLanguageVersion>,
    metrics: Option<VMMetrics>,
}

/// marking of stdlib version which includes vmconfig upgrades.
const VMCONFIG_UPGRADE_VERSION_MARK: u64 = 10;

impl StarcoinVM {
    pub fn new(metrics: Option<VMMetrics>) -> Self {
        let inner = MoveVM::new(super::natives::starcoin_natives())
            .expect("should be able to create Move VM; check if there are duplicated natives");
        Self {
            move_vm: Arc::new(inner),
            vm_config: None,
            version: None,
            move_version: None,
            metrics,
        }
    }

    pub fn load_configs(&mut self, state: &dyn StateView) -> Result<(), Error> {
        if state.is_genesis() {
            self.vm_config = Some(VMConfig {
                gas_schedule: LATEST_GAS_SCHEDULE.clone(),
            });
            self.version = Some(Version { major: 1 });
            Ok(())
        } else {
            self.load_configs_impl(state)
        }
    }

    fn load_configs_impl(&mut self, state: &dyn StateView) -> Result<(), Error> {
        let remote_storage = RemoteStorage::new(state);
        self.version = Some(
            Version::fetch_config(&remote_storage)?
                .ok_or_else(|| format_err!("Load Version fail, Version resource not exist."))?,
        );
        // move version can be none.
        self.move_version = MoveLanguageVersion::fetch_config(&remote_storage)?;

        if let Some(v) = &self.version {
            // if version is 0, it represent latest version. we should consider it.
            let stdlib_version = v.clone().into_stdlib_version();
            self.vm_config = if stdlib_version
                < StdlibVersion::Version(VMCONFIG_UPGRADE_VERSION_MARK)
            {
                debug!(
                    "stdlib version: {}, fetch vmconfig from onchain resource",
                    stdlib_version
                );
                Some(VMConfig::fetch_config(&remote_storage)?.ok_or_else(|| {
                    format_err!("Load VMConfig fail, VMConfig resource not exist.")
                })?)
            } else {
                debug!(
                    "stdlib version: {}, fetch vmconfig from onchain module",
                    stdlib_version
                );
                let instruction_schedule = {
                    let data = self
                        .execute_readonly_function(
                            state,
                            &ModuleId::new(core_code_address(), VM_CONFIG_IDENTIFIER.to_owned()),
                            INSTRUCTION_SCHEDULE_IDENTIFIER.as_ident_str(),
                            vec![],
                            vec![],
                        )?
                        .pop()
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Expect 0x1::VMConfig::instruction_schedule() return value"
                            )
                        })?;
                    bcs_ext::from_bytes::<Vec<GasCost>>(&data)?
                };
                let native_schedule = {
                    let data = self
                        .execute_readonly_function(
                            state,
                            &ModuleId::new(core_code_address(), VM_CONFIG_IDENTIFIER.to_owned()),
                            NATIVE_SCHEDULE_IDENTIFIER.as_ident_str(),
                            vec![],
                            vec![],
                        )?
                        .pop()
                        .ok_or_else(|| {
                            anyhow::anyhow!("Expect 0x1::VMConfig::native_schedule() return value")
                        })?;
                    bcs_ext::from_bytes::<Vec<GasCost>>(&data)?
                };
                let gas_constants = {
                    let data = self
                        .execute_readonly_function(
                            state,
                            &ModuleId::new(core_code_address(), VM_CONFIG_IDENTIFIER.to_owned()),
                            GAS_CONSTANTS_IDENTIFIER.as_ident_str(),
                            vec![],
                            vec![],
                        )?
                        .pop()
                        .ok_or_else(|| {
                            anyhow::anyhow!("Expect 0x1::VMConfig::gas_constants() return value")
                        })?;
                    bcs_ext::from_bytes::<GasConstants>(&data)?
                };

                Some(VMConfig {
                    gas_schedule: CostTable {
                        instruction_table: instruction_schedule,
                        native_table: native_schedule,
                        gas_constants,
                    },
                })
            }
        }
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
    pub fn get_move_version(&self) -> Option<MoveLanguageVersion> {
        self.move_version
    }

    fn check_move_version(&self, package_or_script_bytecode_version: u64) -> Result<(), VMStatus> {
        // if move_version config is not exists on chain, this check will do no harm.
        if let Some(supported_move_version) = &self.move_version {
            if package_or_script_bytecode_version > supported_move_version.major {
                // TODO: currently, if the bytecode version of a package or script is higher than onchain config,
                // return `FEATURE_UNDER_GATING` error, and the txn will not be included in blocks.
                return Err(VMStatus::Error(StatusCode::FEATURE_UNDER_GATING));
            }
        }
        Ok(())
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
        let mut session: SessionAdapter<_> = self.move_vm.new_session(remote_cache).into();
        let mut gas_status = {
            let mut gas_status = GasStatus::new(self.get_gas_schedule()?, GasUnits::new(0));
            gas_status.set_metering(false);
            gas_status
        };

        self.check_gas(&txn_data)?;
        match transaction.payload() {
            TransactionPayload::Package(package) => {
                for module in package.modules() {
                    if let Ok(compiled_module) = CompiledModule::deserialize(module.code()) {
                        // check module's bytecode version.
                        self.check_move_version(compiled_module.version() as u64)?;
                    };
                }
                let enforced = match Self::is_enforced(remote_cache, package.package_address()) {
                    Ok(is_enforced) => is_enforced,
                    _ => false,
                };
                let only_new_module =
                    match Self::only_new_module_strategy(remote_cache, package.package_address()) {
                        Err(e) => {
                            warn!("[VM]Update module strategy deserialize err : {:?}", e);
                            return Err(VMStatus::Error(
                                StatusCode::FAILED_TO_DESERIALIZE_RESOURCE,
                            ));
                        }
                        Ok(only_new_module) => only_new_module,
                    };
                let _ = session
                    .verify_module_bundle(
                        package
                            .modules()
                            .iter()
                            .map(|m| m.code().to_vec())
                            .collect(),
                        package.package_address(),
                        &mut gas_status,
                        PublishModuleBundleOption {
                            force_publish: enforced,
                            only_new_module,
                        },
                    )
                    .map_err(|e| e.into_vm_status())?;
            }
            TransactionPayload::Script(s) => {
                if let Ok(s) = CompiledScript::deserialize(s.code()) {
                    self.check_move_version(s.version() as u64)?;
                };
                session
                    .verify_script_args(
                        s.code().to_vec(),
                        s.ty_args().to_vec(),
                        s.args().to_vec(),
                        vec![txn_data.sender()],
                    )
                    .map_err(|e| e.into_vm_status())?;
            }
            TransactionPayload::ScriptFunction(s) => {
                session
                    .verify_script_function_args(
                        s.module(),
                        s.function(),
                        s.ty_args().to_vec(),
                        s.args().to_vec(),
                        vec![txn_data.sender()],
                    )
                    .map_err(|e| e.into_vm_status())?;
            }
        }
        self.run_prologue(&mut session, &mut gas_status, &txn_data)
    }

    pub fn verify_transaction(
        &mut self,
        state_view: &dyn StateView,
        txn: SignedUserTransaction,
    ) -> Option<VMStatus> {
        let _timer = self.metrics.as_ref().map(|metrics| {
            metrics
                .vm_txn_exe_time
                .with_label_values(&["verify_transaction"])
                .start_timer()
        });
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
        cost_strategy: &mut GasStatus,
        txn_data: &TransactionMetadata,
        package: &Package,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let mut session: SessionAdapter<_> = self.move_vm.new_session(remote_cache).into();

        {
            // Run the validation logic
            cost_strategy.set_metering(false);
            // genesis txn skip check gas and txn prologue.
            if !remote_cache.is_genesis() {
                //let _timer = TXN_VERIFICATION_SECONDS.start_timer();
                self.check_gas(txn_data)?;
                self.run_prologue(&mut session, cost_strategy, txn_data)?;
            }
        }
        {
            // Genesis txn not enable gas charge.
            if !remote_cache.is_genesis() {
                cost_strategy.set_metering(true);
            }
            cost_strategy
                .charge_intrinsic_gas(txn_data.transaction_size())
                .map_err(|e| e.into_vm_status())?;

            let package_address = package.package_address();
            for module in package.modules() {
                if let Ok(compiled_module) = CompiledModule::deserialize(module.code()) {
                    // check module's bytecode version.
                    self.check_move_version(compiled_module.version() as u64)?;
                };
            }
            let enforced = match Self::is_enforced(remote_cache, package_address) {
                Ok(is_enforced) => is_enforced,
                _ => false,
            };
            let only_new_module =
                match Self::only_new_module_strategy(remote_cache, package_address) {
                    Err(e) => {
                        warn!("[VM]Update module strategy deserialize err : {:?}", e);
                        return Err(VMStatus::Error(StatusCode::FAILED_TO_DESERIALIZE_RESOURCE));
                    }
                    Ok(only_new_module) => only_new_module,
                };

            session
                .publish_module_bundle_with_option(
                    package
                        .modules()
                        .iter()
                        .map(|m| m.code().to_vec())
                        .collect(),
                    package.package_address(), // be careful with the sender.
                    cost_strategy,
                    PublishModuleBundleOption {
                        force_publish: enforced,
                        only_new_module,
                    },
                )
                .map_err(|e| e.into_vm_status())?;

            // after publish the modules, we need to clear loader cache, to make init script function and
            // epilogue use the new modules.
            session.empty_loader_cache()?;

            if let Some(init_script) = package.init_script() {
                let genesis_address = genesis_address();
                // If package owner is genesis, then init_script will run using the genesis address
                // instead of the txn sender address. It provides the opportunity to add new resource
                // under the genesis address through DAO.
                let sender = if package_address == genesis_address {
                    cost_strategy.set_metering(false);
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
                    .as_mut()
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

            cost_strategy.set_metering(false);
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
        cost_strategy: &mut GasStatus,
        txn_data: &TransactionMetadata,
        payload: &TransactionPayload,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let mut session: SessionAdapter<_> = self.move_vm.new_session(remote_cache).into();

        // Run the validation logic
        {
            cost_strategy.set_metering(false);
            //let _timer = TXN_VERIFICATION_SECONDS.start_timer();
            self.check_gas(txn_data)?;
            self.run_prologue(&mut session, cost_strategy, txn_data)?;
        }

        // Run the execution logic
        {
            //let _timer = TXN_EXECUTION_SECONDS.start_timer();
            cost_strategy.set_metering(true);
            cost_strategy
                .charge_intrinsic_gas(txn_data.transaction_size())
                .map_err(|e| e.into_vm_status())?;
            match payload {
                TransactionPayload::Script(script) => {
                    // we only use the ok path, let move vm handle the wrong path.
                    if let Ok(s) = CompiledScript::deserialize(script.code()) {
                        self.check_move_version(s.version() as u64)?;
                    };

                    session.as_mut().execute_script(
                        script.code().to_vec(),
                        script.ty_args().to_vec(),
                        script.args().to_vec(),
                        vec![txn_data.sender()],
                        cost_strategy,
                    )
                }
                TransactionPayload::ScriptFunction(script_function) => {
                    session.as_mut().execute_script_function(
                        script_function.module(),
                        script_function.function(),
                        script_function.ty_args().to_vec(),
                        script_function.args().to_vec(),
                        vec![txn_data.sender()],
                        cost_strategy,
                    )
                }
                TransactionPayload::Package(_) => {
                    return Err(VMStatus::Error(StatusCode::UNREACHABLE));
                }
            }
            .map_err(|e| e.into_vm_status())?;

            charge_global_write_gas_usage(cost_strategy, &session, &txn_data.sender())?;

            cost_strategy.set_metering(false);
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
    fn run_prologue<R: MoveResolver>(
        &self,
        session: &mut SessionAdapter<R>,
        gas_status: &mut GasStatus,
        txn_data: &TransactionMetadata,
    ) -> Result<(), VMStatus> {
        let genesis_address = genesis_address();
        let gas_token_ty = TypeTag::Struct(
            txn_data
                .gas_token_code()
                .try_into()
                .map_err(|_e| VMStatus::Error(StatusCode::BAD_TRANSACTION_FEE_CURRENCY))?,
        );
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
            .as_mut()
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
                gas_status,
            )
            .map(|_return_vals| ())
            .or_else(convert_prologue_runtime_error)
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_epilogue<R: MoveResolver>(
        &self,
        session: &mut SessionAdapter<R>,
        gas_status: &mut GasStatus,
        txn_data: &TransactionMetadata,
        success: bool,
    ) -> Result<(), VMStatus> {
        let genesis_address = genesis_address();
        let gas_token_ty = TypeTag::Struct(
            txn_data
                .gas_token_code()
                .try_into()
                .map_err(|_e| VMStatus::Error(StatusCode::BAD_TRANSACTION_FEE_CURRENCY))?,
        );
        let txn_sequence_number = txn_data.sequence_number();
        let txn_authentication_key_preimage = txn_data.authentication_key_preimage().to_vec();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_amount = txn_data.max_gas_amount().get();
        let gas_remaining = gas_status.remaining_gas().get();
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
            .as_mut()
            .execute_function(
                &account_config::TRANSACTION_MANAGER_MODULE,
                function_name,
                vec![gas_token_ty],
                args,
                gas_status,
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
        let cost_table = &ZERO_COST_SCHEDULE;
        let mut gas_status = {
            let mut gas_status = GasStatus::new(cost_table, max_gas_amount);
            gas_status.set_metering(false);
            gas_status
        };

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
        let mut session: SessionAdapter<_> = self.move_vm.new_session(remote_cache).into();
        session
            .as_mut()
            .execute_function(
                &account_config::TRANSACTION_MANAGER_MODULE,
                &account_config::BLOCK_PROLOGUE_NAME,
                vec![],
                args,
                &mut gas_status,
            )
            .map(|_return_vals| ())
            .or_else(convert_prologue_runtime_error)?;
        get_transaction_output(
            &mut (),
            session,
            &gas_status,
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
                    &LATEST_GAS_SCHEDULE
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

        let mut gas_status = {
            let mut gas_status = GasStatus::new(gas_schedule, txn_data.max_gas_amount());
            gas_status.set_metering(false);
            gas_status
        };
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
                            &mut gas_status,
                            &txn_data,
                            payload,
                        ),
                    TransactionPayload::Package(p) => self.execute_package(
                        remote_cache,
                        gas_schedule,
                        &mut gas_status,
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
                                gas_status.remaining_gas(),
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
                    &LATEST_GAS_SCHEDULE
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
        let mut gas_status = {
            let mut gas_status = GasStatus::new(gas_schedule, txn_data.max_gas_amount());
            gas_status.set_metering(false);
            gas_status
        };
        let result = match txn.raw_txn.payload() {
            payload @ TransactionPayload::Script(_)
            | payload @ TransactionPayload::ScriptFunction(_) => self
                .execute_script_or_script_function(
                    &remote_cache,
                    gas_schedule,
                    &mut gas_status,
                    &txn_data,
                    payload,
                ),
            TransactionPayload::Package(p) => {
                self.execute_package(&remote_cache, gas_schedule, &mut gas_status, &txn_data, p)
            }
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
                        gas_status.remaining_gas(),
                        &txn_data,
                        &remote_cache,
                    )
                }
            }
        })
    }

    fn check_reconfigure(
        &mut self,
        state_view: &dyn StateView,
        output: &TransactionOutput,
    ) -> Result<(), Error> {
        for event in output.events() {
            if event.key().get_creator_address() == genesis_address()
                && (event.is::<UpgradeEvent>() || event.is::<ConfigChangeEvent<Version>>())
            {
                info!("Load vm configs trigger by reconfigure event. ");
                self.load_configs(state_view)?;
            }
        }
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
            let txn_type_name = block.type_name().to_string();
            match block {
                TransactionBlock::UserTransaction(txns) => {
                    for transaction in txns {
                        let timer = self.metrics.as_ref().map(|metrics| {
                            metrics
                                .vm_txn_exe_time
                                .with_label_values(&[txn_type_name.as_str()])
                                .start_timer()
                        });
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
                        if let Some(timer) = timer {
                            timer.observe_duration();
                        }
                        if let Some(metrics) = self.metrics.as_ref() {
                            metrics.vm_txn_gas_usage.observe(output.gas_used() as f64);
                            metrics
                                .vm_txn_exe_total
                                .with_label_values(&[
                                    txn_type_name.as_str(),
                                    status.status_type().to_string().as_str(),
                                ])
                                .inc();
                        }
                        result.push((status, output));
                    }
                }
                TransactionBlock::BlockPrologue(block_metadata) => {
                    let timer = self.metrics.as_ref().map(|metrics| {
                        metrics
                            .vm_txn_exe_time
                            .with_label_values(&[txn_type_name.as_str()])
                            .start_timer()
                    });
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
                    if let Some(timer) = timer {
                        timer.observe_duration();
                    }
                    if let Some(metrics) = self.metrics.as_ref() {
                        metrics
                            .vm_txn_exe_total
                            .with_label_values(&[
                                txn_type_name.as_str(),
                                status.status_type().to_string().as_str(),
                            ])
                            .inc();
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
    ) -> Result<Vec<Vec<u8>>, VMStatus> {
        let _timer = self.metrics.as_ref().map(|metrics| {
            metrics
                .vm_txn_exe_time
                .with_label_values(&["execute_readonly_function"])
                .start_timer()
        });
        let data_cache = StateViewCache::new(state_view);

        let cost_table = &ZERO_COST_SCHEDULE;
        let mut gas_status = {
            let mut gas_status = GasStatus::new(cost_table, GasUnits::new(0));
            gas_status.set_metering(false);
            gas_status
        };
        let mut session = self.move_vm.new_session(&data_cache);
        let result = session
            .execute_function(module, function_name, type_params, args, &mut gas_status)
            .map_err(|e| e.into_vm_status())?;

        let (changeset, events) = session.finish().map_err(|e| e.into_vm_status())?;
        let (writeset, _events) = convert_changeset_and_events(changeset, events)?;
        if !writeset.is_empty() {
            warn!("Readonly function {} changes state", function_name);
            return Err(VMStatus::Error(StatusCode::REJECTED_WRITE_SET));
        }
        Ok(result)
    }

    fn success_transaction_cleanup<R: MoveResolver>(
        &self,
        mut session: SessionAdapter<R>,
        gas_schedule: &CostTable,
        gas_left: GasUnits<GasCarrier>,
        txn_data: &TransactionMetadata,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let mut gas_status = {
            let mut gas_status = GasStatus::new(gas_schedule, gas_left);
            gas_status.set_metering(false);
            gas_status
        };
        self.run_epilogue(&mut session, &mut gas_status, txn_data, true)?;

        Ok((
            VMStatus::Executed,
            get_transaction_output(
                &mut (),
                session,
                &gas_status,
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
        let mut gas_status = {
            let mut gas_status = GasStatus::new(gas_schedule, gas_left);
            gas_status.set_metering(false);
            gas_status
        };
        let mut session: SessionAdapter<_> = self.move_vm.new_session(remote_cache).into();

        // init_script doesn't need run epilogue
        if remote_cache.is_genesis() {
            return discard_error_vm_status(error_code);
        }

        match TransactionStatus::from(error_code.clone()) {
            TransactionStatus::Keep(status) => {
                if let Err(e) = self.run_epilogue(&mut session, &mut gas_status, txn_data, false) {
                    return discard_error_vm_status(e);
                }
                let txn_output = get_transaction_output(
                    &mut (),
                    session,
                    &gas_status,
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

impl TransactionBlock {
    pub fn type_name(&self) -> &str {
        match self {
            TransactionBlock::UserTransaction(_) => "UserTransaction",
            TransactionBlock::BlockPrologue(_) => "BlockMetadata",
        }
    }
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

pub(crate) fn charge_global_write_gas_usage<R: MoveResolver>(
    cost_strategy: &mut GasStatus,
    session: &SessionAdapter<R>,
    sender: &AccountAddress,
) -> Result<(), VMStatus> {
    let total_cost = session.as_ref().num_mutated_accounts(sender)
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

    for (addr, account_changeset) in changeset.into_inner() {
        let (modules, resources) = account_changeset.into_inner();
        for (struct_tag, blob_opt) in resources {
            let ap = ap_cache.get_resource_path(addr, struct_tag);
            let op = match blob_opt {
                None => WriteOp::Deletion,
                Some(blob) => WriteOp::Value(blob),
            };
            ops.push((ap, op))
        }

        for (name, blob_opt) in modules {
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

pub(crate) fn get_transaction_output<A: AccessPathCache, R: MoveResolver>(
    ap_cache: &mut A,
    session: SessionAdapter<R>,
    cost_strategy: &GasStatus,
    max_gas_amount: GasUnits<GasCarrier>,
    status: KeptVMStatus,
) -> Result<TransactionOutput, VMStatus> {
    let gas_used: u64 = max_gas_amount.sub(cost_strategy.remaining_gas()).get();

    let (changeset, events) = Into::<Session<R>>::into(session)
        .finish()
        .map_err(|e| e.into_vm_status())?;
    let (write_set, events) = convert_changeset_and_events_cached(ap_cache, changeset, events)?;
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
