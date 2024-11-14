// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::access_path_cache::AccessPathCache;
use crate::data_cache::{AsMoveResolver, StateViewCache, StorageAdapter};
use crate::errors::{
    convert_normal_success_epilogue_error, convert_prologue_runtime_error, error_split,
};
use crate::move_vm_ext::{MoveVmExt, SessionExt, SessionId, StarcoinMoveResolver};
use crate::{
    default_gas_schedule, discard_error_output, discard_error_vm_status, PreprocessedTransaction,
};
use anyhow::{bail, format_err, Error, Result};
use move_core_types::gas_algebra::{InternalGasPerByte, NumBytes};
use move_core_types::move_resource::MoveStructType;
use move_core_types::vm_status::StatusCode::VALUE_SERIALIZATION_ERROR;
use move_vm_runtime::module_traversal::{TraversalContext, TraversalStorage};
use move_vm_runtime::move_vm_adapter::PublishModuleBundleOption;
use move_vm_types::gas::GasMeter;
use num_cpus;
use once_cell::sync::OnceCell;
use starcoin_config::genesis_config::G_LATEST_GAS_PARAMS;
use starcoin_crypto::HashValue;
use starcoin_gas_algebra::{CostTable, Gas, GasConstants, GasCost};
use starcoin_gas_meter::StarcoinGasMeter;
use starcoin_gas_schedule::{
    FromOnChainGasSchedule, InitialGasSchedule, NativeGasParameters, StarcoinGasParameters,
    LATEST_GAS_FEATURE_VERSION,
};
use starcoin_logger::prelude::*;
use starcoin_types::account_config::config_change::ConfigChangeEvent;
use starcoin_types::{
    account_config,
    block_metadata::BlockMetadata,
    transaction::{
        SignatureCheckedTransaction, SignedUserTransaction, Transaction, TransactionOutput,
        TransactionPayload, TransactionStatus,
    },
};
use starcoin_vm_runtime_types::storage::change_set_configs::ChangeSetConfigs;
use starcoin_vm_types::on_chain_config::{Features, TimedFeaturesBuilder};
use starcoin_vm_types::transaction::TransactionAuxiliaryData;
use starcoin_vm_types::{
    access::{ModuleAccess, ScriptAccess},
    account_address::AccountAddress,
    account_config::{
        core_code_address, genesis_address, upgrade::UpgradeEvent, ModuleUpgradeStrategy,
        TwoPhaseUpgradeV2Resource, G_EPILOGUE_NAME, G_PROLOGUE_NAME,
    },
    errors::{Location, PartialVMError, VMResult},
    file_format::{CompiledModule, CompiledScript},
    gas_schedule::G_LATEST_GAS_COST_TABLE,
    genesis_config::StdlibVersion,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
    on_chain_config::{
        FlexiDagConfig, GasSchedule, MoveLanguageVersion, OnChainConfig, VMConfig, Version,
        G_GAS_CONSTANTS_IDENTIFIER, G_INSTRUCTION_SCHEDULE_IDENTIFIER,
        G_NATIVE_SCHEDULE_IDENTIFIER, G_VM_CONFIG_MODULE_IDENTIFIER,
    },
    state_store::{state_key::StateKey, StateView, TStateView},
    state_view::StateReaderExt,
    transaction::{DryRunTransaction, Package, TransactionPayloadType},
    transaction_metadata::{TransactionMetadata, TransactionPayloadMetadata},
    value::{serialize_values, MoveValue},
    vm_status::{KeptVMStatus, StatusCode, VMStatus},
};
use std::{borrow::Borrow, cmp::min, sync::Arc};

static EXECUTION_CONCURRENCY_LEVEL: OnceCell<usize> = OnceCell::new();

#[cfg(feature = "metrics")]
use crate::metrics::VMMetrics;
use crate::{verifier, VMExecutor};

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
    pub fn new<S: StateView>(metrics: Option<VMMetrics>, state: &S) -> Self {
        Self::new_with_config(metrics, state, None)
    }

    #[cfg(feature = "metrics")]
    pub fn new_with_config<S: StateView>(
        metrics: Option<VMMetrics>,
        state: &S,
        chain_id: Option<u8>,
    ) -> Self {
        let chain_id = chain_id.unwrap_or_else(|| {
            state
                .get_chain_id()
                .expect("Failed to get chain id, please check statedb")
                .id()
        });
        let gas_params = StarcoinGasParameters::initial();
        let native_params = gas_params.natives.clone();
        // todo: double check if it's ok to use RemoteStorage as StarcoinMoveResolver
        let resolver = StorageAdapter::new(state);
        let inner = MoveVmExt::new(
            native_params.clone(),
            gas_params.vm.misc.clone(),
            LATEST_GAS_FEATURE_VERSION,
            chain_id,
            Features::default(),
            TimedFeaturesBuilder::enable_all().build(),
            &resolver,
        )
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
    pub fn new<S: StateView>(state: &S) -> Self {
        let chain_id = state.get_chain_id().id();
        let gas_params = StarcoinGasParameters::initial();
        let native_params = gas_params.natives.clone();
        // todo: double check if it's ok to use RemoteStorage as StarcoinMoveResolver
        let resolver = StorageAdapter::new(state);
        let inner = MoveVmExt::new(
            native_params.clone(),
            gas_params.vm.misc.clone(),
            1,
            chain_id,
            Features::default(),
            TimedFeaturesBuilder::enable_all().build(),
            resolver,
        )
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
            self.gas_schedule = Some(default_gas_schedule());

            #[cfg(feature = "print_gas_info")]
            self.gas_schedule.as_ref().unwrap().info("from is_genesis");
            return Ok(());
        }

        self.load_configs_impl(state)?;

        match self.gas_schedule.as_ref() {
            None => {
                bail!("failed to load gas schedule!");
            }
            Some(gs) => {
                // TODO(simon): retrive gas_feature_version from chain.
                let gas_feature_version = LATEST_GAS_FEATURE_VERSION;
                let gas_schdule_treemap = gs.clone().to_btree_map();
                let gas_params = StarcoinGasParameters::from_on_chain_gas_schedule(
                    &gas_schdule_treemap,
                    gas_feature_version,
                )
                .map_err(|e| format_err!("{e}"))?;

                // TODO(simon): do double check
                // if params.natives != self.native_params {
                debug!("update native_params");
                match Arc::get_mut(&mut self.move_vm) {
                    None => {
                        bail!("failed to get move vm when load config");
                    }
                    Some(mv) => {
                        let gas_params = gas_params.clone();
                        mv.update_native_functions(
                            gas_params.natives,
                            gas_params.vm.misc,
                            gas_feature_version,
                        )?;
                    }
                }
                self.native_params = gas_params.natives.clone();
                self.gas_params = Some(gas_params);
            }
        }
        Ok(())
    }

    fn load_configs_impl<S: StateView>(&mut self, state: &S) -> Result<(), Error> {
        let remote_storage = StorageAdapter::new(state);
        self.version = Version::fetch_config(&remote_storage);
        // move version can be none.
        self.move_version = MoveLanguageVersion::fetch_config(&remote_storage);

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
                (
                    VMConfig::fetch_config(&remote_storage)
                        .map(|v| GasSchedule::from(&v.gas_schedule)),
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
                            &ModuleId::new(
                                core_code_address(),
                                G_VM_CONFIG_MODULE_IDENTIFIER.to_owned(),
                            ),
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
                            &ModuleId::new(
                                core_code_address(),
                                G_VM_CONFIG_MODULE_IDENTIFIER.to_owned(),
                            ),
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
                            &ModuleId::new(
                                core_code_address(),
                                G_VM_CONFIG_MODULE_IDENTIFIER.to_owned(),
                            ),
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
                self.flexi_dag_config = FlexiDagConfig::fetch_config(&remote_storage);
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
            .ok_or(VMStatus::error(StatusCode::VM_STARTUP_FAILURE, None))
    }

    pub fn get_gas_schedule(&self) -> Result<&CostTable, VMStatus> {
        self.vm_config
            .as_ref()
            .map(|config| &config.gas_schedule)
            .ok_or(VMStatus::error(StatusCode::VM_STARTUP_FAILURE, None))
    }

    pub fn get_version(&self) -> Result<Version, VMStatus> {
        self.version
            .clone()
            .ok_or(VMStatus::error(StatusCode::VM_STARTUP_FAILURE, None))
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
                return Err(VMStatus::error(StatusCode::FEATURE_UNDER_GATING, None));
            }
        }
        Ok(())
    }

    fn check_gas(&self, txn_data: &TransactionMetadata) -> Result<(), VMStatus> {
        let txn_gas_params = &self.get_gas_parameters()?.vm.txn;
        let raw_bytes_len = txn_data.transaction_size;
        // The transaction is too large.
        if raw_bytes_len > txn_gas_params.max_transaction_size_in_bytes {
            warn!(
                "[VM] Transaction size too big {} (max {})",
                raw_bytes_len, txn_gas_params.max_transaction_size_in_bytes
            );
            return Err(VMStatus::error(
                StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE,
                None,
            ));
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
            return Err(VMStatus::error(
                StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND,
                None,
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
            return Err(VMStatus::error(
                StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS,
                None,
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
            return Err(VMStatus::error(
                StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND,
                None,
            ));
        }

        // The submitted gas price is greater than the maximum gas unit price set by the VM.
        if txn_data.gas_unit_price() > txn_gas_params.max_price_per_gas_unit {
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                txn_gas_params.max_price_per_gas_unit,
                txn_data.gas_unit_price()
            );
            return Err(VMStatus::error(
                StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND,
                None,
            ));
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
        let mut session = self
            .move_vm
            .new_session(&data_cache, SessionId::txn(transaction));
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
                            return Err(VMStatus::error(
                                StatusCode::FAILED_TO_DESERIALIZE_RESOURCE,
                                None,
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
            TransactionPayload::EntryFunction(s) => {
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
            Err(_) => return Some(VMStatus::error(StatusCode::INVALID_SIGNATURE, None)),
        };
        if let Err(err) = self.load_configs(state_view) {
            warn!("Load config error at verify_transaction: {}", err);
            return Some(VMStatus::error(StatusCode::VM_STARTUP_FAILURE, None));
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
        let key = StateKey::resource(&package_address, &ModuleUpgradeStrategy::struct_tag())?;
        if let Some(data) = remote_cache.get_state_value(&key)? {
            Ok(bcs_ext::from_bytes::<ModuleUpgradeStrategy>(data.bytes())?.only_new_module())
        } else {
            Ok(false)
        }
    }

    fn is_enforced<S: StateView>(
        remote_cache: &StateViewCache<S>,
        package_address: AccountAddress,
    ) -> Result<bool> {
        let chain_id = remote_cache.get_chain_id()?;
        let block_number = match remote_cache.get_block_metadata().map(|data| data.number) {
            Ok(n) => n,
            Err(_) => remote_cache.get_block_metadata().map(|v| v.number)?,
        };

        // from mainnet after 8015088 and barnard after 8311392, we disable enforce upgrade
        if package_address == genesis_address()
            || (chain_id.is_main() && block_number < 8015088)
            || (chain_id.is_barnard() && block_number < 8311392)
        {
            let key =
                StateKey::resource(&package_address, &TwoPhaseUpgradeV2Resource::struct_tag())?;
            if let Some(data) = remote_cache.get_state_value(&key)? {
                let enforced =
                    bcs_ext::from_bytes::<TwoPhaseUpgradeV2Resource>(data.bytes())?.enforced();
                Ok(enforced)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn execute_package<S: StarcoinMoveResolver + StateView>(
        &self,
        mut session: SessionExt,
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
                    return Err(VMStatus::error(
                        StatusCode::FAILED_TO_DESERIALIZE_RESOURCE,
                        None,
                    ));
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
                Self::validate_execute_entry_function(
                    &mut session,
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

    fn execute_script_or_script_function(
        &self,
        mut session: SessionExt,
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
                    Self::validate_execute_script(
                        &mut session,
                        script.code().to_vec(),
                        script.ty_args().to_vec(),
                        script.args().to_vec(),
                        gas_meter,
                        txn_data.sender(),
                    )
                }
                TransactionPayload::EntryFunction(script_function) => {
                    debug!("TransactionPayload::{:?}", script_function);
                    Self::validate_execute_entry_function(
                        &mut session,
                        script_function.module(),
                        script_function.function(),
                        script_function.ty_args().to_vec(),
                        script_function.args().to_vec(),
                        gas_meter,
                        txn_data.sender(),
                    )
                }
                TransactionPayload::Package(_) => {
                    return Err(VMStatus::error(StatusCode::UNREACHABLE, None));
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

    fn validate_execute_entry_function(
        session: &mut SessionExt,
        module: &ModuleId,
        function_name: &IdentStr,
        ty_args: Vec<TypeTag>,
        args: Vec<impl Borrow<[u8]>>,
        gas_meter: &mut impl GasMeter,
        sender: AccountAddress,
    ) -> VMResult<()> {
        let loaded_func = session.load_function(module, function_name, &ty_args)?;

        verifier::transaction_arg_validation::validate_combine_singer_and_args(
            session,
            vec![sender],
            &args,
            &loaded_func,
        )?;

        let final_args = SessionExt::check_and_rearrange_args_by_signer_position(
            loaded_func.borrow(),
            args.iter().map(|b| b.borrow().to_vec()).collect(),
            sender,
        )?;

        let tranversal_storage = TraversalStorage::new();
        session.execute_entry_function(
            loaded_func,
            final_args,
            gas_meter,
            &mut TraversalContext::new(&tranversal_storage),
        )
    }

    fn validate_execute_script(
        session: &mut SessionExt,
        script: impl Borrow<[u8]>,
        ty_args: Vec<TypeTag>,
        args: Vec<impl Borrow<[u8]>>,
        gas_meter: &mut impl GasMeter,
        sender: AccountAddress,
    ) -> VMResult<()> {
        let loaded_func = session.load_script(script.borrow(), ty_args.as_ref())?;

        verifier::transaction_arg_validation::validate_combine_singer_and_args(
            session,
            vec![sender],
            &args,
            &loaded_func,
        )?;

        let final_args = SessionExt::check_and_rearrange_args_by_signer_position(
            loaded_func.borrow(),
            args.iter().map(|b| b.borrow().to_vec()).collect(),
            sender,
        )?;

        let traversal_storage = TraversalStorage::new();
        session.execute_script(
            script,
            ty_args,
            final_args,
            gas_meter,
            &mut TraversalContext::new(&traversal_storage),
        )
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_prologue(
        &self,
        session: &mut SessionExt,
        gas_meter: &mut StarcoinGasMeter,
        txn_data: &TransactionMetadata,
    ) -> Result<(), VMStatus> {
        let genesis_address = genesis_address();
        let gas_token_ty =
            TypeTag::Struct(Box::new(txn_data.gas_token_code().try_into().map_err(
                |_e| VMStatus::error(StatusCode::BAD_TRANSACTION_FEE_CURRENCY, None),
            )?));
        info!(
            "StarcoinVM::run_prologue | Gas token data: {:?}",
            gas_token_ty
        );

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

        let traversal_storage = TraversalStorage::new();
        // Run prologue by genesis account
        session
            .execute_function_bypass_visibility(
                &account_config::G_TRANSACTION_VALIDATION_MODULE,
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
                &mut TraversalContext::new(&traversal_storage),
            )
            .map(|_return_vals| ())
            .or_else(convert_prologue_runtime_error)
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_epilogue(
        &self,
        session: &mut SessionExt,
        gas_meter: &mut StarcoinGasMeter,
        txn_data: &TransactionMetadata,
        success: bool,
    ) -> Result<(), VMStatus> {
        let genesis_address = genesis_address();
        let gas_token_ty =
            TypeTag::Struct(Box::new(txn_data.gas_token_code().try_into().map_err(
                |_e| VMStatus::error(StatusCode::BAD_TRANSACTION_FEE_CURRENCY, None),
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
        //let stdlib_version = self.get_version()?.into_stdlib_version();
        // Run epilogue by genesis account, second arg is txn sender.
        // From stdlib v5, the epilogue function add `txn_authentication_key_preimage` argument, change to epilogue_v2
        // let (function_name, args) = if stdlib_version > StdlibVersion::Version(4) {
        //     (
        //         &G_EPILOGUE_NAME,
        //         serialize_values(&vec![
        //             MoveValue::Signer(genesis_address),
        //             MoveValue::Address(txn_data.sender),
        //             MoveValue::U64(txn_sequence_number),
        //             MoveValue::vector_u8(txn_authentication_key_preimage),
        //             MoveValue::U64(txn_gas_price),
        //             MoveValue::U64(txn_max_gas_amount),
        //             MoveValue::U64(gas_remaining),
        //             MoveValue::U8(payload_type.into()),
        //             MoveValue::vector_u8(script_or_package_hash.to_vec()),
        //             MoveValue::Address(package_address),
        //             MoveValue::Bool(success),
        //         ]),
        //     )
        // } else {
        //     (
        //         &G_EPILOGUE_NAME,
        //         serialize_values(&vec![
        //             MoveValue::Signer(genesis_address),
        //             MoveValue::Address(txn_data.sender),
        //             MoveValue::U64(txn_sequence_number),
        //             MoveValue::U64(txn_gas_price),
        //             MoveValue::U64(txn_max_gas_amount),
        //             MoveValue::U64(gas_remaining),
        //             MoveValue::U8(payload_type.into()),
        //             MoveValue::vector_u8(script_or_package_hash.to_vec()),
        //             MoveValue::Address(package_address),
        //             MoveValue::Bool(success),
        //         ]),
        //     )
        // };
        let (function_name, args) = (
            &G_EPILOGUE_NAME,
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
        );

        let traversal_storage = TraversalStorage::new();
        session
            .execute_function_bypass_visibility(
                &account_config::G_TRANSACTION_VALIDATION_MODULE,
                function_name,
                vec![gas_token_ty],
                args,
                gas_meter,
                &mut TraversalContext::new(&traversal_storage),
            )
            .map(|_return_vals| ())
            .or_else(convert_normal_success_epilogue_error)
    }

    fn process_block_metadata<S: StarcoinMoveResolver>(
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
        let (parent_id, timestamp, author, uncles, number, chain_id, parent_gas_used, parents_hash) =
            block_metadata.into_inner();
        let mut function_name = &account_config::G_BLOCK_PROLOGUE_NAME;
        let mut args_vec = vec![
            MoveValue::Signer(txn_sender),
            MoveValue::vector_u8(parent_id.to_vec()),
            MoveValue::U64(timestamp),
            MoveValue::Address(author),
            MoveValue::vector_u8(Vec::new()),
            MoveValue::U64(uncles),
            MoveValue::U64(number),
            MoveValue::U8(chain_id.id()),
            MoveValue::U64(parent_gas_used),
        ];
        if let Some(version) = stdlib_version {
            if version >= StdlibVersion::Version(FLEXI_DAG_UPGRADE_VERSION_MARK) {
                args_vec.push(MoveValue::vector_u8(
                    bcs_ext::to_bytes(&parents_hash.unwrap_or_default())
                        .or(Err(VMStatus::error(VALUE_SERIALIZATION_ERROR, None)))?,
                ));
                function_name = &account_config::G_BLOCK_PROLOGUE_NAME;
            }
        }
        let args = serialize_values(&args_vec);
        let mut session = self.move_vm.new_session(storage, session_id);
        let traverse_storage = TraversalStorage::new();
        session
            .execute_function_bypass_visibility(
                &account_config::G_BLOCK_MODULE,
                function_name,
                vec![],
                args,
                &mut gas_meter,
                &mut TraversalContext::new(&traverse_storage),
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

    fn execute_user_transaction<S: StarcoinMoveResolver + StateView>(
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

        let session = self
            .move_vm
            .new_session(storage, SessionId::txn_meta(&txn_data));
        let mut gas_meter = StarcoinGasMeter::new(gas_params.clone(), txn_data.max_gas_amount());
        // check signature
        let signature_checked_txn = match txn.check_signature() {
            Ok(t) => Ok(t),
            Err(_) => Err(VMStatus::error(StatusCode::INVALID_SIGNATURE, None)),
        };

        match signature_checked_txn {
            Ok(txn) => {
                let result = match txn.payload() {
                    payload @ TransactionPayload::Script(_)
                    | payload @ TransactionPayload::EntryFunction(_) => self
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

    pub fn dry_run_transaction<S: StarcoinMoveResolver + StateView>(
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
            .new_session(storage, SessionId::txn_meta(&txn_data));
        let mut gas_meter = StarcoinGasMeter::new(gas_params.clone(), txn_data.max_gas_amount());
        gas_meter.set_metering(false);
        let result = match txn.raw_txn.payload() {
            payload @ TransactionPayload::Script(_)
            | payload @ TransactionPayload::EntryFunction(_) => {
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
        self.load_configs(&data_cache).map_err(|err| {
            error!(
                "StarcoinVM::execute_block_transactions | load_configs return error: ‘{:?}’ ",
                err
            );
            VMStatus::error(StatusCode::STORAGE_ERROR, None)
        })?;

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
                            .map_err(|_err| VMStatus::error(StatusCode::STORAGE_ERROR, None))?;

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
                return Err(VMStatus::error(StatusCode::VM_STARTUP_FAILURE, None));
            }
            let gas_params = self.get_gas_parameters()?;
            let mut gas_meter = StarcoinGasMeter::new(
                G_LATEST_GAS_PARAMS.clone(),
                gas_params.vm.txn.maximum_number_of_gas_units,
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

        let traversal_storage = TraversalStorage::new();
        let mut session = self.move_vm.new_session(&data_cache, SessionId::void());
        let result = session
            .execute_function_bypass_visibility(
                module,
                function_name,
                type_params,
                args,
                &mut gas_meter,
                &mut TraversalContext::new(&traversal_storage),
            )
            .map_err(|e| e.into_vm_status())?
            .return_values
            .into_iter()
            .map(|(a, _)| a)
            .collect();

        let change_set_config =
            ChangeSetConfigs::unlimited_at_gas_feature_version(LATEST_GAS_FEATURE_VERSION);
        let (change_set, module_write_set) = session
            .finish(&change_set_config)
            .map_err(|e| e.into_vm_status())?;
        let (write_set, _events) = change_set
            .try_combine_into_storage_change_set(module_write_set)
            .map_err(|e| {
                PartialVMError::from(e)
                    .finish(Location::Undefined)
                    .into_vm_status()
            })?
            .into_inner();
        if !write_set.is_empty() {
            warn!("Readonly function {} changes state", function_name);
            return Err(VMStatus::error(StatusCode::REJECTED_WRITE_SET, None));
        }
        Ok(result)
    }

    fn success_transaction_cleanup(
        &self,
        mut session: SessionExt,
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

    fn failed_transaction_cleanup<S: StarcoinMoveResolver + StateView>(
        &self,
        error_code: VMStatus,
        gas_meter: &mut StarcoinGasMeter,
        txn_data: &TransactionMetadata,
        storage: &S,
    ) -> (VMStatus, TransactionOutput) {
        gas_meter.set_metering(false);
        let mut session = self
            .move_vm
            .new_session(storage, SessionId::txn_meta(txn_data));

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
                (VMStatus::error(status, None), discard_error_output(status))
            }
            TransactionStatus::Retry => unreachable!(),
        }
    }

    pub fn get_gas_parameters(&self) -> Result<&StarcoinGasParameters, VMStatus> {
        self.gas_params.as_ref().ok_or_else(|| {
            debug!("VM Startup Failed. Gas Parameters Not Found");
            VMStatus::error(StatusCode::VM_STARTUP_FAILURE, None)
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
        // todo: retrieve chain_id properly
        let chain_id = if state_view.is_genesis() {
            Some(match txns.first().unwrap() {
                Transaction::UserTransaction(txn) => txn.chain_id().id(),
                Transaction::BlockMetadata(meta) => meta.chain_id().id(),
            })
        } else {
            None
        };
        let mut vm = Self::new_with_config(metrics, state_view, chain_id);
        vm.execute_block_transactions(state_view, txns, block_gas_limit)
    }

    pub fn load_module<R: StarcoinMoveResolver>(
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

pub(crate) fn charge_global_write_gas_usage(
    gas_meter: &mut StarcoinGasMeter,
    session: &SessionExt,
    sender: &AccountAddress,
) -> Result<(), VMStatus> {
    let write_set_gas = u64::from(gas_meter.cal_write_set_gas());
    let total_cost = InternalGasPerByte::from(write_set_gas)
        * NumBytes::new(session.num_mutated_accounts(sender));
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

pub(crate) fn get_transaction_output<A: AccessPathCache>(
    _ap_cache: &mut A,
    session: SessionExt,
    gas_left: Gas,
    max_gas_amount: Gas,
    status: KeptVMStatus,
) -> Result<TransactionOutput, VMStatus> {
    // original code use sub, now we use checked_sub
    let gas_used = max_gas_amount
        .checked_sub(gas_left)
        .expect("Balance should always be less than or equal to max gas amount");
    let change_set_config =
        ChangeSetConfigs::unlimited_at_gas_feature_version(LATEST_GAS_FEATURE_VERSION);
    let (change_set, module_write_set) = session.finish(&change_set_config)?;
    let (write_set, events) = change_set
        .try_combine_into_storage_change_set(module_write_set)
        .map_err(|e| {
            PartialVMError::from(e)
                .finish(Location::Undefined)
                .into_vm_status()
        })?
        .into_inner();
    Ok(TransactionOutput::new(
        write_set,
        events,
        u64::from(gas_used),
        TransactionStatus::Keep(status),
        TransactionAuxiliaryData::None,
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
        state_view: &(impl StateView + Sync),
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
            // debug!("TurboSTM executor concurrency_level {}", concurrency_level);1
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

impl StarcoinVM {
    pub(crate) fn should_restart_execution(output: &TransactionOutput) -> bool {
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

    pub(crate) fn execute_single_transaction<S: StarcoinMoveResolver + StateView>(
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
