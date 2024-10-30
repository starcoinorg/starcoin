// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_vm_ext::session::SessionId;
use crate::move_vm_ext::warm_vm_cache::WarmVmCache;
use crate::move_vm_ext::{SessionExt, StarcoinMoveResolver};
use crate::natives;
use crate::natives::starcoin_natives_with_builder;
use move_binary_format::deserializer::DeserializerConfig;
use move_binary_format::errors::VMResult;
use move_vm_runtime::config::VMConfig;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_runtime::native_extensions::NativeContextExtensions;
use move_vm_types::loaded_data::runtime_types::TypeBuilder;
use starcoin_crypto::HashValue;
use starcoin_framework::natives::aggregator_natives::NativeAggregatorContext;
use starcoin_gas_algebra::DynamicExpression;
use starcoin_gas_schedule::{MiscGasParameters, NativeGasParameters, LATEST_GAS_FEATURE_VERSION};
use starcoin_native_interface::SafeNativeBuilder;
use starcoin_table_natives::NativeTableContext;
use starcoin_vm_runtime_types::storage::change_set_configs::ChangeSetConfigs;
use starcoin_vm_types::{
    errors::PartialVMResult,
    genesis_config::ChainId,
    on_chain_config::{Features, TimedFeatureFlag, TimedFeatures, TimedFeaturesBuilder},
};
use std::ops::Deref;
use std::sync::Arc;
use starcoin_framework::natives::event::NativeEventContext;
use starcoin_framework::natives::object::NativeObjectContext;

/// MoveVM wrapper which is used to run genesis initializations. Designed as a
/// stand-alone struct to ensure all genesis configurations are in one place,
/// and are modified accordingly. The VM is initialized with default parameters,
/// and should only be used to run genesis sessions.
pub struct GenesisMoveVM {
    vm: MoveVM,
    chain_id: ChainId,
    features: Features,
}

impl GenesisMoveVM {
    pub fn new(chain_id: ChainId) -> Self {
        let features = Features::default();
        let timed_features = TimedFeaturesBuilder::enable_all().build();

        let vm_config = starcoin_prod_vm_config(&features, &timed_features, TypeBuilder::Legacy);

        // All genesis sessions run with unmetered gas meter, and here we set
        // the gas parameters for natives as zeros (because they do not matter).
        let mut native_builder = SafeNativeBuilder::new(
            LATEST_GAS_FEATURE_VERSION,
            NativeGasParameters::zeros(),
            MiscGasParameters::zeros(),
            timed_features.clone(),
            features.clone(),
        );

        let vm = MoveVM::new_with_config(
            starcoin_natives_with_builder(&mut native_builder),
            vm_config.clone(),
        )
        .expect("creating MoveVM shouldn't failed.");

        Self {
            vm,
            chain_id,
            features,
        }
    }

    pub fn genesis_change_set_configs(&self) -> ChangeSetConfigs {
        // Because genesis sessions are not metered, there are no change set
        // (storage) costs as well.
        ChangeSetConfigs::unlimited_at_gas_feature_version(LATEST_GAS_FEATURE_VERSION)
    }

    pub fn new_genesis_session<'r, R: StarcoinMoveResolver>(
        &self,
        resolver: &'r R,
        genesis_id: HashValue,
    ) -> SessionExt<'r, '_> {
        let session_id = SessionId::genesis(genesis_id);
        let txn_hash: [u8; 32] = session_id
            .as_uuid()
            .to_vec()
            .try_into()
            .expect("HashValue should convert to [u8; 32]");
        let mut extensions = NativeContextExtensions::default();
        extensions.add(NativeTableContext::new(txn_hash, resolver));

        self.vm.flush_loader_cache_if_invalidated();

        SessionExt::new(
            self.vm.new_session_with_extensions(resolver, extensions),
            resolver,
            Arc::new(self.features.clone()),
        )
    }
}

pub struct MoveVmExt {
    inner: MoveVM,
    _chain_id: u8,
    features: Arc<Features>,
}

impl MoveVmExt {
    fn new_impl<F>(
        native_gas_parameters: NativeGasParameters,
        misc_gas_parameters: MiscGasParameters,
        gas_feature_version: u64,
        _chain_id: u8,
        features: Features,
        timed_features: TimedFeatures,
        gas_hook: Option<F>,
        resolver: &impl StarcoinMoveResolver,
    ) -> VMResult<Self>
    where
        F: Fn(DynamicExpression) + Send + Sync + 'static,
    {
        // Note: binary format v6 adds a few new integer types and their corresponding instructions.
        //       Therefore it depends on a new version of the gas schedule and cannot be allowed if
        //       the gas schedule hasn't been updated yet.
        let max_binary_format_version =
            get_max_binary_format_version(&features, Some(gas_feature_version));

        let max_identifier_size = get_max_identifier_size(&features);

        let _enable_invariant_violation_check_in_swap_loc =
            !timed_features.is_enabled(TimedFeatureFlag::DisableInvariantViolationCheckInSwapLoc);
        let _type_size_limit = true;

        let verifier_config = verifier_config(&features, &timed_features);

        let mut type_max_cost = 0;
        let mut type_base_cost = 0;
        let mut type_byte_cost = 0;
        if timed_features.is_enabled(TimedFeatureFlag::LimitTypeTagSize) {
            // 5000 limits type tag total size < 5000 bytes and < 50 nodes
            type_max_cost = 5000;
            type_base_cost = 100;
            type_byte_cost = 1;
        }

        let mut builder = SafeNativeBuilder::new(
            gas_feature_version,
            native_gas_parameters.clone(),
            misc_gas_parameters.clone(),
            timed_features.clone(),
            features.clone(),
        );

        if let Some(hook) = gas_hook {
            builder.set_gas_hook(hook);
        }
        Ok(Self {
            inner: WarmVmCache::get_warm_vm(
                builder,
                VMConfig {
                    verifier_config,
                    deserializer_config: DeserializerConfig::new(max_binary_format_version, max_identifier_size),
                    paranoid_type_checks: /*crate::StarcoinVM::get_paranoid_checks() */ false,
                    max_value_nest_depth: Some(128),
                    type_max_cost,
                    type_base_cost,
                    type_byte_cost,
                    // todo: support aggregator_v2_type_tagging, set false as default now.
                    aggregator_v2_type_tagging: false,
                    check_invariant_in_swap_loc: false,
                    ty_builder: TypeBuilder::Legacy,
                },
                resolver,
            )?,
            _chain_id,
            features: Arc::new(features),
        })
    }
    pub fn new(
        native_gas_params: NativeGasParameters,
        misc_gas_parameters: MiscGasParameters,
        gas_feature_version: u64,
        chain_id: u8,
        features: Features,
        timed_features: TimedFeatures,
        resolver: &impl StarcoinMoveResolver,
    ) -> VMResult<Self> {
        Self::new_impl::<fn(DynamicExpression)>(
            native_gas_params,
            misc_gas_parameters,
            gas_feature_version,
            chain_id,
            features,
            timed_features,
            None,
            resolver,
        )
    }

    pub fn new_with_gas_hook<F>(
        native_gas_params: NativeGasParameters,
        misc_gas_params: MiscGasParameters,
        gas_feature_version: u64,
        chain_id: u8,
        features: Features,
        timed_features: TimedFeatures,
        gas_hook: Option<F>,
        resolver: &impl StarcoinMoveResolver,
    ) -> VMResult<Self>
    where
        F: Fn(DynamicExpression) + Send + Sync + 'static,
    {
        Self::new_impl(
            native_gas_params,
            misc_gas_params,
            gas_feature_version,
            chain_id,
            features,
            timed_features,
            gas_hook,
            resolver,
        )
    }

    pub fn new_session<'r, S: StarcoinMoveResolver>(
        &self,
        resolver: &'r S,
        session_id: SessionId,
    ) -> SessionExt<'r, '_> {
        let mut extensions = NativeContextExtensions::default();
        let txn_hash: [u8; 32] = session_id
            .as_uuid()
            .to_vec()
            .try_into()
            .expect("HashValue should convert to [u8; 32]");

        extensions.add(NativeTableContext::new(txn_hash, resolver));
        extensions.add(NativeAggregatorContext::new(txn_hash, resolver, resolver));
        extensions.add(NativeEventContext::default());
        extensions.add(NativeObjectContext::default());

        // The VM code loader has bugs around module upgrade. After a module upgrade, the internal
        // cache needs to be flushed to work around those bugs.
        self.inner.flush_loader_cache_if_invalidated();
        SessionExt::new(
            self.inner.new_session_with_extensions(resolver, extensions),
            resolver,
            self.features.clone(),
        )
    }

    pub fn update_native_functions(
        &mut self,
        native_gas_params: NativeGasParameters,
        misc_gas_parameters: MiscGasParameters,
    ) -> PartialVMResult<()> {
        //todo: select featrure version properly
        let native_functions = natives::starcoin_natives(
            1,
            native_gas_params,
            misc_gas_parameters,
            TimedFeaturesBuilder::enable_all().build(),
            Default::default(),
        );
        self.inner.update_native_functions(native_functions)
    }
}

impl Deref for MoveVmExt {
    type Target = MoveVM;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
