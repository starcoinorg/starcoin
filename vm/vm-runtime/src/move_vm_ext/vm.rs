// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_vm_ext::session::SessionId;
use crate::move_vm_ext::warm_vm_cache::WarmVmCache;
use crate::move_vm_ext::{SessionExt, StarcoinMoveResolver};
use crate::natives;
use move_binary_format::errors::VMResult;
use move_vm_runtime::move_vm::MoveVM;
use move_vm_runtime::native_extensions::NativeContextExtensions;
use move_vm_types::loaded_data::runtime_types::TypeBuilder;
use starcoin_framework::natives::{
    aggregator_natives::NativeAggregatorContext, event::NativeEventContext,
    object::NativeObjectContext,
};
use starcoin_gas_algebra::DynamicExpression;
use starcoin_gas_schedule::{MiscGasParameters, NativeGasParameters};
use starcoin_native_interface::SafeNativeBuilder;
use starcoin_table_natives::NativeTableContext;
use starcoin_types::vm::config::starcoin_prod_vm_config;
use starcoin_vm_types::{
    errors::PartialVMResult,
    on_chain_config::{Features, TimedFeatureFlag, TimedFeatures, TimedFeaturesBuilder},
};

use std::ops::Deref;
use std::sync::Arc;
use starcoin_framework::natives::transaction_context::NativeTransactionContext;

pub struct MoveVmExt {
    inner: MoveVM,
    chain_id: u8,
    features: Arc<Features>,
}

impl MoveVmExt {
    fn new_impl<F>(
        native_gas_parameters: NativeGasParameters,
        misc_gas_parameters: MiscGasParameters,
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
        let vm_config = starcoin_prod_vm_config(&features, &timed_features, TypeBuilder::Legacy);

        let _enable_invariant_violation_check_in_swap_loc =
            !timed_features.is_enabled(TimedFeatureFlag::DisableInvariantViolationCheckInSwapLoc);
        let _type_size_limit = true;

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
            inner: WarmVmCache::get_warm_vm(builder, vm_config, resolver)?,
            chain_id,
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
        extensions.add(NativeTransactionContext::new(
            txn_hash.to_vec(),
            //session_id.into_script_hash(),
            vec![1], // TODO(BobOng): [compiler-v2] to confirm the script hash
            self.chain_id,
            // TODO(BobOng): [compiler-v2] to confirm the user transaction context
            None,
        ));

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
        gas_feature_version: u64,
    ) -> PartialVMResult<()> {
        let native_functions = natives::starcoin_natives(
            gas_feature_version,
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
