// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytes::Bytes;
use move_core_types::value::MoveTypeLayout;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::state_storage_usage::StateStorageUsage;
use starcoin_vm_types::state_store::state_value::{StateValue, StateValueMetadata};
use starcoin_vm_types::state_store::StateViewId;
use starcoin_vm_types::state_view::StateView;

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
    ) -> anyhow::Result<Option<StateValue>>;

    fn get_resource_bytes(
        &self,
        state_key: &Self::Key,
        maybe_layout: Option<&Self::Layout>,
    ) -> anyhow::Result<Option<Bytes>> {
        let maybe_state_value = self.get_resource_state_value(state_key, maybe_layout)?;
        Ok(maybe_state_value.map(|state_value| state_value.bytes().clone()))
    }

    fn get_resource_state_value_metadata(
        &self,
        state_key: &Self::Key,
    ) -> anyhow::Result<Option<StateValueMetadata>> {
        // For metadata, layouts are not important.
        self.get_resource_state_value(state_key, None)
            .map(|maybe_state_value| maybe_state_value.map(StateValue::into_metadata))
    }

    fn resource_exists(&self, state_key: &Self::Key) -> anyhow::Result<bool> {
        // For existence, layouts are not important.
        self.get_resource_state_value(state_key, None)
            .map(|maybe_state_value| maybe_state_value.is_some())
    }
}

/// Allows to query modules from the state.
pub trait TModuleView {
    type Key;

    /// Returns
    ///   -  Ok(None)         if the module is not in storage,
    ///   -  Ok(Some(...))    if the module exists in storage,
    ///   -  Err(...)         otherwise (e.g. storage error).
    fn get_module_state_value(&self, state_key: &Self::Key) -> anyhow::Result<Option<StateValue>>;

    fn get_module_bytes(&self, state_key: &Self::Key) -> anyhow::Result<Option<Bytes>> {
        let maybe_state_value = self.get_module_state_value(state_key)?;
        Ok(maybe_state_value.map(|state_value| state_value.bytes().clone()))
    }

    fn get_module_state_value_metadata(
        &self,
        state_key: &Self::Key,
    ) -> anyhow::Result<Option<StateValueMetadata>> {
        let maybe_state_value = self.get_module_state_value(state_key)?;
        Ok(maybe_state_value.map(StateValue::into_metadata))
    }

    fn module_exists(&self, state_key: &Self::Key) -> anyhow::Result<bool> {
        self.get_module_state_value(state_key)
            .map(|maybe_state_value| maybe_state_value.is_some())
    }
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
pub trait TExecutorView<K, L>: TResourceView<Key = K, Layout = L> + TModuleView<Key = K> {}

impl<A, K, L> TExecutorView<K, L> for A where
    A: TResourceView<Key = K, Layout = L> + TModuleView<Key = K>
{
}

pub trait ExecutorView: TExecutorView<StateKey, MoveTypeLayout> {}

impl<T> ExecutorView for T where T: TExecutorView<StateKey, MoveTypeLayout> {}

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
    ) -> anyhow::Result<Option<StateValue>> {
        self.get_state_value(state_key)
    }
}

impl<S> TModuleView for S
where
    S: StateView,
{
    type Key = StateKey;

    fn get_module_state_value(&self, state_key: &Self::Key) -> anyhow::Result<Option<StateValue>> {
        self.get_state_value(state_key)
    }
}

/// Allows to query modules from the state.
pub trait StateStorageView {
    fn id(&self) -> StateViewId;

    fn get_usage(&self) -> Result<StateStorageUsage, StateviewError>;
}

impl<S> StateStorageView for S
    where
        S: StateView,
{
    fn id(&self) -> StateViewId {
        self.id()
    }

    fn get_usage(&self) -> Result<StateStorageUsage, StateviewError> {
        self.get_usage().map_err(Into::into)
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

pub fn size_u32_as_uleb128(mut value: usize) -> usize {
    let mut len = 1;
    while value >= 0x80 {
        // 7 (lowest) bits of data get written in a single byte.
        len += 1;
        value >>= 7;
    }
    len
}

#[test]
fn test_size_u32_as_uleb128() {
    assert_eq!(size_u32_as_uleb128(0), 1);
    assert_eq!(size_u32_as_uleb128(127), 1);
    assert_eq!(size_u32_as_uleb128(128), 2);
    assert_eq!(size_u32_as_uleb128(128 * 128 - 1), 2);
    assert_eq!(size_u32_as_uleb128(128 * 128), 3);
}
