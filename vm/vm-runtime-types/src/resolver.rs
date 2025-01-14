// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytes::Bytes;
use move_core_types::value::MoveTypeLayout;
use move_vm_types::delayed_values::delayed_field_id::DelayedFieldID;
use starcoin_aggregator::resolver::{TAggregatorV1View, TDelayedFieldView};
use starcoin_vm_types::errors::PartialVMResult;
use starcoin_vm_types::write_set::WriteOp;
use starcoin_vm_types::{
    language_storage::StructTag,
    state_store::{
        state_key::StateKey,
        state_storage_usage::StateStorageUsage,
        state_value::{StateValue, StateValueMetadata},
        StateView, StateViewId,
    },
};
use std::collections::{BTreeMap, HashMap};

/// Allows to query resources from the state.
pub trait TResourceView {
    type Key;
    type Layout;

    /// Returns
    ///   -  Ok(None)         if the resource is not in storage,
    ///   -  Ok(Some(...))    if the resource exists in storage,
    ///   -  Err(...)         otherwise (e.g. storage error).
    fn get_resource_state_value(
        &self,
        state_key: &Self::Key,
        maybe_layout: Option<&Self::Layout>,
    ) -> PartialVMResult<Option<StateValue>>;

    fn get_resource_bytes(
        &self,
        state_key: &Self::Key,
        maybe_layout: Option<&Self::Layout>,
    ) -> PartialVMResult<Option<Bytes>> {
        let maybe_state_value = self.get_resource_state_value(state_key, maybe_layout)?;
        Ok(maybe_state_value.map(|state_value| state_value.bytes().clone()))
    }

    fn get_resource_state_value_metadata(
        &self,
        state_key: &Self::Key,
    ) -> PartialVMResult<Option<StateValueMetadata>> {
        // For metadata, layouts are not important.
        self.get_resource_state_value(state_key, None)
            .map(|maybe_state_value| maybe_state_value.map(StateValue::into_metadata))
    }
    fn get_resource_state_value_size(&self, state_key: &Self::Key) -> PartialVMResult<Option<u64>> {
        self.get_resource_state_value(state_key, None)
            .map(|maybe_state_value| maybe_state_value.map(|state_value| state_value.size() as u64))
    }

    fn resource_exists(&self, state_key: &Self::Key) -> PartialVMResult<bool> {
        // For existence, layouts are not important.
        self.get_resource_state_value(state_key, None)
            .map(|maybe_state_value| maybe_state_value.is_some())
    }
}

/// Metadata and exists queries for the resource group, determined by a key, must be resolved
/// via TResourceView's corresponding interfaces w. key (get_resource_state_value_metadata &
/// resource_exists). This simplifies interfaces for now, TODO: revisit later.
pub trait TResourceGroupView {
    type GroupKey;
    type ResourceTag;
    type Layout;

    /// Some resolvers might not be capable of the optimization, and should return false.
    /// Others might return based on the config or the run paramaters.
    fn is_resource_group_split_in_change_set_capable(&self) -> bool {
        false
    }

    /// The size of the resource group, based on the sizes of the latest entries at observed
    /// tags. During parallel execution, this is an estimated value that will get validated,
    /// but as long as it is assumed correct, the transaction can deterministically derive
    /// its behavior, e.g. charge the first access or write-related gas accordingly. The
    /// implementation ensures that resource_group_size, resource_exists, and .._metadata
    /// methods return somewhat consistent values (e.g. size != 0 if exists is true), and
    /// otherwise return an error as the validation is guaranteed to fail.
    ///
    /// The collected size is only guaranteed to correspond to the correct size when executed
    /// from a quiescent, correct state. The result can be viewed as a branch prediction in
    /// the parallel execution setting, as a wrong value will be (later) caught by validation.
    /// Thus, R/W conflicts are avoided, as long as the estimates are correct (e.g. updating
    /// struct members of a fixed size).
    fn resource_group_size(&self, group_key: &Self::GroupKey) -> PartialVMResult<ResourceGroupSize>;

    fn get_resource_from_group(
        &self,
        group_key: &Self::GroupKey,
        resource_tag: &Self::ResourceTag,
        maybe_layout: Option<&Self::Layout>,
    ) -> PartialVMResult<Option<Bytes>>;

    /// Needed for charging storage fees for a resource group write, as that requires knowing
    /// the size of the resource group AFTER the changeset of the transaction is applied (while
    /// the resource_group_size method provides the total group size BEFORE). To compute the
    /// AFTER size, for each modified resources within the group, the prior size can be
    /// determined by the following method (returns 0 for non-existent / deleted resources).
    fn resource_size_in_group(
        &self,
        group_key: &Self::GroupKey,
        resource_tag: &Self::ResourceTag,
    ) -> PartialVMResult<usize> {
        Ok(self
            .get_resource_from_group(group_key, resource_tag, None)?
            .map_or(0, |bytes| bytes.len()))
    }

    /// Needed for backwards compatibility with the additional safety mechanism for resource
    /// groups, where the violation of the following invariant causes transaction failure:
    /// - if a resource is modified or deleted it must already exist within a group,
    /// and if it is created, it must not previously exist.
    ///
    /// For normal resources, this is asserted, but for resource groups the behavior (that
    /// we maintain) is for the transaction to fail with INVARIANT_VIOLATION_ERROR.
    /// Thus, the state does not change and blockchain does not halt while the underlying
    /// issue is addressed. In order to maintain the behavior we check for resource existence,
    /// which in the context of parallel execution does not cause a full R/W conflict.
    fn resource_exists_in_group(
        &self,
        group_key: &Self::GroupKey,
        resource_tag: &Self::ResourceTag,
    ) -> PartialVMResult<bool> {
        self.get_resource_from_group(group_key, resource_tag, None)
            .map(|maybe_bytes| maybe_bytes.is_some())
    }

    fn release_group_cache(
        &self,
    ) -> Option<HashMap<Self::GroupKey, BTreeMap<Self::ResourceTag, Bytes>>>;
}

/// Allows to query modules from the state.
pub trait TModuleView {
    type Key;

    /// Returns
    ///   -  Ok(None)         if the module is not in storage,
    ///   -  Ok(Some(...))    if the module exists in storage,
    ///   -  Err(...)         otherwise (e.g. storage error).
    fn get_module_state_value(&self, state_key: &Self::Key) -> PartialVMResult<Option<StateValue>>;

    fn get_module_bytes(&self, state_key: &Self::Key) -> PartialVMResult<Option<Bytes>> {
        let maybe_state_value = self.get_module_state_value(state_key)?;
        Ok(maybe_state_value.map(|state_value| state_value.bytes().clone()))
    }

    fn get_module_state_value_metadata(
        &self,
        state_key: &Self::Key,
    ) -> PartialVMResult<Option<StateValueMetadata>> {
        let maybe_state_value = self.get_module_state_value(state_key)?;
        Ok(maybe_state_value.map(StateValue::into_metadata))
    }

    fn get_module_state_value_size(&self, state_key: &Self::Key) -> PartialVMResult<Option<u64>> {
        let maybe_state_value = self.get_module_state_value(state_key)?;
        Ok(maybe_state_value.map(|state_value| state_value.size() as u64))
    }

    fn module_exists(&self, state_key: &Self::Key) -> PartialVMResult<bool> {
        self.get_module_state_value(state_key)
            .map(|maybe_state_value| maybe_state_value.is_some())
    }
}

/// Allows to query state information, e.g. its usage.
pub trait StateStorageView {
    fn id(&self) -> StateViewId;

    fn get_usage(&self) -> anyhow::Result<StateStorageUsage>;
}

/// A fine-grained view of the state during execution.
///
/// - The `StateView` trait should be used by the storage backend, e.g. a DB.
///   It only allows a generic key-value access and always returns bytes or
///   state values.
/// - The `ExecutorView` trait is used at executor level, e.g. BlockSTM. When
///   a block is executed, the types of accesses are always known (for example,
///   whether a resource is accessed or a module). Fine-grained structure of
///   `ExecutorView` allows to:
///     1. Specialize on access type,
///     2. Separate execution and storage abstractions.
///
/// StateView currently has a basic implementation of the ExecutorView trait,
/// which is used across tests and basic applications in the system.
/// TODO: audit and reconsider the default implementation (e.g. should not
/// resolve AggregatorV2 via the state-view based default implementation, as it
/// doesn't provide a value exchange functionality).
pub trait TExecutorView<K, T, L, I, V>:
    TResourceView<Key = K, Layout = L>
    + TModuleView<Key = K>
    + TAggregatorV1View<Identifier = K>
    + TDelayedFieldView<Identifier = I, ResourceKey = K, ResourceGroupTag = T>
    + StateStorageView
{
}

impl<A, K, T, L, I, V> TExecutorView<K, T, L, I, V> for A where
    A: TResourceView<Key = K, Layout = L>
        + TModuleView<Key = K>
        + TAggregatorV1View<Identifier = K>
        + TDelayedFieldView<Identifier = I, ResourceKey = K, ResourceGroupTag = T>
        + StateStorageView
{
}

pub trait ExecutorView:
    TExecutorView<StateKey, StructTag, MoveTypeLayout, DelayedFieldID, WriteOp>
{
}

impl<T> ExecutorView for T where
    T: TExecutorView<StateKey, StructTag, MoveTypeLayout, DelayedFieldID, WriteOp>
{
}

pub trait ResourceGroupView:
    TResourceGroupView<GroupKey = StateKey, ResourceTag = StructTag, Layout = MoveTypeLayout>
{
}

impl<T> ResourceGroupView for T where
    T: TResourceGroupView<GroupKey = StateKey, ResourceTag = StructTag, Layout = MoveTypeLayout>
{
}
/// Direct implementations for StateView.
impl<S> TResourceView for S
where
    S: StateView,
{
    type Key = StateKey;
    type Layout = MoveTypeLayout;

    fn get_resource_state_value(
        &self,
        state_key: &Self::Key,
        _maybe_layout: Option<&Self::Layout>,
    ) -> PartialVMResult<Option<StateValue>> {
        self.get_state_value(state_key).map_err(Into::into)
    }
}

impl<S> TResourceGroupView for S
    where S: StateView {
    type GroupKey = StateKey;
    type ResourceTag = StructTag;
    type Layout = MoveTypeLayout;

    fn is_resource_group_split_in_change_set_capable(&self) -> bool {
        todo!()
    }

    fn resource_group_size(&self, group_key: &Self::GroupKey) -> PartialVMResult<ResourceGroupSize> {
        todo!()
    }

    fn get_resource_from_group(&self, group_key: &Self::GroupKey, resource_tag: &Self::ResourceTag, maybe_layout: Option<&Self::Layout>) -> PartialVMResult<Option<Bytes>> {
        todo!()
    }

    fn resource_size_in_group(&self, group_key: &Self::GroupKey, resource_tag: &Self::ResourceTag) -> PartialVMResult<usize> {
        todo!()
    }

    fn resource_exists_in_group(&self, group_key: &Self::GroupKey, resource_tag: &Self::ResourceTag) -> PartialVMResult<bool> {
        todo!()
    }

    fn release_group_cache(&self) -> Option<HashMap<Self::GroupKey, BTreeMap<Self::ResourceTag, Bytes>>> {
        todo!()
    }
}

impl<S> TModuleView for S
where
    S: StateView,
{
    type Key = StateKey;

    fn get_module_state_value(&self, state_key: &Self::Key) -> PartialVMResult<Option<StateValue>> {
        self.get_state_value(state_key).map_err(Into::into)
    }
}

impl<S> StateStorageView for S
where
    S: StateView,
{
    fn id(&self) -> StateViewId {
        self.id()
    }

    fn get_usage(&self) -> anyhow::Result<StateStorageUsage> {
        Ok(self.get_usage()?)
    }
}


/// Allows to query storage metadata in the VM session. Needed for storage refunds.
/// - Result being Err means storage error or some incostistency (e.g. during speculation,
/// needing to abort/halt the transaction with an error status).
/// - Ok(None) means that the corresponding data does not exist / was deleted.
/// - Ok(Some(_ : MetadataKind)) may be internally None (within Kind) if the metadata was
/// not previously provided (e.g. Legacy WriteOps).
pub trait StateValueMetadataResolver {
    fn get_module_state_value_metadata(
        &self,
        state_key: &StateKey,
    ) -> anyhow::Result<Option<StateValueMetadata>>;

    /// Can also be used to get the metadata of a resource group at a provided group key.
    fn get_resource_state_value_metadata(
        &self,
        state_key: &StateKey,
    ) -> anyhow::Result<Option<StateValueMetadata>>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceGroupSize {
    Concrete(u64),
    /// Combined represents what would the size be if we know individual
    /// parts that contribute to it. This is useful when individual parts
    /// are changing, and we want to know what the size of the group would be.
    ///
    /// Formula is based on how bcs serializes the BTreeMap:
    ///   varint encoding len(num_tagged_resources) + all_tagged_resources_size
    /// Also, if num_tagged_resources is 0, then the size is 0, because we will not store
    /// empty resource group in storage.
    Combined {
        num_tagged_resources: usize,
        all_tagged_resources_size: u64,
    },
}

pub fn size_u32_as_uleb128(mut value: usize) -> usize {
    let mut len = 1;
    while value >= 0x80 {
        // 7 (lowest) bits of data get written in a single byte.
        len += 1;
        value >>= 7;
    }
    len
}

impl ResourceGroupSize {
    pub fn zero_combined() -> Self {
        Self::Combined {
            num_tagged_resources: 0,
            all_tagged_resources_size: 0,
        }
    }

    pub fn zero_concrete() -> Self {
        Self::Concrete(0)
    }

    pub fn get(&self) -> u64 {
        match self {
            Self::Concrete(size) => *size,
            Self::Combined {
                num_tagged_resources,
                all_tagged_resources_size,
            } => {
                if *num_tagged_resources == 0 {
                    0
                } else {
                    size_u32_as_uleb128(*num_tagged_resources) as u64 + *all_tagged_resources_size
                }
            }
        }
    }
}

#[test]
fn test_size_u32_as_uleb128() {
    assert_eq!(size_u32_as_uleb128(0), 1);
    assert_eq!(size_u32_as_uleb128(127), 1);
    assert_eq!(size_u32_as_uleb128(128), 2);
    assert_eq!(size_u32_as_uleb128(128 * 128 - 1), 2);
    assert_eq!(size_u32_as_uleb128(128 * 128), 3);
}
