// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0


/*
/// The function returns all native functions supported by Starcoin.
/// NOTICE:
/// - mostly re-use natives defined in move-stdlib.
/// - be careful with the native cost table index used in the implementation
pub fn starcoin_natives(gas_params: NativeGasParameters) -> NativeFunctionTable {
    let mut natives = vec![];

    macro_rules! add_natives_from_module {
        ($module_name: expr, $natives: expr) => {
            natives.extend(
                $natives.map(|(func_name, func)| ($module_name.to_string(), func_name, func)),
            );
        };
    }

    add_natives_from_module!(
        "Hash",
        move_stdlib::natives::hash::make_all(gas_params.move_stdlib.hash)
    );
    add_natives_from_module!(
        "Hash",
        starcoin_frameworks::natives::hash::make_all(gas_params.starcoin_natives.hash)
    );
    add_natives_from_module!(
        "BCS",
        move_stdlib::natives::bcs::make_all(gas_params.move_stdlib.bcs)
    );
    add_natives_from_module!(
        "FromBCS",
        starcoin_frameworks::natives::from_bcs::make_all(gas_params.starcoin_natives.from_bcs)
    );
    add_natives_from_module!(
        "Signature",
        starcoin_frameworks::natives::signature::make_all(gas_params.starcoin_natives.signature)
    );
    add_natives_from_module!(
        "Event",
        move_stdlib::natives::event::make_all(gas_params.nursery.clone().event)
    );
    add_natives_from_module!(
        "Account",
        starcoin_frameworks::natives::account::make_all(gas_params.starcoin_natives.account)
    );
    add_natives_from_module!(
        "Signer",
        move_stdlib::natives::signer::make_all(gas_params.move_stdlib.signer)
    );
    add_natives_from_module!(
        "Token",
        starcoin_frameworks::natives::token::make_all(gas_params.starcoin_natives.token)
    );
    add_natives_from_module!(
        "U256",
        starcoin_frameworks::natives::u256::make_all(gas_params.starcoin_natives.u256)
    );
    #[cfg(feature = "testing")]
    add_natives_from_module!(
        "unit_test",
        move_stdlib::natives::unit_test::make_all(gas_params.move_stdlib.unit_test)
    );
    add_natives_from_module!(
        "String",
        move_stdlib::natives::string::make_all(gas_params.move_stdlib.string)
    );
    add_natives_from_module!(
        "Debug",
        move_stdlib::natives::debug::make_all(gas_params.nursery.debug, CORE_CODE_ADDRESS)
    );
    add_natives_from_module!(
        "Secp256k1",
        starcoin_frameworks::natives::secp256k1::make_all(gas_params.starcoin_natives.secp256k1)
    );

    let natives = make_table_from_iter(CORE_CODE_ADDRESS, natives);
    natives
        .into_iter()
        .chain(table_natives(CORE_CODE_ADDRESS, gas_params.table))
        .collect()
}

fn table_natives(
    table_addr: AccountAddress,
    gas_params: move_table_extension::GasParameters,
) -> NativeFunctionTable {
    let natives: [(&str, &str, NativeFunction); 8] = [
        (
            "Table",
            "new_table_handle",
            move_table_extension::make_native_new_table_handle(gas_params.new_table_handle),
        ),
        (
            "Table",
            "add_box",
            move_table_extension::make_native_add_box(
                gas_params.common.clone(),
                gas_params.add_box,
            ),
        ),
        (
            "Table",
            "borrow_box",
            move_table_extension::make_native_borrow_box(
                gas_params.common.clone(),
                gas_params.borrow_box.clone(),
            ),
        ),
        (
            "Table",
            "borrow_box_mut",
            move_table_extension::make_native_borrow_box(
                gas_params.common.clone(),
                gas_params.borrow_box,
            ),
        ),
        (
            "Table",
            "remove_box",
            move_table_extension::make_native_remove_box(
                gas_params.common.clone(),
                gas_params.remove_box,
            ),
        ),
        (
            "Table",
            "contains_box",
            move_table_extension::make_native_contains_box(
                gas_params.common,
                gas_params.contains_box,
            ),
        ),
        (
            "Table",
            "destroy_empty_box",
            move_table_extension::make_native_destroy_empty_box(gas_params.destroy_empty_box),
        ),
        (
            "Table",
            "drop_unchecked_box",
            move_table_extension::make_native_drop_unchecked_box(gas_params.drop_unchecked_box),
        ),
    ];

    native_functions::make_table_from_iter(table_addr, natives)
} */




use starcoin_gas_schedule::{MiscGasParameters, NativeGasParameters, LATEST_GAS_FEATURE_VERSION};
use starcoin_native_interface::SafeNativeBuilder;
#[cfg(feature = "testing")]
use move_table_extension::{TableHandle, TableResolver};
use starcoin_vm_types::{
    account_config::CORE_CODE_ADDRESS,
    on_chain_config::{Features, TimedFeatures, TimedFeaturesBuilder},
};
#[cfg(feature = "testing")]
use starcoin_vm_types::{
    state_store::{state_key::StateKey, state_value::StateValue},
    write_set::WriteOp,
};
#[cfg(feature = "testing")]
use bytes::Bytes;
#[cfg(feature = "testing")]
use move_binary_format::errors::PartialVMError;
#[cfg(feature = "testing")]
use move_core_types::{language_storage::StructTag, value::MoveTypeLayout};
use move_vm_runtime::native_functions::NativeFunctionTable;
#[cfg(feature = "testing")]
use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
};
use move_table_extension::NativeTableContext;
#[cfg(feature = "testing")]
use {
    starcoin_frameworks::natives::{
        aggregator_natives::NativeAggregatorContext, code::NativeCodeContext,
        cryptography::ristretto255_point::NativeRistrettoPointContext,
        transaction_context::NativeTransactionContext,
    },
    move_vm_runtime::native_extensions::NativeContextExtensions,
    once_cell::sync::Lazy,
};

#[cfg(feature = "testing")]
struct StarcoinBlankStorage;

#[cfg(feature = "testing")]
impl StarcoinBlankStorage {
    pub fn new() -> Self {
        Self {}
    }
}

/*
#[cfg(feature = "testing")]
impl TAggregatorV1View for StarcoinBlankStorage {
    type Identifier = StateKey;

    fn get_aggregator_v1_state_value(
        &self,
        _id: &Self::Identifier,
    ) -> anyhow::Result<Option<StateValue>> {
        Ok(None)
    }
} */

/*
#[cfg(feature = "testing")]
impl TDelayedFieldView for AptosBlankStorage {
    type Identifier = DelayedFieldID;
    type ResourceGroupTag = StructTag;
    type ResourceKey = StateKey;
    type ResourceValue = WriteOp;

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

    fn generate_delayed_field_id(&self) -> Self::Identifier {
        unreachable!()
    }

    fn validate_and_convert_delayed_field_id(
        &self,
        _id: u64,
    ) -> Result<Self::Identifier, PanicError> {
        unreachable!()
    }

    fn get_reads_needing_exchange(
        &self,
        _delayed_write_set_keys: &HashSet<Self::Identifier>,
        _skip: &HashSet<Self::ResourceKey>,
    ) -> Result<BTreeMap<Self::ResourceKey, (Self::ResourceValue, Arc<MoveTypeLayout>)>, PanicError>
    {
        unreachable!()
    }

    fn get_group_reads_needing_exchange(
        &self,
        _delayed_write_set_keys: &HashSet<Self::Identifier>,
        _skip: &HashSet<Self::ResourceKey>,
    ) -> Result<BTreeMap<Self::ResourceKey, (Self::ResourceValue, u64)>, PanicError> {
        unimplemented!()
    }
} */

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
#[allow(clippy::redundant_closure)]
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
        .filter(|(_, name, _, _)| name.as_str() != "vector")
        .chain(starcoin_frameworks::natives::all_natives(
            CORE_CODE_ADDRESS,
            builder,
        ))
        .chain(aptos_table_natives::table_natives(
            CORE_CODE_ADDRESS,
            builder,
        ))
        .collect()
}

pub fn assert_no_test_natives(err_msg: &str) {
    assert!(
        aptos_natives(
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
fn unit_test_extensions_hook(exts: &mut NativeContextExtensions) {
    use aptos_table_natives::NativeTableContext;

    exts.add(NativeTableContext::new([0u8; 32], &*DUMMY_RESOLVER));
    exts.add(NativeCodeContext::default());
    exts.add(NativeTransactionContext::new(
        vec![1],
        vec![1],
        ChainId::test().id(),
    )); // We use the testing environment chain ID here
    exts.add(NativeAggregatorContext::new(
        [0; 32],
        &*DUMMY_RESOLVER,
        &*DUMMY_RESOLVER,
    ));
    exts.add(NativeRistrettoPointContext::new());
    exts.add(AlgebraContext::new());
    exts.add(NativeEventContext::default());
}
