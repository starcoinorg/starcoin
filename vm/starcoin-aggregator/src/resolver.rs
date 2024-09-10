// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    aggregator_v1_extension::{addition_v1_error, subtraction_v1_error},
    bounded_math::SignedU128,
    delta_change_set::{serialize, DeltaOp},
    types::{
        code_invariant_error, DelayedFieldID, DelayedFieldValue, DelayedFieldsSpeculativeError,
        DeltaApplicationFailureReason, PanicOr,
    },
};
use starcoin_vm_types::{
    aggregator::PanicError,
    state_store::{
        state_key::StateKey,
        state_value::{StateValue, StateValueMetadata},
    },
    write_set::WriteOp, state_view::StateView,
};
use move_binary_format::errors::Location;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    identifier::IdentStr,
    language_storage::{ModuleId, StructTag, CORE_CODE_ADDRESS},
    value::MoveTypeLayout,
    vm_status::{StatusCode, VMStatus},
};
use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
};

/// We differentiate between deprecated way to interact with aggregators (TAggregatorV1View),
/// and new, more general, TDelayedFieldView.

/// Allows to query AggregatorV1 values from the state storage.
pub trait TAggregatorV1View {
    type Identifier;

    /// Aggregator V1 is implemented as a state item, and therefore the API has
    /// the same pattern as for modules or resources:
    ///   -  Ok(None)         if aggregator value is not in storage,
    ///   -  Ok(Some(...))    if aggregator value exists in storage,
    ///   -  Err(...)         otherwise (e.g. storage error or failed delta
    ///                       application).
    fn get_aggregator_v1_state_value(
        &self,
        id: &Self::Identifier,
    ) -> anyhow::Result<Option<StateValue>>;

    fn get_aggregator_v1_value(&self, id: &Self::Identifier) -> anyhow::Result<Option<u128>> {
        let maybe_state_value = self.get_aggregator_v1_state_value(id)?;
        match maybe_state_value {
            Some(state_value) => Ok(Some(bcs::from_bytes(state_value.bytes())?)),
            None => Ok(None),
        }
    }

    /// Because aggregator V1 is a state item, it also can have metadata (for
    /// example used to calculate storage refunds).
    fn get_aggregator_v1_state_value_metadata(
        &self,
        id: &Self::Identifier,
    ) -> anyhow::Result<Option<StateValueMetadata>> {
        // When getting state value metadata for aggregator V1, we need to do a
        // precise read.
        let maybe_state_value = self.get_aggregator_v1_state_value(id)?;
        Ok(maybe_state_value.map(StateValue::into_metadata))
    }

    /// Consumes a single delta of aggregator V1, and tries to materialize it
    /// with a given identifier (state key). If materialization succeeds, a
    /// write op is produced.
    fn try_convert_aggregator_v1_delta_into_write_op(
        &self,
        id: &Self::Identifier,
        delta_op: &DeltaOp,
    ) -> anyhow::Result<WriteOp, VMStatus> {
        let base = self
            .get_aggregator_v1_value(id)
            .map_err(|e| {
                VMStatus::error(
                    StatusCode::SPECULATIVE_EXECUTION_ABORT_ERROR,
                    Some(e.to_string()),
                )
            })?
            .ok_or_else(|| {
                VMStatus::error(
                    StatusCode::SPECULATIVE_EXECUTION_ABORT_ERROR,
                    Some("Cannot convert delta for deleted aggregator".to_string()),
                )
            })?;

        // We need to set abort location for Aggregator V1 to ensure correct VMStatus can be constructed.
        const AGGREGATOR_V1_ADDRESS: AccountAddress = CORE_CODE_ADDRESS;
        const AGGREGATOR_V1_MODULE_NAME: &IdentStr = ident_str!("aggregator");

        delta_op
            .apply_to(base)
            .map_err(|e| match &e {
                PanicOr::Or(DelayedFieldsSpeculativeError::DeltaApplication {
                                reason: DeltaApplicationFailureReason::Overflow,
                                ..
                            }) => addition_v1_error(e),
                PanicOr::Or(DelayedFieldsSpeculativeError::DeltaApplication {
                                reason: DeltaApplicationFailureReason::Underflow,
                                ..
                            }) => subtraction_v1_error(e),
                _ => code_invariant_error(format!("Unexpected delta application error: {:?}", e))
                    .into(),
            })
            .map_err(|partial_error| {
                partial_error
                    .finish(Location::Module(ModuleId::new(
                        AGGREGATOR_V1_ADDRESS,
                        AGGREGATOR_V1_MODULE_NAME.into(),
                    )))
                    .into_vm_status()
            })
            .map(|result| WriteOp::legacy_modification(serialize(&result).into()))
    }
}

pub trait AggregatorV1Resolver: TAggregatorV1View<Identifier = StateKey> {}

impl<T> AggregatorV1Resolver for T where T: TAggregatorV1View<Identifier = StateKey> {}

impl<S> TAggregatorV1View for S
    where
        S: StateView,
{
    type Identifier = StateKey;

    fn get_aggregator_v1_state_value(
        &self,
        state_key: &Self::Identifier,
    ) -> anyhow::Result<Option<StateValue>> {
        self.get_state_value(state_key)
    }
}

/// Allows to query DelayedFields (AggregatorV2/AggregatorSnapshots) values
/// from the state storage.
pub trait TDelayedFieldView {
    type Identifier;
    type ResourceKey;
    type ResourceGroupTag;
    type ResourceValue;

    fn is_delayed_field_optimization_capable(&self) -> bool;

    /// Fetch a value of a DelayedField.
    fn get_delayed_field_value(
        &self,
        id: &Self::Identifier,
    ) -> Result<DelayedFieldValue, PanicOr<DelayedFieldsSpeculativeError>>;

    /// Fetch an outcome of whether additional delta can be applied.
    /// `base_delta` argument represents a cumulative value that we previously checked,
    /// and `delta` argument represents a new increment.
    /// (This allows method to be stateless, and not require it to store previous calls,
    /// i.e. for sequential execution)
    ///
    /// For example, calls would go like this:
    /// try_add_delta_outcome(base_delta = 0, delta = 5) -> true
    /// try_add_delta_outcome(base_delta = 5, delta = 3) -> true
    /// try_add_delta_outcome(base_delta = 8, delta = 2) -> false
    /// try_add_delta_outcome(base_delta = 8, delta = 3) -> false
    /// try_add_delta_outcome(base_delta = 8, delta = -3) -> true
    /// try_add_delta_outcome(base_delta = 5, delta = 2) -> true
    /// ...
    fn delayed_field_try_add_delta_outcome(
        &self,
        id: &Self::Identifier,
        base_delta: &SignedU128,
        delta: &SignedU128,
        max_value: u128,
    ) -> Result<bool, PanicOr<DelayedFieldsSpeculativeError>>;

    /// Returns a unique per-block identifier that can be used when creating a
    /// new aggregator V2.
    fn generate_delayed_field_id(&self) -> Self::Identifier;

    /// Validate that given value (from aggregator structure) is a valid delayed field identifier,
    /// and convert it to Self::Identifier if so.
    fn validate_and_convert_delayed_field_id(
        &self,
        id: u64,
    ) -> Result<Self::Identifier, PanicError>;

    /// Returns the list of resources that satisfy all the following conditions:
    /// 1. The resource is read during the transaction execution.
    /// 2. The resource is not present in write set of the VM Change Set.
    /// 3. The resource has a delayed field in it that is part of delayed field change set.
    /// We get these resources and include them in the write set of the transaction output.
    fn get_reads_needing_exchange(
        &self,
        delayed_write_set_keys: &HashSet<Self::Identifier>,
        skip: &HashSet<Self::ResourceKey>,
    ) -> Result<BTreeMap<Self::ResourceKey, (Self::ResourceValue, Arc<MoveTypeLayout>)>, PanicError>;

    /// Returns the list of resource groups that satisfy all the following conditions:
    /// 1. At least one of the resource in the group is read during the transaction execution.
    /// 2. The resource group is not present in the write set of the VM Change Set.
    /// 3. At least one of the resources in the group has a delayed field in it that is part.
    /// of delayed field change set.
    /// We get these resource groups and include them in the write set of the transaction output.
    /// For each such resource group, this function outputs (resource key, (metadata op, resource group size))
    fn get_group_reads_needing_exchange(
        &self,
        delayed_write_set_keys: &HashSet<Self::Identifier>,
        skip: &HashSet<Self::ResourceKey>,
    ) -> Result<BTreeMap<Self::ResourceKey, (Self::ResourceValue, u64)>, PanicError>;
}

pub trait DelayedFieldResolver:
TDelayedFieldView<
    Identifier = DelayedFieldID,
    ResourceKey = StateKey,
    ResourceGroupTag = StructTag,
    ResourceValue = WriteOp,
>
{
}

impl<T> DelayedFieldResolver for T where
    T: TDelayedFieldView<
        Identifier = DelayedFieldID,
        ResourceKey = StateKey,
        ResourceGroupTag = StructTag,
        ResourceValue = WriteOp,
    >
{
}

impl<S> TDelayedFieldView for S
    where
        S: StateView,
{
    type Identifier = DelayedFieldID;
    type ResourceGroupTag = StructTag;
    type ResourceKey = StateKey;
    type ResourceValue = WriteOp;

    fn is_delayed_field_optimization_capable(&self) -> bool {
        // For resolvers that are not capable, it cannot be enabled
        false
    }

    fn get_delayed_field_value(
        &self,
        _id: &Self::Identifier,
    ) -> Result<DelayedFieldValue, PanicOr<DelayedFieldsSpeculativeError>> {
        unimplemented!("get_delayed_field_value not implemented")
    }

    fn delayed_field_try_add_delta_outcome(
        &self,
        _id: &Self::Identifier,
        _base_delta: &SignedU128,
        _delta: &SignedU128,
        _max_value: u128,
    ) -> Result<bool, PanicOr<DelayedFieldsSpeculativeError>> {
        unimplemented!("delayed_field_try_add_delta_outcome not implemented")
    }

    /// Returns a unique per-block identifier that can be used when creating a
    /// new aggregator V2.
    fn generate_delayed_field_id(&self) -> Self::Identifier {
        unimplemented!("generate_delayed_field_id not implemented")
    }

    fn validate_and_convert_delayed_field_id(
        &self,
        _id: u64,
    ) -> Result<Self::Identifier, PanicError> {
        unimplemented!("get_and_validate_delayed_field_id not implemented")
    }

    fn get_reads_needing_exchange(
        &self,
        _delayed_write_set_keys: &HashSet<Self::Identifier>,
        _skip: &HashSet<Self::ResourceKey>,
    ) -> Result<BTreeMap<Self::ResourceKey, (Self::ResourceValue, Arc<MoveTypeLayout>)>, PanicError>
    {
        unimplemented!("get_reads_needing_exchange not implemented")
    }

    fn get_group_reads_needing_exchange(
        &self,
        _delayed_write_set_keys: &HashSet<Self::Identifier>,
        _skip: &HashSet<Self::ResourceKey>,
    ) -> Result<BTreeMap<Self::ResourceKey, (Self::ResourceValue, u64)>, PanicError> {
        unimplemented!("get_group_reads_needing_exchange not implemented")
    }
}
