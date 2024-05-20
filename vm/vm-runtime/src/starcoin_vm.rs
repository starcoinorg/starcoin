// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path_cache::AccessPathCache;
use crate::adapter_common::{
    discard_error_output, discard_error_vm_status, PreprocessedTransaction, VMAdapter,
};
use crate::data_cache::{AsMoveResolver, RemoteStorage, StateViewCache};
use crate::errors::{
    convert_normal_success_epilogue_error, convert_prologue_runtime_error, error_split,
};
use crate::move_vm_ext::{MoveResolverExt, MoveVmExt, SessionId, SessionOutput};
use anyhow::{bail, format_err, Error, Result};
use move_core_types::gas_algebra::{InternalGasPerByte, NumBytes};
use move_core_types::vm_status::StatusCode::VALUE_SERIALIZATION_ERROR;
use move_table_extension::NativeTableContext;
use move_vm_runtime::move_vm_adapter::{PublishModuleBundleOption, SessionAdapter};
use move_vm_runtime::session::Session;
use num_cpus;
use once_cell::sync::OnceCell;
use starcoin_config::genesis_config::G_LATEST_GAS_PARAMS;
use starcoin_crypto::HashValue;
use starcoin_gas::{NativeGasParameters, StarcoinGasMeter, StarcoinGasParameters};
use starcoin_gas_algebra_ext::{
    CostTable, FromOnChainGasSchedule, Gas, GasConstants, GasCost, InitialGasSchedule,
};
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
};
use starcoin_vm_types::access::{ModuleAccess, ScriptAccess};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::account_config::upgrade::UpgradeEvent;
use starcoin_vm_types::account_config::{
    core_code_address, genesis_address, ModuleUpgradeStrategy, TwoPhaseUpgradeV2Resource,
    G_EPILOGUE_NAME, G_EPILOGUE_V2_NAME, G_PROLOGUE_NAME,
};
use starcoin_vm_types::errors::VMResult;
use starcoin_vm_types::file_format::{CompiledModule, CompiledScript};
use starcoin_vm_types::gas_schedule::G_LATEST_GAS_COST_TABLE;
use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::identifier::IdentStr;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::on_chain_config::{
    FlexiDagConfig, GasSchedule, MoveLanguageVersion, G_GAS_CONSTANTS_IDENTIFIER,
    G_INSTRUCTION_SCHEDULE_IDENTIFIER, G_NATIVE_SCHEDULE_IDENTIFIER, G_VM_CONFIG_IDENTIFIER,
};
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_view::StateReaderExt;
use starcoin_vm_types::transaction::{DryRunTransaction, Package, TransactionPayloadType};
use starcoin_vm_types::transaction_metadata::TransactionPayloadMetadata;
use starcoin_vm_types::value::{serialize_values, MoveValue};
use starcoin_vm_types::vm_status::KeptVMStatus;
use starcoin_vm_types::{
    errors::Location,
    language_storage::TypeTag,
    on_chain_config::{OnChainConfig, VMConfig, Version},
    state_view::StateView,
    transaction_metadata::TransactionMetadata,
    vm_status::{StatusCode, VMStatus},
};
use std::cmp::min;
use std::sync::Arc;

static EXECUTION_CONCURRENCY_LEVEL: OnceCell<usize> = OnceCell::new();

#[cfg(feature = "metrics")]
use crate::metrics::VMMetrics;
use crate::VMExecutor;

#[derive(Clone)]
#[allow(clippy::upper_case_acronyms)]
/// Wrapper of MoveVM
pub struct StarcoinVM {
    move_vm: Arc<MoveVmExt>,
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

/// marking of stdlib version which includes vmconfig upgrades.
const VMCONFIG_UPGRADE_VERSION_MARK: u64 = 10;
const FLEXI_DAG_UPGRADE_VERSION_MARK: u64 = 12;
// const GAS_SCHEDULE_UPGRADE_VERSION_MARK: u64 = 12;

impl StarcoinVM {
    #[cfg(feature = "metrics")]
    pub fn new(metrics: Option<VMMetrics>) -> Self {
        let gas_params = StarcoinGasParameters::initial();
        let native_params = gas_params.natives.clone();
        let inner = MoveVmExt::new(native_params.clone())
            .expect("should be able to create Move VM; check if there are duplicated natives");
        Self {
            move_vm: Arc::new(inner),
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
        let inner = MoveVmExt::new(native_params.clone())
            .expect("should be able to create Move VM; check if there are duplicated natives");
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

    pub fn load_configs<S: StateView>(&mut self, state: &S) -> Result<(), Error> {
        if state.is_genesis() {
            self.vm_config = Some(VMConfig {
                gas_schedule: G_LATEST_GAS_COST_TABLE.clone(),
            });
            self.version = Some(Version { major: 1 });
            self.gas_schedule = Some(GasSchedule::from(&G_LATEST_GAS_COST_TABLE.clone()));

            #[cfg(feature = "print_gas_info")]
            self.gas_schedule.as_ref().unwrap().info("from is_genesis");
        } else {
            self.load_configs_impl(state)?;
        }

        match self.gas_schedule.as_ref() {
            None => {
                bail!("failed to load gas schedule!");
            }
            Some(gs) => {
                let gas_params =
                    StarcoinGasParameters::from_on_chain_gas_schedule(&gs.clone().to_btree_map());
                if let Some(ref params) = gas_params {
                    if params.natives != self.native_params {
                        debug!("update native_params");
                        match Arc::get_mut(&mut self.move_vm) {
                            None => {
                                bail!("failed to get move vm when load config");
                            }
                            Some(mv) => {
                                mv.update_native_functions(params.clone().natives)?;
                            }
                        }
                        self.native_params = params.natives.clone();
                    }
                    self.gas_params = gas_params;
                }
            }
        }
        Ok(())
    }

    fn load_configs_impl<S: StateView>(&mut self, state: &S) -> Result<(), Error> {
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
            let _message;
            (self.gas_schedule, _message) = if stdlib_version
                < StdlibVersion::Version(VMCONFIG_UPGRADE_VERSION_MARK)
            {
                debug!(
                    "stdlib version: {}, fetch VMConfig from onchain resource",
                    stdlib_version
                );
                let gas_cost_table = VMConfig::fetch_config(&remote_storage)?
                    .ok_or_else(|| format_err!("Load VMConfig fail, VMConfig resource not exist."))?
                    .gas_schedule;
                (
                    Some(GasSchedule::from(&gas_cost_table)),
                    "gas schedule from VMConfig",
                )
            } else {
                debug!(
                    "stdlib version: {}, fetch VMConfig from onchain module",
                    stdlib_version
                );
                let instruction_schedule = {
                    let data = self
                        .execute_readonly_function_internal(
                            state,
                            &ModuleId::new(core_code_address(), G_VM_CONFIG_IDENTIFIER.to_owned()),
                            G_INSTRUCTION_SCHEDULE_IDENTIFIER.as_ident_str(),
                            vec![],
                            vec![],
                            false,
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
                        .execute_readonly_function_internal(
                            state,
                            &ModuleId::new(core_code_address(), G_VM_CONFIG_IDENTIFIER.to_owned()),
                            G_NATIVE_SCHEDULE_IDENTIFIER.as_ident_str(),
                            vec![],
                            vec![],
                            false,
                        )?
                        .pop()
                        .ok_or_else(|| {
                            anyhow::anyhow!("Expect 0x1::VMConfig::native_schedule() return value")
                        })?;
                    bcs_ext::from_bytes::<Vec<GasCost>>(&data)?
                };
                let gas_constants = {
                    let data = self
                        .execute_readonly_function_internal(
                            state,
                            &ModuleId::new(core_code_address(), G_VM_CONFIG_IDENTIFIER.to_owned()),
                            G_GAS_CONSTANTS_IDENTIFIER.as_ident_str(),
                            vec![],
                            vec![],
                            false,
                        )?
                        .pop()
                        .ok_or_else(|| {
                            anyhow::anyhow!("Expect 0x1::VMConfig::gas_constants() return value")
                        })?;
                    bcs_ext::from_bytes::<GasConstants>(&data)?
                };
                let cost_table = CostTable {
                    instruction_table: instruction_schedule,
                    native_table: native_schedule,
                    gas_constants,
                };
                (
                    Some(GasSchedule::from(&cost_table)),
                    "gas schedule from VMConfig",
                )
            };
            if stdlib_version >= StdlibVersion::Version(FLEXI_DAG_UPGRADE_VERSION_MARK) {
                self.flexi_dag_config = FlexiDagConfig::fetch_config(&remote_storage)?;
                debug!(
                    "stdlib version: {}, fetch flexi_dag_config {:?} from FlexiDagConfig module",
                    stdlib_version, self.flexi_dag_config,
                );
            }
            #[cfg(feature = "print_gas_info")]
            match self.gas_schedule.as_ref() {
                None => {
                    bail!("failed to load the gas schedule when trying to print its info");
                }
                Some(gs) => {
                    gs.info(_message);
                }
            }
        }
        Ok(())
    }

    pub fn get_flexidag_config(&self) -> Result<FlexiDagConfig, VMStatus> {
        self.flexi_dag_config
            .ok_or(VMStatus::Error(StatusCode::VM_STARTUP_FAILURE))
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
        let txn_gas_params = &self.get_gas_parameters()?.txn;
        let raw_bytes_len = txn_data.transaction_size;
        // The transaction is too large.
        if raw_bytes_len > txn_gas_params.max_transaction_size_in_bytes {
            warn!(
                "[VM] Transaction size too big {} (max {})",
                raw_bytes_len, txn_gas_params.max_transaction_size_in_bytes
            );
            return Err(VMStatus::Error(StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE));
        }

        // The submitted max gas units that the transaction can consume is greater than the
        // maximum number of gas units bound that we have set for any
        // transaction.
        if txn_data.max_gas_amount() > txn_gas_params.maximum_number_of_gas_units {
            warn!(
                "[VM] Gas unit error; max {}, submitted {}, with scaling_factor {}",
                txn_gas_params.maximum_number_of_gas_units,
                txn_data.max_gas_amount(),
                txn_gas_params.gas_unit_scaling_factor
            );
            return Err(VMStatus::Error(
                StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND,
            ));
        }

        // The submitted transactions max gas units needs to be at least enough to cover the
        // intrinsic cost of the transaction as calculated against the size of the
        // underlying `RawTransaction`
        let intrinsic_gas = txn_gas_params
            .calculate_intrinsic_gas(raw_bytes_len)
            .to_unit_round_up_with_params(txn_gas_params);
        if txn_data.max_gas_amount() < intrinsic_gas {
            warn!(
                "[VM] Gas unit error; min {}, submitted {}, with scaling_factor {}",
                intrinsic_gas,
                txn_data.max_gas_amount(),
                txn_gas_params.gas_unit_scaling_factor
            );
            return Err(VMStatus::Error(
                StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS,
            ));
        }

        // The submitted gas price is less than the minimum gas unit price set by the VM.
        // NB: MIN_PRICE_PER_GAS_UNIT may equal zero, but need not in the future. Hence why
        // we turn off the clippy warning.
        #[allow(clippy::absurd_extreme_comparisons)]
        let below_min_bound = txn_data.gas_unit_price() < txn_gas_params.min_price_per_gas_unit;
        if below_min_bound {
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                txn_gas_params.min_price_per_gas_unit,
                txn_data.gas_unit_price()
            );
            return Err(VMStatus::Error(StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND));
        }

        // The submitted gas price is greater than the maximum gas unit price set by the VM.
        if txn_data.gas_unit_price() > txn_gas_params.max_price_per_gas_unit {
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                txn_gas_params.max_price_per_gas_unit,
                txn_data.gas_unit_price()
            );
            return Err(VMStatus::Error(StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND));
        }
        Ok(())
    }

    fn verify_transaction_impl<S: StateView>(
        &mut self,
        transaction: &SignatureCheckedTransaction,
        remote_cache: &StateViewCache<S>,
    ) -> Result<(), VMStatus> {
        let txn_data = TransactionMetadata::new(transaction)?;
        let data_cache = remote_cache.as_move_resolver();
        let mut session: SessionAdapter<_> = self
            .move_vm
            .new_session(&data_cache, SessionId::txn(transaction))
            .into();
        let gas_params = self.get_gas_parameters()?;
        let mut gas_meter = StarcoinGasMeter::new(gas_params.clone(), txn_data.max_gas_amount());
        gas_meter.set_metering(false);
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
                        &mut gas_meter,
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
                        txn_data.sender(),
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
                        txn_data.sender(),
                    )
                    .map_err(|e| e.into_vm_status())?;
            }
        }
        self.run_prologue(&mut session, &mut gas_meter, &txn_data)
    }

    pub fn verify_transaction<S: StateView>(
        &mut self,
        state_view: &S,
        txn: SignedUserTransaction,
    ) -> Option<VMStatus> {
        #[cfg(feature = "metrics")]
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

    fn only_new_module_strategy<S: StateView>(
        remote_cache: &StateViewCache<S>,
        package_address: AccountAddress,
    ) -> Result<bool> {
        let strategy_access_path = access_path_for_module_upgrade_strategy(package_address);
        if let Some(data) =
            remote_cache.get_state_value(&StateKey::AccessPath(strategy_access_path))?
        {
            Ok(bcs_ext::from_bytes::<ModuleUpgradeStrategy>(&data)?.only_new_module())
        } else {
            Ok(false)
        }
    }

    fn is_enforced<S: StateView>(
        remote_cache: &StateViewCache<S>,
        package_address: AccountAddress,
    ) -> Result<bool> {
        let chain_id = remote_cache.get_chain_id()?;
        let block_number = remote_cache
            .get_block_metadata_v2()
            .and_then(|v2| match v2 {
                Some(meta) => Ok(meta.number),
                None => remote_cache.get_block_metadata().map(|v| v.number),
            })?;

        // from mainnet after 8015088 and barnard after 8311392, we disable enforce upgrade
        if package_address == genesis_address()
            || (chain_id.is_main() && block_number < 8015088)
            || (chain_id.is_barnard() && block_number < 8311392)
        {
            let two_phase_upgrade_v2_path = access_path_for_two_phase_upgrade_v2(package_address);
            if let Some(data) =
                remote_cache.get_state_value(&StateKey::AccessPath(two_phase_upgrade_v2_path))?
            {
                let enforced = bcs_ext::from_bytes::<TwoPhaseUpgradeV2Resource>(&data)?.enforced();
                Ok(enforced)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn execute_package<S: MoveResolverExt + StateView>(
        &self,
        mut session: SessionAdapter<S>,
        gas_meter: &mut StarcoinGasMeter,
        txn_data: &TransactionMetadata,
        package: &Package,
        storage: &S,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let data_cache = StateViewCache::new(storage);
        {
            // Run the validation logic
            gas_meter.set_metering(false);

            // // genesis txn skip check gas and txn prologue.
            if !data_cache.is_genesis() {
                //let _timer = TXN_VERIFICATION_SECONDS.start_timer();
                self.check_gas(txn_data)?;
                self.run_prologue(&mut session, gas_meter, txn_data)?;
            }
        }
        {
            // Genesis txn not enable gas charge.
            if !data_cache.is_genesis() {
                gas_meter.set_metering(true);
            }

            gas_meter
                .charge_intrinsic_gas_for_transaction(txn_data.transaction_size())
                .map_err(|e| e.into_vm_status())?;
            let package_address = package.package_address();
            for module in package.modules() {
                if let Ok(compiled_module) = CompiledModule::deserialize(module.code()) {
                    // check module's bytecode version.
                    self.check_move_version(compiled_module.version() as u64)?;
                };
            }

            let enforced = match Self::is_enforced(&data_cache, package_address) {
                Ok(is_enforced) => is_enforced,
                _ => false,
            };
            let only_new_module = match Self::only_new_module_strategy(&data_cache, package_address)
            {
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
                     gas_meter,
                    PublishModuleBundleOption {
                        force_publish: enforced,
                        only_new_module,
                    },
                )
                .map_err(|e| {
                    warn!("[VM] execute_package error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    e.into_vm_status()
                })?;

            // after publish the modules, we need to clear loader cache, to make init script function and
            // epilogue use the new modules.
            // clear logic move in publish_module_bundle_with_option

            if let Some(init_script) = package.init_script() {
                let genesis_address = genesis_address();
                // If package owner is genesis, then init_script will run using the genesis address
                // instead of the txn sender address. It provides the opportunity to add new resource
                // under the genesis address through DAO.
                let sender = if package_address == genesis_address {
                    gas_meter.set_metering(false);
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
                    .execute_entry_function(
                        init_script.module(),
                        init_script.function(),
                        init_script.ty_args().to_vec(),
                        init_script.args().to_vec(),
                        gas_meter,
                        sender,
                    )
                    .map_err(|e| e.into_vm_status())?;
            }
            charge_global_write_gas_usage(gas_meter, &session, &txn_data.sender())?;

            gas_meter.set_metering(false);
            self.success_transaction_cleanup(session, gas_meter, txn_data)
        }
    }

    fn execute_script_or_script_function<S: MoveResolverExt>(
        &self,
        mut session: SessionAdapter<S>,
        gas_meter: &mut StarcoinGasMeter,
        txn_data: &TransactionMetadata,
        payload: &TransactionPayload,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        // Run the validation logic
        {
            gas_meter.set_metering(false);
            self.check_gas(txn_data)?;
            self.run_prologue(&mut session, gas_meter, txn_data)?;
        }

        // Run the execution logic
        {
            //let _timer = TXN_EXECUTION_SECONDS.start_timer();
            gas_meter.set_metering(true);
            gas_meter
                .charge_intrinsic_gas_for_transaction(txn_data.transaction_size())
                .map_err(|e| e.into_vm_status())?;
            match payload {
                TransactionPayload::Script(script) => {
                    // we only use the ok path, let move vm handle the wrong path.
                    if let Ok(s) = CompiledScript::deserialize(script.code()) {
                        self.check_move_version(s.version() as u64)?;
                    };
                    debug!("TransactionPayload::{:?}", script);
                    session.execute_script(
                        script.code().to_vec(),
                        script.ty_args().to_vec(),
                        script.args().to_vec(),
                        gas_meter,
                        txn_data.sender(),
                    )
                }
                TransactionPayload::ScriptFunction(script_function) => {
                    debug!("TransactionPayload::{:?}", script_function);
                    session.execute_entry_function(
                        script_function.module(),
                        script_function.function(),
                        script_function.ty_args().to_vec(),
                        script_function.args().to_vec(),
                        gas_meter,
                        txn_data.sender(),
                    )
                }
                TransactionPayload::Package(_) => {
                    return Err(VMStatus::Error(StatusCode::UNREACHABLE));
                }
            }
            .map_err(|e|
                {
                    warn!("[VM] execute_script_function error, status_type: {:?}, status_code:{:?}, message:{:?}, location:{:?}", e.status_type(), e.major_status(), e.message(), e.location());
                    e.into_vm_status()
                })?;
            charge_global_write_gas_usage(gas_meter, &session, &txn_data.sender())?;

            self.success_transaction_cleanup(session, gas_meter, txn_data)
        }
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_prologue<R: MoveResolverExt>(
        &self,
        session: &mut SessionAdapter<R>,
        gas_meter: &mut StarcoinGasMeter,
        txn_data: &TransactionMetadata,
    ) -> Result<(), VMStatus> {
        let genesis_address = genesis_address();
        let gas_token_ty =
            TypeTag::Struct(Box::new(txn_data.gas_token_code().try_into().map_err(
                |_e| VMStatus::Error(StatusCode::BAD_TRANSACTION_FEE_CURRENCY),
            )?));
        let txn_sequence_number = txn_data.sequence_number();
        let authentication_key_preimage = txn_data.authentication_key_preimage().to_vec();
        let txn_gas_price = u64::from(txn_data.gas_unit_price());
        let txn_max_gas_amount = u64::from(txn_data.max_gas_amount());
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
            .execute_function_bypass_visibility(
                &account_config::G_TRANSACTION_MANAGER_MODULE,
                &G_PROLOGUE_NAME,
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
                gas_meter,
            )
            .map(|_return_vals| ())
            .or_else(convert_prologue_runtime_error)
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_epilogue<R: MoveResolverExt>(
        &self,
        session: &mut SessionAdapter<R>,
        gas_meter: &mut StarcoinGasMeter,
        txn_data: &TransactionMetadata,
        success: bool,
    ) -> Result<(), VMStatus> {
        let genesis_address = genesis_address();
        let gas_token_ty =
            TypeTag::Struct(Box::new(txn_data.gas_token_code().try_into().map_err(
                |_e| VMStatus::Error(StatusCode::BAD_TRANSACTION_FEE_CURRENCY),
            )?));
        let txn_sequence_number = txn_data.sequence_number();
        let txn_authentication_key_preimage = txn_data.authentication_key_preimage().to_vec();
        let txn_gas_price = u64::from(txn_data.gas_unit_price());
        let txn_max_gas_amount = u64::from(txn_data.max_gas_amount());
        let gas_remaining = u64::from(gas_meter.balance());
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
                &G_EPILOGUE_V2_NAME,
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
                &G_EPILOGUE_NAME,
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
            .execute_function_bypass_visibility(
                &account_config::G_TRANSACTION_MANAGER_MODULE,
                function_name,
                vec![gas_token_ty],
                args,
                gas_meter,
            )
            .map(|_return_vals| ())
            .or_else(convert_normal_success_epilogue_error)
    }

    fn process_block_metadata<S: MoveResolverExt>(
        &self,
        storage: &S,
        block_metadata: BlockMetadata,
    ) -> Result<TransactionOutput, VMStatus> {
        #[cfg(testing)]
        info!("process_block_meta begin");
        let stdlib_version = self.version.clone().map(|v| v.into_stdlib_version());
        let txn_sender = account_config::genesis_address();
        // always use 0 gas for system.
        let max_gas_amount: Gas = 0.into();
        // let mut gas_meter = UnmeteredGasMeter;
        // for debug
        let mut gas_meter = StarcoinGasMeter::new(StarcoinGasParameters::zeros(), max_gas_amount);
        gas_meter.set_metering(false);
        let session_id = SessionId::block_meta(&block_metadata);
        let (
            parent_id,
            timestamp,
            author,
            author_auth_key,
            uncles,
            number,
            chain_id,
            parent_gas_used,
            parents_hash,
        ) = block_metadata.into_inner();
        let mut function_name = &account_config::G_BLOCK_PROLOGUE_NAME;
        let mut args_vec = vec![
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
        ];
        if let Some(version) = stdlib_version {
            if version >= StdlibVersion::Version(FLEXI_DAG_UPGRADE_VERSION_MARK) {
                args_vec.push(MoveValue::vector_u8(
                    bcs_ext::to_bytes(&parents_hash.unwrap_or_default())
                        .or(Err(VMStatus::Error(VALUE_SERIALIZATION_ERROR)))?,
                ));
                function_name = &account_config::G_BLOCK_PROLOGUE_V2_NAME;
            }
        }
        let args = serialize_values(&args_vec);
        let mut session: SessionAdapter<_> = self.move_vm.new_session(storage, session_id).into();
        session
            .as_mut()
            .execute_function_bypass_visibility(
                &account_config::G_TRANSACTION_MANAGER_MODULE,
                function_name,
                vec![],
                args,
                &mut gas_meter,
            )
            .map(|_return_vals| ())
            .or_else(convert_prologue_runtime_error)?;
        #[cfg(testing)]
        info!("process_block_meta end");
        get_transaction_output(
            &mut (),
            session,
            0.into(),
            max_gas_amount,
            KeptVMStatus::Executed,
        )
    }

    fn execute_user_transaction<S: MoveResolverExt + StateView>(
        &self,
        storage: &S,
        txn: SignedUserTransaction,
    ) -> (VMStatus, TransactionOutput) {
        let txn_data = match TransactionMetadata::new(&txn) {
            Ok(txn_data) => txn_data,
            Err(e) => {
                return discard_error_vm_status(e);
            }
        };
        let gas_params = match self.get_gas_parameters() {
            Ok(gas_params) => gas_params,
            Err(e) => {
                if storage.is_genesis() {
                    &G_LATEST_GAS_PARAMS
                } else {
                    return discard_error_vm_status(e);
                }
            }
        };

        let session: SessionAdapter<_> = self
            .move_vm
            .new_session(storage, SessionId::txn_meta(&txn_data))
            .into();
        let mut gas_meter = StarcoinGasMeter::new(gas_params.clone(), txn_data.max_gas_amount());
        gas_meter.set_metering(false);
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
                            session,
                            &mut gas_meter,
                            &txn_data,
                            payload,
                        ),
                    TransactionPayload::Package(p) => {
                        self.execute_package(session, &mut gas_meter, &txn_data, p, storage)
                    }
                };
                match result {
                    Ok(status_and_output) => {
                        log_vm_status(
                            txn.id(),
                            &txn_data,
                            &status_and_output.0,
                            Some(&status_and_output.1),
                        );
                        status_and_output
                    }
                    Err(err) => {
                        let txn_status = TransactionStatus::from(err.clone());
                        log_vm_status(txn.id(), &txn_data, &err, None);
                        if txn_status.is_discarded() {
                            discard_error_vm_status(err)
                        } else {
                            self.failed_transaction_cleanup(err, &mut gas_meter, &txn_data, storage)
                        }
                    }
                }
            }
            Err(e) => discard_error_vm_status(e),
        }
    }

    pub fn dry_run_transaction<S: MoveResolverExt + StateView>(
        &mut self,
        storage: &S,
        txn: DryRunTransaction,
    ) -> Result<(VMStatus, TransactionOutput)> {
        // TODO load config by config change event.
        self.load_configs(&storage)?;

        let gas_params = match self.get_gas_parameters() {
            Ok(gas_params) => gas_params,
            Err(e) => {
                if storage.is_genesis() {
                    &G_LATEST_GAS_PARAMS
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
        let session = self
            .move_vm
            .new_session(storage, SessionId::txn_meta(&txn_data))
            .into();
        let mut gas_meter = StarcoinGasMeter::new(gas_params.clone(), txn_data.max_gas_amount());
        gas_meter.set_metering(false);
        let result = match txn.raw_txn.payload() {
            payload @ TransactionPayload::Script(_)
            | payload @ TransactionPayload::ScriptFunction(_) => {
                self.execute_script_or_script_function(session, &mut gas_meter, &txn_data, payload)
            }
            TransactionPayload::Package(p) => {
                self.execute_package(session, &mut gas_meter, &txn_data, p, storage)
            }
        };
        Ok(match result {
            Ok(status_and_output) => status_and_output,
            Err(err) => {
                let txn_status = TransactionStatus::from(err.clone());
                if txn_status.is_discarded() {
                    discard_error_vm_status(err)
                } else {
                    self.failed_transaction_cleanup(err, &mut gas_meter, &txn_data, storage)
                }
            }
        })
    }

    fn check_reconfigure<S: StateView>(
        &mut self,
        state_view: &S,
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
    pub fn execute_block_transactions<S: StateView>(
        &mut self,
        storage: &S,
        transactions: Vec<Transaction>,
        block_gas_limit: Option<u64>,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>, VMStatus> {
        let mut data_cache = StateViewCache::new(storage);
        let mut result = vec![];

        // TODO load config by config change event
        self.load_configs(&data_cache)
            .map_err(|_err| VMStatus::Error(StatusCode::STORAGE_ERROR))?;

        let mut gas_left = block_gas_limit.unwrap_or(u64::MAX);
        let blocks = chunk_block_transactions(transactions);

        'outer: for block in blocks {
            #[cfg(feature = "metrics")]
            let txn_type_name = block.type_name().to_string();
            match block {
                TransactionBlock::UserTransaction(txns) => {
                    for transaction in txns {
                        #[cfg(feature = "metrics")]
                        let timer = self.metrics.as_ref().map(|metrics| {
                            metrics
                                .vm_txn_exe_time
                                .with_label_values(&[txn_type_name.as_str()])
                                .start_timer()
                        });

                        let gas_unit_price = transaction.gas_unit_price();

                        let (status, output) = self
                            .execute_user_transaction(&data_cache.as_move_resolver(), transaction);

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
                            // Push write set to write set
                            data_cache.push_write_set(output.write_set())
                        }
                        // TODO load config by config change event
                        self.check_reconfigure(&data_cache, &output)
                            .map_err(|_err| VMStatus::Error(StatusCode::STORAGE_ERROR))?;

                        #[cfg(feature = "metrics")]
                        if let Some(timer) = timer {
                            timer.observe_duration();
                        }
                        #[cfg(feature = "metrics")]
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
                    #[cfg(feature = "metrics")]
                    let timer = self.metrics.as_ref().map(|metrics| {
                        metrics
                            .vm_txn_exe_time
                            .with_label_values(&[txn_type_name.as_str()])
                            .start_timer()
                    });

                    let (status, output) = match self
                        .process_block_metadata(&data_cache.as_move_resolver(), block_metadata)
                    {
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
                        // Push write set to write set
                        data_cache.push_write_set(output.write_set())
                    }
                    #[cfg(feature = "metrics")]
                    if let Some(timer) = timer {
                        timer.observe_duration();
                    }
                    #[cfg(feature = "metrics")]
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

    pub fn execute_readonly_function<S: StateView>(
        &mut self,
        state_view: &S,
        module: &ModuleId,
        function_name: &IdentStr,
        type_params: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<Vec<u8>>, VMStatus> {
        self.execute_readonly_function_internal(
            state_view,
            module,
            function_name,
            type_params,
            args,
            true,
        )
    }

    fn execute_readonly_function_internal<S: StateView>(
        &mut self,
        state_view: &S,
        module: &ModuleId,
        function_name: &IdentStr,
        type_params: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
        check_gas: bool,
    ) -> Result<Vec<Vec<u8>>, VMStatus> {
        #[cfg(feature = "metrics")]
        let _timer = self.metrics.as_ref().map(|metrics| {
            metrics
                .vm_txn_exe_time
                .with_label_values(&["execute_readonly_function"])
                .start_timer()
        });
        let data_cache = state_view.as_move_resolver();

        let mut gas_meter = if check_gas {
            if let Err(err) = self.load_configs(state_view) {
                warn!(
                    "Load config error at execute_readonly_function_internal: {}",
                    err
                );
                return Err(VMStatus::Error(StatusCode::VM_STARTUP_FAILURE));
            }
            let gas_params = self.get_gas_parameters()?;
            let mut gas_meter = StarcoinGasMeter::new(
                G_LATEST_GAS_PARAMS.clone(),
                gas_params.txn.maximum_number_of_gas_units,
            );
            gas_meter.set_metering(true);
            gas_meter
        } else {
            let max_gas_amount: Gas = 0.into();
            let mut gas_meter =
                StarcoinGasMeter::new(StarcoinGasParameters::zeros(), max_gas_amount);
            gas_meter.set_metering(false);
            gas_meter
        };
        let mut session = self.move_vm.new_session(&data_cache, SessionId::void());
        let result = session
            .execute_function_bypass_visibility(
                module,
                function_name,
                type_params,
                args,
                &mut gas_meter,
            )
            .map_err(|e| e.into_vm_status())?
            .return_values
            .into_iter()
            .map(|(a, _)| a)
            .collect();

        let (change_set, events, mut extensions) = session
            .finish_with_extensions()
            .map_err(|e| e.into_vm_status())?;
        let table_context: NativeTableContext = extensions.remove();
        let table_change_set = table_context
            .into_change_set()
            .map_err(|e| e.finish(Location::Undefined))?;
        // Ignore new table infos.
        // No table infos should be produced in readonly function.
        let (_table_infos, write_set, _events) = SessionOutput {
            change_set,
            events,
            table_change_set,
        }
        .into_change_set(&mut ())?;
        if !write_set.is_empty() {
            warn!("Readonly function {} changes state", function_name);
            return Err(VMStatus::Error(StatusCode::REJECTED_WRITE_SET));
        }
        Ok(result)
    }

    fn success_transaction_cleanup<R: MoveResolverExt>(
        &self,
        mut session: SessionAdapter<R>,
        gas_meter: &mut StarcoinGasMeter,
        txn_data: &TransactionMetadata,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        gas_meter.set_metering(false);
        self.run_epilogue(&mut session, gas_meter, txn_data, true)?;

        Ok((
            VMStatus::Executed,
            get_transaction_output(
                &mut (),
                session,
                gas_meter.balance(),
                txn_data.max_gas_amount,
                KeptVMStatus::Executed,
            )?,
        ))
    }

    fn failed_transaction_cleanup<S: MoveResolverExt + StateView>(
        &self,
        error_code: VMStatus,
        gas_meter: &mut StarcoinGasMeter,
        txn_data: &TransactionMetadata,
        storage: &S,
    ) -> (VMStatus, TransactionOutput) {
        gas_meter.set_metering(false);
        let mut session: SessionAdapter<_> = self
            .move_vm
            .new_session(storage, SessionId::txn_meta(txn_data))
            .into();

        // init_script doesn't need run epilogue
        if storage.is_genesis() {
            return discard_error_vm_status(error_code);
        }

        match TransactionStatus::from(error_code.clone()) {
            TransactionStatus::Keep(status) => {
                if let Err(e) = self.run_epilogue(&mut session, gas_meter, txn_data, false) {
                    return discard_error_vm_status(e);
                }
                let txn_output = get_transaction_output(
                    &mut (),
                    session,
                    gas_meter.balance(),
                    txn_data.max_gas_amount,
                    status,
                )
                .unwrap_or_else(|e| discard_error_vm_status(e).1);
                (error_code, txn_output)
            }
            TransactionStatus::Discard(status) => {
                (VMStatus::Error(status), discard_error_output(status))
            }
            TransactionStatus::Retry => unreachable!(),
        }
    }

    pub fn get_gas_parameters(&self) -> Result<&StarcoinGasParameters, VMStatus> {
        self.gas_params.as_ref().ok_or_else(|| {
            debug!("VM Startup Failed. Gas Parameters Not Found");
            VMStatus::Error(StatusCode::VM_STARTUP_FAILURE)
        })
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
        let mut vm = StarcoinVM::new(metrics);
        vm.execute_block_transactions(state_view, txns, block_gas_limit)
    }

    pub fn load_module<R: MoveResolverExt>(
        &self,
        module_id: &ModuleId,
        remote: &R,
    ) -> VMResult<Arc<CompiledModule>> {
        self.move_vm.load_module(module_id, remote)
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

pub(crate) fn charge_global_write_gas_usage<R: MoveResolverExt>(
    gas_meter: &mut StarcoinGasMeter,
    session: &SessionAdapter<R>,
    sender: &AccountAddress,
) -> Result<(), VMStatus> {
    let write_set_gas = u64::from(gas_meter.cal_write_set_gas());
    let total_cost = InternalGasPerByte::from(write_set_gas)
        * NumBytes::new(session.as_ref().num_mutated_accounts(sender));
    #[cfg(testing)]
    info!(
        "charge_global_write_gas_usage {} {}",
        total_cost,
        gas_meter.get_metering()
    );
    gas_meter
        .deduct_gas(total_cost)
        .map_err(|p_err| p_err.finish(Location::Undefined).into_vm_status())
}

pub(crate) fn get_transaction_output<A: AccessPathCache, R: MoveResolverExt>(
    ap_cache: &mut A,
    session: SessionAdapter<R>,
    gas_left: Gas,
    max_gas_amount: Gas,
    status: KeptVMStatus,
) -> Result<TransactionOutput, VMStatus> {
    // original code use sub, now we use checked_sub
    let gas_used = max_gas_amount
        .checked_sub(gas_left)
        .expect("Balance should always be less than or equal to max gas amount");
    let (change_set, events, mut extensions) =
        Into::<Session<R>>::into(session).finish_with_extensions()?;
    let table_context: NativeTableContext = extensions.remove();
    let table_change_set = table_context
        .into_change_set()
        .map_err(|e| e.finish(Location::Undefined))?;
    let (table_infos, write_set, events) = SessionOutput {
        change_set,
        events,
        table_change_set,
    }
    .into_change_set(ap_cache)?;
    Ok(TransactionOutput::new(
        table_infos,
        write_set,
        events,
        u64::from(gas_used),
        TransactionStatus::Keep(status),
    ))
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

// Executor external API
impl VMExecutor for StarcoinVM {
    /// Execute a block of `transactions`. The output vector will have the exact same length as the
    /// input vector. The discarded transactions will be marked as `TransactionStatus::Discard` and
    /// have an empty `WriteSet`. Also `state_view` is immutable, and does not have interior
    /// mutability. Writes to be applied to the data view are encoded in the write set part of a
    /// transaction output.
    fn execute_block(
        transactions: Vec<Transaction>,
        state_view: &impl StateView,
        block_gas_limit: Option<u64>,
        metrics: Option<VMMetrics>,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        let concurrency_level = Self::get_concurrency_level();
        if concurrency_level > 1 {
            let (result, _) = crate::parallel_executor::ParallelStarcoinVM::execute_block(
                transactions,
                state_view,
                concurrency_level,
                block_gas_limit,
                metrics,
            )?;
            // debug!("TurboSTM executor concurrency_level {}", concurrency_level);
            Ok(result)
        } else {
            let output = Self::execute_block_and_keep_vm_status(
                transactions,
                state_view,
                block_gas_limit,
                metrics,
            )?;
            Ok(output
                .into_iter()
                .map(|(_vm_status, txn_output)| txn_output)
                .collect())
        }
    }
}

impl VMAdapter for StarcoinVM {
    fn new_session<'r, R: MoveResolverExt>(
        &self,
        remote: &'r R,
        session_id: SessionId,
    ) -> SessionAdapter<'r, '_, R> {
        self.move_vm.new_session(remote, session_id).into()
    }

    fn check_signature(txn: SignedUserTransaction) -> Result<SignatureCheckedTransaction> {
        txn.check_signature()
    }

    fn should_restart_execution(output: &TransactionOutput) -> bool {
        // XXX FIXME YSG if GasSchedule.move UpgradeEvent
        for event in output.events() {
            if event.key().get_creator_address() == genesis_address()
                && (event.is::<UpgradeEvent>() || event.is::<ConfigChangeEvent<Version>>())
            {
                info!("should_restart_execution happen");
                return true;
            }
        }
        false
    }

    fn execute_single_transaction<S: MoveResolverExt + StateView>(
        &self,
        txn: &PreprocessedTransaction,
        data_cache: &S,
    ) -> Result<(VMStatus, TransactionOutput, Option<String>), VMStatus> {
        Ok(match txn {
            PreprocessedTransaction::UserTransaction(txn) => {
                let sender = txn.sender().to_string();
                let (vm_status, output) = self.execute_user_transaction(data_cache, *txn.clone());
                // XXX FIXME YSG
                // let gas_unit_price = transaction.gas_unit_price(); think about gas_used OutOfGas
                (vm_status, output, Some(sender))
            }
            PreprocessedTransaction::BlockMetadata(block_meta) => {
                let (vm_status, output) =
                    match self.process_block_metadata(data_cache, block_meta.clone()) {
                        Ok(output) => (VMStatus::Executed, output),
                        Err(vm_status) => discard_error_vm_status(vm_status),
                    };
                (vm_status, output, Some("block_meta".to_string()))
            }
        })
    }
}
