// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_vm_runtime::native_functions::NativeFunctionTable;
use starcoin_framework::natives::object::NativeObjectContext;
use starcoin_framework::natives::randomness::RandomnessContext;
use starcoin_gas_schedule::{MiscGasParameters, NativeGasParameters, LATEST_GAS_FEATURE_VERSION};
use starcoin_native_interface::SafeNativeBuilder;
use starcoin_vm_types::{
    account_config::CORE_CODE_ADDRESS,
    on_chain_config::{Features, TimedFeatures, TimedFeaturesBuilder},
};
#[cfg(feature = "testing")]
use {
    move_binary_format::errors::PartialVMError,
    move_binary_format::errors::PartialVMResult,
    move_core_types::language_storage::StructTag,
    move_core_types::value::MoveTypeLayout,
    move_table_extension::{TableHandle, TableResolver},
    move_vm_types::delayed_values::delayed_field_id::DelayedFieldID,
    starcoin_aggregator::bounded_math::SignedU128,
    starcoin_aggregator::resolver::{TAggregatorV1View, TDelayedFieldView},
    starcoin_aggregator::types::{DelayedFieldValue, DelayedFieldsSpeculativeError, PanicOr},
    starcoin_vm_types::state_store::state_value::StateValueMetadata,
    starcoin_vm_types::state_store::{state_key::StateKey, state_value::StateValue},
    std::collections::{BTreeMap, HashSet},
    std::sync::Arc,
    {bytes::Bytes, starcoin_types::delayed_fields::PanicError},
    {move_vm_runtime::native_extensions::NativeContextExtensions, once_cell::sync::Lazy},
};

#[cfg(feature = "testing")]
#[allow(dead_code)]
struct StarcoinBlankStorage;

#[cfg(feature = "testing")]
impl StarcoinBlankStorage {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(feature = "testing")]
impl TAggregatorV1View for StarcoinBlankStorage {
    type Identifier = StateKey;

    fn get_aggregator_v1_state_value(
        &self,
        _id: &Self::Identifier,
    ) -> PartialVMResult<Option<StateValue>> {
        Ok(None)
    }
}

#[cfg(feature = "testing")]
impl TDelayedFieldView for StarcoinBlankStorage {
    type Identifier = DelayedFieldID;
    type ResourceGroupTag = StructTag;
    type ResourceKey = StateKey;

    fn is_delayed_field_optimization_capable(&self) -> bool {
        false
    }

    fn get_delayed_field_value(
        &self,
        _id: &Self::Identifier,
    ) -> Result<DelayedFieldValue, PanicOr<DelayedFieldsSpeculativeError>> {
        unreachable!()
    }

    fn delayed_field_try_add_delta_outcome(
        &self,
        _id: &Self::Identifier,
        _base_delta: &SignedU128,
        _delta: &SignedU128,
        _max_value: u128,
    ) -> Result<bool, PanicOr<DelayedFieldsSpeculativeError>> {
        unreachable!()
    }

    fn generate_delayed_field_id(&self, _width: u32) -> Self::Identifier {
        unreachable!()
    }

    fn validate_delayed_field_id(&self, _id: &Self::Identifier) -> Result<(), PanicError> {
        unreachable!()
    }

    fn get_reads_needing_exchange(
        &self,
        _delayed_write_set_keys: &HashSet<Self::Identifier>,
        _skip: &HashSet<Self::ResourceKey>,
    ) -> Result<
        BTreeMap<Self::ResourceKey, (StateValueMetadata, u64, Arc<MoveTypeLayout>)>,
        PanicError,
    > {
        unreachable!()
    }

    fn get_group_reads_needing_exchange(
        &self,
        _delayed_write_set_keys: &HashSet<Self::Identifier>,
        _skip: &HashSet<Self::ResourceKey>,
    ) -> PartialVMResult<BTreeMap<Self::ResourceKey, (StateValueMetadata, u64)>> {
        unimplemented!()
    }
}

#[cfg(feature = "testing")]
impl TableResolver for StarcoinBlankStorage {
    fn resolve_table_entry_bytes_with_layout(
        &self,
        _handle: &TableHandle,
        _key: &[u8],
        _layout: Option<&MoveTypeLayout>,
    ) -> Result<Option<Bytes>, PartialVMError> {
        Ok(None)
    }
}

#[cfg(feature = "testing")]
#[allow(clippy::redundant_closure, dead_code)]
static DUMMY_RESOLVER: Lazy<StarcoinBlankStorage> = Lazy::new(|| StarcoinBlankStorage::new());

pub fn starcoin_natives(
    gas_feature_version: u64,
    native_gas_params: NativeGasParameters,
    misc_gas_params: MiscGasParameters,
    timed_features: TimedFeatures,
    features: Features,
) -> NativeFunctionTable {
    let mut builder = SafeNativeBuilder::new(
        gas_feature_version,
        native_gas_params,
        misc_gas_params,
        timed_features,
        features,
    );

    starcoin_natives_with_builder(&mut builder)
}

pub fn starcoin_natives_with_builder(builder: &mut SafeNativeBuilder) -> NativeFunctionTable {
    #[allow(unreachable_code)]
    starcoin_move_stdlib::natives::all_natives(CORE_CODE_ADDRESS, builder)
        .into_iter()
        .chain(starcoin_table_natives::table_natives(
            CORE_CODE_ADDRESS,
            builder,
        ))
        .chain(starcoin_framework::natives::all_natives(
            CORE_CODE_ADDRESS,
            builder,
        ))
        .collect()
}

pub fn assert_no_test_natives(err_msg: &str) {
    assert!(
        starcoin_natives(
            LATEST_GAS_FEATURE_VERSION,
            NativeGasParameters::zeros(),
            MiscGasParameters::zeros(),
            TimedFeaturesBuilder::enable_all().build(),
            Features::default()
        )
        .into_iter()
        .all(|(_, module_name, func_name, _)| {
            !(module_name.as_str() == "unit_test"
                && func_name.as_str() == "create_signers_for_testing"
                || module_name.as_str() == "ed25519"
                    && func_name.as_str() == "generate_keys_internal"
                || module_name.as_str() == "ed25519" && func_name.as_str() == "sign_internal"
                || module_name.as_str() == "multi_ed25519"
                    && func_name.as_str() == "generate_keys_internal"
                || module_name.as_str() == "multi_ed25519" && func_name.as_str() == "sign_internal"
                || module_name.as_str() == "bls12381"
                    && func_name.as_str() == "generate_keys_internal"
                || module_name.as_str() == "bls12381" && func_name.as_str() == "sign_internal"
                || module_name.as_str() == "bls12381"
                    && func_name.as_str() == "generate_proof_of_possession_internal"
                || module_name.as_str() == "event"
                    && func_name.as_str() == "emitted_events_internal")
        }),
        "{}",
        err_msg
    )
}

#[cfg(feature = "testing")]
pub fn configure_for_unit_test() {
    move_unit_test::extensions::set_extension_hook(Box::new(unit_test_extensions_hook))
}

#[cfg(feature = "testing")]
#[allow(dead_code)]
fn unit_test_extensions_hook(exts: &mut NativeContextExtensions) {
    use starcoin_framework::natives::aggregator_natives::NativeAggregatorContext;
    use starcoin_framework::natives::code::NativeCodeContext;
    use starcoin_framework::natives::cryptography::algebra::AlgebraContext;
    use starcoin_framework::natives::cryptography::ristretto255_point::NativeRistrettoPointContext;
    use starcoin_framework::natives::event::NativeEventContext;
    use starcoin_framework::natives::transaction_context::NativeTransactionContext;
    use starcoin_table_natives::NativeTableContext;
    use starcoin_vm_types::genesis_config::ChainId;

    exts.add(NativeTableContext::new([0u8; 32], &*DUMMY_RESOLVER));
    exts.add(NativeCodeContext::default());
    exts.add(NativeTransactionContext::new(
        vec![1],
        vec![1],
        ChainId::test().id(),
        None,
    )); // We use the testing environment chain ID here
    exts.add(NativeAggregatorContext::new(
        [0; 32],
        &*DUMMY_RESOLVER,
        &*DUMMY_RESOLVER,
    ));
    exts.add(NativeRistrettoPointContext::new());
    exts.add(AlgebraContext::new());
    exts.add(NativeEventContext::default());
    exts.add(NativeObjectContext::default());
    exts.add(RandomnessContext::new());
}
