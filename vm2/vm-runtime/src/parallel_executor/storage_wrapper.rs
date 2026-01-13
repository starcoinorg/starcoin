// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytes::Bytes;
use dashmap::DashMap;
use move_binary_format::errors::PartialVMError;
use move_binary_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::{ModuleId, StructTag};
use move_core_types::metadata::Metadata;
use move_core_types::resolver::{resource_size, ModuleResolver, ResourceResolver};
use move_core_types::value::MoveTypeLayout;
use move_table_extension::{TableHandle, TableResolver};
use move_vm_types::delayed_values::delayed_field_id::{
    DelayedFieldID, ExtractUniqueIndex, TryFromMoveValue,
};
use move_vm_types::value_serde::{
    deserialize_and_allow_delayed_values, deserialize_and_replace_values_with_ids,
    serialize_and_allow_delayed_values, ValueToIdentifierMapping,
};
use move_vm_types::value_traversal::find_identifiers_in_value;
use starcoin_aggregator::bounded_math::{BoundedMath, SignedU128};
use starcoin_aggregator::delta_math::DeltaHistory;
use starcoin_aggregator::types::{
    code_invariant_error, DelayedFieldValue, DelayedFieldsSpeculativeError, PanicOr, ReadPosition,
};
use starcoin_mvhashmap::types::MVDelayedFieldsError;
use starcoin_mvhashmap::versioned_delayed_fields::TVersionedDelayedFieldView;
use starcoin_parallel_executor::delayed_field::{DelayedFieldRead, DelayedFieldReadKind};
use starcoin_parallel_executor::executor::MVHashMapView;
use starcoin_types::vm::config::starcoin_prod_deserializer_config;
use starcoin_vm_runtime_types::resolver::{ExecutorView, ResourceGroupSize, TResourceGroupView};
use starcoin_vm_runtime_types::resource_group_adapter::group_size_as_sum;
use starcoin_vm_types::errors::PartialVMResult;
use starcoin_vm_types::on_chain_config::{Features, OnChainConfig};
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::{
    errors::StateviewError, state_storage_usage::StateStorageUsage, state_value::StateValue,
    state_value::StateValueMetadata, StateView, TStateView,
};
use starcoin_vm_types::write_set::{TransactionWrite, WriteOp};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use crate::data_cache::get_resource_group_member_from_metadata;
use crate::move_vm_ext::{resource_state_key, AsExecutorView, ResourceGroupResolver};
use crate::parallel_executor::{ParallelStateKey, ParallelStateValue};

#[derive(Clone)]
struct CachedWriteOp {
    write_op: WriteOp,
    exchanged: bool,
}

impl CachedWriteOp {
    fn new(write_op: WriteOp, exchanged: bool) -> Self {
        Self {
            write_op,
            exchanged,
        }
    }
}

#[derive(Default)]
pub(crate) struct DelayedFieldCache {
    base_values: DashMap<StateKey, CachedWriteOp>,
    group_member_values: DashMap<(StateKey, StructTag), Bytes>,
}

impl DelayedFieldCache {
    pub fn get_base_value(&self, key: &StateKey) -> Option<WriteOp> {
        self.base_values
            .get(key)
            .map(|entry| entry.write_op.clone())
    }

    pub fn is_base_value_exchanged(&self, key: &StateKey) -> bool {
        self.base_values
            .get(key)
            .map(|entry| entry.exchanged)
            .unwrap_or(false)
    }

    pub fn get_or_insert_base_value<F>(&self, key: StateKey, exchanged: bool, build: F) -> WriteOp
    where
        F: FnOnce() -> WriteOp,
    {
        match self.base_values.entry(key) {
            dashmap::mapref::entry::Entry::Occupied(entry) => entry.get().write_op.clone(),
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                let value = build();
                entry.insert(CachedWriteOp::new(value.clone(), exchanged));
                value
            }
        }
    }

    pub fn insert_base_value(&self, key: StateKey, value: WriteOp, exchanged: bool) {
        let _ = self
            .base_values
            .insert(key, CachedWriteOp::new(value, exchanged));
    }

    #[allow(dead_code)]
    pub fn snapshot_base_values(&self) -> BTreeMap<StateKey, WriteOp> {
        self.base_values
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().write_op.clone()))
            .collect()
    }

    pub fn get_group_member_value(&self, group_key: &StateKey, tag: &StructTag) -> Option<Bytes> {
        self.group_member_values
            .get(&(group_key.clone(), tag.clone()))
            .map(|entry| entry.clone())
    }

    pub fn get_or_insert_group_member_value<F>(
        &self,
        group_key: StateKey,
        tag: StructTag,
        build: F,
    ) -> Bytes
    where
        F: FnOnce() -> Bytes,
    {
        match self.group_member_values.entry((group_key, tag)) {
            dashmap::mapref::entry::Entry::Occupied(entry) => entry.get().clone(),
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                let value = build();
                entry.insert(value.clone());
                value
            }
        }
    }

    pub fn insert_group_member_value(&self, group_key: StateKey, tag: StructTag, value: Bytes) {
        let _ = self.group_member_values.insert((group_key, tag), value);
    }

    pub fn remove_group_member_value(&self, group_key: &StateKey, tag: &StructTag) {
        let _ = self
            .group_member_values
            .remove(&(group_key.clone(), tag.clone()));
    }
}

#[derive(Clone)]
struct ResourceReadInfo {
    metadata: StateValueMetadata,
    size: u64,
    layout: Arc<MoveTypeLayout>,
    delayed_ids: HashSet<DelayedFieldID>,
}

#[derive(Clone)]
struct GroupReadInfo {
    metadata: StateValueMetadata,
    size: u64,
    delayed_ids: HashSet<DelayedFieldID>,
    layouts: BTreeMap<StructTag, Arc<MoveTypeLayout>>,
}

pub(crate) struct VersionedView<'a, S: StateView> {
    base_view: &'a S,
    hashmap_view: &'a MVHashMapView<'a, ParallelStateKey, ParallelStateValue>,
    delayed_field_cache: Arc<DelayedFieldCache>,
    delayed_fields_enabled: bool,
    accessed_groups: RefCell<HashSet<StateKey>>,
    resource_reads: RefCell<HashMap<StateKey, ResourceReadInfo>>,
    group_reads: RefCell<HashMap<StateKey, GroupReadInfo>>,
}

impl<'a, S: StateView> VersionedView<'a, S> {
    pub fn new(
        base_view: &'a S,
        hashmap_view: &'a MVHashMapView<'a, ParallelStateKey, ParallelStateValue>,
        delayed_field_cache: Arc<DelayedFieldCache>,
        delayed_fields_enabled: bool,
    ) -> Self {
        Self {
            base_view,
            hashmap_view,
            delayed_field_cache,
            delayed_fields_enabled,
            accessed_groups: RefCell::new(HashSet::new()),
            resource_reads: RefCell::new(HashMap::new()),
            group_reads: RefCell::new(HashMap::new()),
        }
    }

    #[allow(dead_code)]
    pub fn delayed_field_cache(&self) -> &DelayedFieldCache {
        &self.delayed_field_cache
    }

    fn read_parallel_resource(&self, state_key: &StateKey) -> Option<WriteOp> {
        self.hashmap_view
            .read(&ParallelStateKey::Resource(state_key.clone()))
            .and_then(|value| value.as_write_op().cloned())
    }

    fn read_parallel_group_member(
        &self,
        group_key: &StateKey,
        tag: &StructTag,
    ) -> Option<WriteOp> {
        self.hashmap_view
            .read(&ParallelStateKey::GroupMember {
                group_key: group_key.clone(),
                tag: tag.clone(),
            })
            .and_then(|value| value.as_write_op().cloned())
    }

    fn read_parallel_group_size(&self, group_key: &StateKey) -> Option<ResourceGroupSize> {
        self.hashmap_view
            .read(&ParallelStateKey::GroupSize(group_key.clone()))
            .and_then(|value| value.as_group_size())
    }

    pub fn take_group_read_layouts(
        &self,
    ) -> HashMap<StateKey, BTreeMap<StructTag, Arc<MoveTypeLayout>>> {
        self.group_reads
            .borrow_mut()
            .drain()
            .map(|(k, v)| (k, v.layouts))
            .collect()
    }

    fn record_resource_read(
        &self,
        key: &StateKey,
        state_value: &StateValue,
        layout: &MoveTypeLayout,
        delayed_ids: HashSet<DelayedFieldID>,
    ) {
        if delayed_ids.is_empty() {
            return;
        }
        let info = ResourceReadInfo {
            metadata: state_value.clone().into_metadata(),
            size: state_value.size() as u64,
            layout: Arc::new(layout.clone()),
            delayed_ids,
        };
        self.resource_reads
            .borrow_mut()
            .entry(key.clone())
            .and_modify(|existing| {
                existing
                    .delayed_ids
                    .extend(info.delayed_ids.iter().cloned());
            })
            .or_insert(info);
    }

    fn record_group_read(
        &self,
        group_key: &StateKey,
        metadata: StateValueMetadata,
        size: u64,
        tag: StructTag,
        layout: &MoveTypeLayout,
        delayed_ids: HashSet<DelayedFieldID>,
    ) {
        if delayed_ids.is_empty() {
            return;
        }
        self.group_reads
            .borrow_mut()
            .entry(group_key.clone())
            .and_modify(|existing| {
                existing.delayed_ids.extend(delayed_ids.iter().cloned());
                existing
                    .layouts
                    .insert(tag.clone(), Arc::new(layout.clone()));
            })
            .or_insert_with(|| {
                let mut layouts = BTreeMap::new();
                layouts.insert(tag.clone(), Arc::new(layout.clone()));
                GroupReadInfo {
                    metadata,
                    size,
                    delayed_ids,
                    layouts,
                }
            });
    }

    fn delayed_ids_from_bytes(
        &self,
        bytes: &Bytes,
        layout: &MoveTypeLayout,
    ) -> Result<HashSet<DelayedFieldID>, StateviewError> {
        let value = deserialize_and_allow_delayed_values(bytes, layout).ok_or_else(|| {
            StateviewError::Other("Failed to deserialize value for delayed field scan".to_string())
        })?;
        let mut ids: HashSet<u64> = HashSet::new();
        find_identifiers_in_value(&value, &mut ids).map_err(|e| {
            StateviewError::Other(format!("Failed to scan delayed field identifiers: {:?}", e))
        })?;
        Ok(ids.into_iter().map(DelayedFieldID::from).collect())
    }

    fn exchange_state_value(
        &self,
        _state_key: &StateKey,
        state_value: &StateValue,
        layout: &MoveTypeLayout,
    ) -> Result<(StateValue, HashSet<DelayedFieldID>), StateviewError> {
        struct Mapping<'a, S: StateView> {
            view: &'a VersionedView<'a, S>,
            delayed_ids: RefCell<HashSet<DelayedFieldID>>,
        }

        impl<S: StateView> ValueToIdentifierMapping for Mapping<'_, S> {
            type Identifier = DelayedFieldID;

            fn value_to_identifier(
                &self,
                kind: &move_core_types::value::IdentifierMappingKind,
                layout: &MoveTypeLayout,
                value: move_vm_types::values::Value,
            ) -> move_binary_format::errors::PartialVMResult<Self::Identifier> {
                let (base_value, width) =
                    DelayedFieldValue::try_from_move_value(layout, value, kind)?;
                let id = self.view.hashmap_view.generate_delayed_field_id(width);
                self.view
                    .hashmap_view
                    .delayed_fields()
                    .set_base_value(id, base_value);
                self.delayed_ids.borrow_mut().insert(id);
                Ok(id)
            }

            fn identifier_to_value(
                &self,
                _layout: &MoveTypeLayout,
                _identifier: Self::Identifier,
            ) -> move_binary_format::errors::PartialVMResult<move_vm_types::values::Value>
            {
                Err(move_binary_format::errors::PartialVMError::new(
                    move_core_types::vm_status::StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                ))
            }
        }

        let mapping = Mapping {
            view: self,
            delayed_ids: RefCell::new(HashSet::new()),
        };

        let value = deserialize_and_replace_values_with_ids(state_value.bytes(), layout, &mapping)
            .ok_or_else(|| {
                StateviewError::Other("Failed to replace delayed values with ids".to_string())
            })?;
        let serialized = serialize_and_allow_delayed_values(&value, layout)
            .map_err(|e| StateviewError::Other(e.to_string()))?
            .ok_or_else(|| {
                StateviewError::Other("Failed to serialize value with delayed ids".to_string())
            })?;

        let exchanged = StateValue::new_with_metadata(
            Bytes::from(serialized),
            state_value.clone().into_metadata(),
        );
        Ok((exchanged, mapping.delayed_ids.into_inner()))
    }

    fn read_state_value_impl(
        &self,
        state_key: &StateKey,
        maybe_layout: Option<&MoveTypeLayout>,
    ) -> Result<Option<StateValue>, StateviewError> {
        if let Some(write) = self.read_parallel_resource(state_key) {
            let maybe_state_value = write.as_state_value();
            if let (Some(layout), Some(state_value)) = (maybe_layout, maybe_state_value.as_ref()) {
                let ids = self.delayed_ids_from_bytes(state_value.bytes(), layout)?;
                self.record_resource_read(state_key, state_value, layout, ids);
            }
            return Ok(maybe_state_value);
        }

        if let Some(write) = self.delayed_field_cache.get_base_value(state_key) {
            let maybe_state_value = write.as_state_value();
            if let (Some(layout), Some(state_value)) = (maybe_layout, maybe_state_value.as_ref()) {
                if !self.delayed_field_cache.is_base_value_exchanged(state_key) {
                    let (exchanged, ids) = self
                        .exchange_state_value(state_key, state_value, layout)
                        .expect("exchange_state_value should succeed");
                    self.delayed_field_cache.insert_base_value(
                        state_key.clone(),
                        WriteOp::from_state_value(Some(exchanged.clone())),
                        true,
                    );
                    self.record_resource_read(state_key, &exchanged, layout, ids);
                    return Ok(Some(exchanged));
                }
                let ids = self.delayed_ids_from_bytes(state_value.bytes(), layout)?;
                self.record_resource_read(state_key, state_value, layout, ids);
            }
            return Ok(maybe_state_value);
        }

        let maybe_state_value = self.base_view.get_state_value(state_key)?;
        if let (Some(layout), Some(state_value)) = (maybe_layout, maybe_state_value.as_ref()) {
            let exchanged =
                self.delayed_field_cache
                    .get_or_insert_base_value(state_key.clone(), true, || {
                        let (value_with_ids, ids) = self
                            .exchange_state_value(state_key, state_value, layout)
                            .expect("exchange_state_value should succeed");
                        self.record_resource_read(state_key, &value_with_ids, layout, ids);
                        WriteOp::from_state_value(Some(value_with_ids))
                    });
            return Ok(exchanged.as_state_value());
        }

        Ok(maybe_state_value)
    }

    fn read_group_bytes(&self, group_key: &StateKey) -> Result<Option<Bytes>, StateviewError> {
        if let Some(write) = self.delayed_field_cache.get_base_value(group_key) {
            return Ok(write.bytes().cloned());
        }
        let maybe_state_value = self.base_view.get_state_value(group_key)?;
        if let Some(state_value) = &maybe_state_value {
            let _ =
                self.delayed_field_cache
                    .get_or_insert_base_value(group_key.clone(), false, || {
                        WriteOp::from_state_value(Some(state_value.clone()))
                    });
        }
        Ok(maybe_state_value.map(|v| v.bytes().clone()))
    }

    fn read_group_map(
        &self,
        group_key: &StateKey,
    ) -> Result<BTreeMap<StructTag, Bytes>, StateviewError> {
        let bytes = self.read_group_bytes(group_key)?;
        match bytes {
            None => Ok(BTreeMap::new()),
            Some(bytes) if bytes.is_empty() => Ok(BTreeMap::new()),
            Some(bytes) => Ok(bcs::from_bytes(&bytes)?),
        }
    }
}

impl<S: StateView> TStateView for VersionedView<'_, S> {
    type Key = StateKey;

    fn get_state_value(&self, state_key: &Self::Key) -> Result<Option<StateValue>, StateviewError> {
        self.read_state_value_impl(state_key, None)
    }

    fn get_state_value_with_layout(
        &self,
        state_key: &Self::Key,
        maybe_layout: Option<&MoveTypeLayout>,
    ) -> Result<Option<StateValue>, StateviewError> {
        self.read_state_value_impl(state_key, maybe_layout)
    }

    fn get_usage(&self) -> Result<StateStorageUsage, StateviewError> {
        unimplemented!("get_usage not implemented for VersionedView")
    }

    fn is_genesis(&self) -> bool {
        self.base_view.is_genesis()
    }
}

impl<S: StateView> TResourceGroupView for VersionedView<'_, S> {
    type GroupKey = StateKey;
    type ResourceTag = StructTag;
    type Layout = MoveTypeLayout;

    fn is_resource_group_split_in_change_set_capable(&self) -> bool {
        true
    }

    fn resource_group_size(
        &self,
        group_key: &Self::GroupKey,
    ) -> PartialVMResult<ResourceGroupSize> {
        if let Some(size) = self.read_parallel_group_size(group_key) {
            return Ok(size);
        }
        let group_map = self
            .read_group_map(group_key)
            .map_err(PartialVMError::from)?;
        group_size_as_sum(group_map.iter().map(|(t, v)| (t, v.len())))
    }

    fn get_resource_from_group(
        &self,
        group_key: &Self::GroupKey,
        resource_tag: &Self::ResourceTag,
        maybe_layout: Option<&MoveTypeLayout>,
    ) -> PartialVMResult<Option<Bytes>> {
        if let Some(write) = self.read_parallel_group_member(group_key, resource_tag) {
            let raw_bytes = match write.bytes() {
                None => return Ok(None),
                Some(bytes) => bytes.clone(),
            };

            if let Some(layout) = maybe_layout {
                let ids = self
                    .delayed_ids_from_bytes(&raw_bytes, layout)
                    .map_err(PartialVMError::from)?;
                let metadata = self
                    .read_state_value_impl(group_key, None)
                    .map_err(PartialVMError::from)?
                    .map(|v| v.into_metadata())
                    .unwrap_or_else(StateValueMetadata::none);
                let group_size =
                    TResourceGroupView::resource_group_size(self, group_key)?;
                self.record_group_read(
                    group_key,
                    metadata,
                    group_size.get(),
                    resource_tag.clone(),
                    layout,
                    ids,
                );
            }

            return Ok(Some(raw_bytes));
        }

        if let Some(layout) = maybe_layout {
            if let Some(cached) = self
                .delayed_field_cache
                .get_group_member_value(group_key, resource_tag)
            {
                let ids = self
                    .delayed_ids_from_bytes(&cached, layout)
                    .map_err(PartialVMError::from)?;
                let metadata = self
                    .read_state_value_impl(group_key, None)
                    .map_err(PartialVMError::from)?
                    .map(|v| v.into_metadata())
                    .unwrap_or_else(StateValueMetadata::none);
                let group_size =
                    TResourceGroupView::resource_group_size(self, group_key)?;
                self.record_group_read(
                    group_key,
                    metadata,
                    group_size.get(),
                    resource_tag.clone(),
                    layout,
                    ids,
                );
                return Ok(Some(cached));
            }
        }

        let group_map = self
            .read_group_map(group_key)
            .map_err(PartialVMError::from)?;
        let raw_bytes = match group_map.get(resource_tag) {
            None => return Ok(None),
            Some(bytes) => bytes.clone(),
        };

        if let Some(layout) = maybe_layout {
            let metadata = self
                .read_state_value_impl(group_key, None)
                .map_err(PartialVMError::from)?
                .map(|v| v.into_metadata())
                .unwrap_or_else(StateValueMetadata::none);
            let group_size =
                TResourceGroupView::resource_group_size(self, group_key)?;

            let exchanged = self.delayed_field_cache.get_or_insert_group_member_value(
                group_key.clone(),
                resource_tag.clone(),
                || {
                    let state_value =
                        StateValue::new_with_metadata(raw_bytes.clone(), metadata.clone());
                    let (value_with_ids, ids) = self
                        .exchange_state_value(group_key, &state_value, layout)
                        .expect("exchange_state_value should succeed");
                    self.record_group_read(
                        group_key,
                        metadata,
                        group_size.get(),
                        resource_tag.clone(),
                        layout,
                        ids,
                    );
                    value_with_ids.bytes().clone()
                },
            );
            return Ok(Some(exchanged));
        }

        Ok(Some(raw_bytes))
    }

    fn resource_size_in_group(
        &self,
        group_key: &Self::GroupKey,
        resource_tag: &Self::ResourceTag,
    ) -> PartialVMResult<usize> {
        if let Some(write) = self.read_parallel_group_member(group_key, resource_tag) {
            return Ok(write.bytes().map(|b| b.len()).unwrap_or(0));
        }
        let group_map = self
            .read_group_map(group_key)
            .map_err(PartialVMError::from)?;
        Ok(group_map.get(resource_tag).map(|b| b.len()).unwrap_or(0))
    }

    fn resource_exists_in_group(
        &self,
        group_key: &Self::GroupKey,
        resource_tag: &Self::ResourceTag,
    ) -> PartialVMResult<bool> {
        if let Some(write) = self.read_parallel_group_member(group_key, resource_tag) {
            return Ok(write.bytes().is_some());
        }
        let group_map = self
            .read_group_map(group_key)
            .map_err(PartialVMError::from)?;
        Ok(group_map.contains_key(resource_tag))
    }

    fn release_group_cache(
        &self,
    ) -> Option<HashMap<Self::GroupKey, BTreeMap<Self::ResourceTag, Bytes>>> {
        None
    }
}

impl<S: StateView> ModuleResolver for VersionedView<'_, S> {
    type Error = PartialVMError;

    fn get_module_metadata(&self, module_id: &ModuleId) -> Vec<Metadata> {
        let module = match self.get_module(module_id) {
            Ok(Some(module)) => module,
            _ => return vec![],
        };

        let features = Features::fetch_config(self).unwrap_or_default();
        let deserializer_config = starcoin_prod_deserializer_config(&features);
        let compiled_module =
            match CompiledModule::deserialize_with_config(&module, &deserializer_config) {
                Ok(module) => module,
                _ => return vec![],
            };

        compiled_module.metadata
    }

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Bytes>, Self::Error> {
        let key = StateKey::module_id(module_id);
        self.read_state_value_impl(&key, None)
            .map(|r| r.map(|v| v.bytes().clone()))
            .map_err(PartialVMError::from)
    }
}

impl<S: StateView> ResourceResolver for VersionedView<'_, S> {
    type Error = PartialVMError;

    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
        metadata: &[Metadata],
        maybe_layout: Option<&MoveTypeLayout>,
    ) -> Result<(Option<Bytes>, usize), Self::Error> {
        let resource_group = get_resource_group_member_from_metadata(struct_tag, metadata);
        if let Some(resource_group) = resource_group {
            let key = StateKey::resource_group(address, &resource_group);
            let buf = self.get_resource_from_group(&key, struct_tag, maybe_layout)?;

            let first_access = self.accessed_groups.borrow_mut().insert(key.clone());
            let group_size = if first_access {
                TResourceGroupView::resource_group_size(self, &key)?.get()
            } else {
                0
            };

            let buf_size = resource_size(&buf);
            Ok((buf, buf_size + group_size as usize))
        } else {
            let state_key = resource_state_key(address, struct_tag)?;
            let buf = if maybe_layout.is_some() {
                self.read_state_value_impl(&state_key, maybe_layout)
                    .map_err(PartialVMError::from)?
                    .map(|v| v.bytes().clone())
            } else {
                self.read_state_value_impl(&state_key, None)
                    .map_err(PartialVMError::from)?
                    .map(|v| v.bytes().clone())
            };
            let buf_size = resource_size(&buf);
            Ok((buf, buf_size))
        }
    }
}

impl<S: StateView> ResourceGroupResolver for VersionedView<'_, S> {
    fn release_resource_group_cache(
        &self,
    ) -> Option<HashMap<StateKey, BTreeMap<StructTag, Bytes>>> {
        self.release_group_cache()
    }

    fn resource_group_size(&self, group_key: &StateKey) -> PartialVMResult<ResourceGroupSize> {
        TResourceGroupView::resource_group_size(self, group_key)
    }

    fn resource_size_in_group(
        &self,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> PartialVMResult<usize> {
        TResourceGroupView::resource_size_in_group(self, group_key, resource_tag)
    }

    fn resource_exists_in_group(
        &self,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> PartialVMResult<bool> {
        TResourceGroupView::resource_exists_in_group(self, group_key, resource_tag)
    }
}

impl<S: StateView> TableResolver for VersionedView<'_, S> {
    fn resolve_table_entry_bytes_with_layout(
        &self,
        handle: &TableHandle,
        key: &[u8],
        _maybe_layout: Option<&MoveTypeLayout>,
    ) -> Result<Option<Bytes>, PartialVMError> {
        let state_key = StateKey::table_item(&(*handle).into(), key);
        self.read_state_value_impl(&state_key, None)
            .map(|r| r.map(|v| v.bytes().clone()))
            .map_err(PartialVMError::from)
    }
}

impl<S: StateView> AsExecutorView for VersionedView<'_, S> {
    fn as_executor_view(&self) -> &dyn ExecutorView {
        self
    }
}

fn get_delayed_field_value_impl(
    view: &MVHashMapView<'_, ParallelStateKey, ParallelStateValue>,
    id: &DelayedFieldID,
    capture_read: bool,
) -> Result<DelayedFieldValue, PanicOr<DelayedFieldsSpeculativeError>> {
    let cached = view.get_delayed_field_by_kind(id, DelayedFieldReadKind::Value);
    if let Some(DelayedFieldRead::Value { value }) = cached {
        return Ok(value);
    }

    loop {
        match view.delayed_fields().read(id, view.txn_idx()) {
            Ok(value) => {
                if capture_read {
                    view.capture_delayed_field_read(
                        *id,
                        false,
                        DelayedFieldRead::Value {
                            value: value.clone(),
                        },
                    )?;
                }
                return Ok(value);
            }
            Err(PanicOr::Or(MVDelayedFieldsError::Dependency(dep_idx))) => {
                if !view.wait_for_dependency(dep_idx) {
                    return Err(PanicOr::Or(DelayedFieldsSpeculativeError::InconsistentRead));
                }
            }
            Err(e) => {
                let mapped = match &e {
                    PanicOr::CodeInvariantError(err) => PanicOr::CodeInvariantError(err.clone()),
                    PanicOr::Or(MVDelayedFieldsError::NotFound) => {
                        PanicOr::Or(DelayedFieldsSpeculativeError::NotFound(*id))
                    }
                    _ => PanicOr::Or(DelayedFieldsSpeculativeError::InconsistentRead),
                };
                if capture_read {
                    view.capture_delayed_field_read_error(&mapped);
                }
                return Err(mapped);
            }
        }
    }
}

fn compute_history_read(
    base_delta: &SignedU128,
    delta: &SignedU128,
    max_value: u128,
    mut history: DeltaHistory,
    base_value: u128,
) -> Result<(bool, DelayedFieldRead), PanicOr<DelayedFieldsSpeculativeError>> {
    let math = BoundedMath::new(max_value);
    let before = DelayedFieldValue::Aggregator(base_value).into_aggregator_value()?;
    let before_value = expect_ok(math.unsigned_add_delta(before, base_delta))?;
    let result = if math.unsigned_add_delta(before_value, delta).is_err() {
        match delta {
            SignedU128::Positive(delta_value) => history.record_overflow(*delta_value),
            SignedU128::Negative(delta_value) => history.record_underflow(*delta_value),
        };
        false
    } else {
        let new_delta = expect_ok(math.signed_add(base_delta, delta))?;
        history.record_success(new_delta);
        true
    };

    Ok((
        result,
        DelayedFieldRead::HistoryBounded {
            restriction: history,
            max_value,
            inner_aggregator_value: base_value,
        },
    ))
}

fn compute_history_first_read(
    delta: &SignedU128,
    max_value: u128,
    base_value: u128,
) -> Result<(bool, DelayedFieldRead), PanicOr<DelayedFieldsSpeculativeError>> {
    let math = BoundedMath::new(max_value);
    let mut history = DeltaHistory::new();
    let result = if math.unsigned_add_delta(base_value, delta).is_err() {
        match delta {
            SignedU128::Positive(delta_value) => history.record_overflow(*delta_value),
            SignedU128::Negative(delta_value) => history.record_underflow(*delta_value),
        };
        false
    } else {
        history.record_success(*delta);
        true
    };
    Ok((
        result,
        DelayedFieldRead::HistoryBounded {
            restriction: history,
            max_value,
            inner_aggregator_value: base_value,
        },
    ))
}

fn expect_ok<V, E: std::fmt::Debug>(
    value: Result<V, E>,
) -> Result<V, PanicOr<DelayedFieldsSpeculativeError>> {
    value.map_err(|e| code_invariant_error(format!("Expected Ok, got Err({:?})", e)).into())
}

fn delayed_field_try_add_delta_outcome_impl(
    view: &MVHashMapView<'_, ParallelStateKey, ParallelStateValue>,
    id: &DelayedFieldID,
    base_delta: &SignedU128,
    delta: &SignedU128,
    max_value: u128,
) -> Result<bool, PanicOr<DelayedFieldsSpeculativeError>> {
    if delta.abs() > max_value {
        return Ok(false);
    }

    match view.get_delayed_field_by_kind(id, DelayedFieldReadKind::HistoryBounded) {
        Some(DelayedFieldRead::Value { value }) => {
            let math = BoundedMath::new(max_value);
            let before =
                expect_ok(math.unsigned_add_delta(value.into_aggregator_value()?, base_delta))?;
            return Ok(math.unsigned_add_delta(before, delta).is_ok());
        }
        Some(DelayedFieldRead::HistoryBounded {
            restriction,
            max_value: before_max,
            inner_aggregator_value,
        }) => {
            if before_max != max_value {
                return Err(
                    code_invariant_error("Cannot merge deltas with different limits").into(),
                );
            }
            let (result, read) = compute_history_read(
                base_delta,
                delta,
                max_value,
                restriction,
                inner_aggregator_value,
            )?;
            view.capture_delayed_field_read(*id, true, read)?;
            return Ok(result);
        }
        None => {}
    }

    if !base_delta.is_zero() {
        return Err(code_invariant_error(
            "Passed-in delta is not zero, but no delayed field read is recorded",
        )
        .into());
    }

    let predicted_value = loop {
        match view.delayed_fields().read_latest_predicted_value(
            id,
            view.txn_idx(),
            ReadPosition::BeforeCurrentTxn,
        ) {
            Ok(value) => break value,
            Err(MVDelayedFieldsError::Dependency(dep_idx)) => {
                if !view.wait_for_dependency(dep_idx) {
                    return Err(PanicOr::Or(DelayedFieldsSpeculativeError::InconsistentRead));
                }
            }
            Err(MVDelayedFieldsError::NotFound | MVDelayedFieldsError::DeltaApplicationFailure) => {
                return Err(PanicOr::Or(DelayedFieldsSpeculativeError::InconsistentRead));
            }
        }
    }
    .into_aggregator_value()?;

    let (result, read) = compute_history_first_read(delta, max_value, predicted_value)?;
    view.capture_delayed_field_read(*id, false, read)?;
    Ok(result)
}

impl<S: StateView> starcoin_aggregator::resolver::TDelayedFieldView for VersionedView<'_, S> {
    type Identifier = DelayedFieldID;
    type ResourceKey = StateKey;
    type ResourceGroupTag = StructTag;

    fn is_delayed_field_optimization_capable(&self) -> bool {
        self.delayed_fields_enabled
    }

    fn get_delayed_field_value(
        &self,
        id: &Self::Identifier,
    ) -> Result<DelayedFieldValue, PanicOr<DelayedFieldsSpeculativeError>> {
        get_delayed_field_value_impl(self.hashmap_view, id, true)
    }

    fn delayed_field_try_add_delta_outcome(
        &self,
        id: &Self::Identifier,
        base_delta: &SignedU128,
        delta: &SignedU128,
        max_value: u128,
    ) -> Result<bool, PanicOr<DelayedFieldsSpeculativeError>> {
        delayed_field_try_add_delta_outcome_impl(
            self.hashmap_view,
            id,
            base_delta,
            delta,
            max_value,
        )
    }

    fn generate_delayed_field_id(&self, width: u32) -> Self::Identifier {
        self.hashmap_view.generate_delayed_field_id(width)
    }

    fn validate_delayed_field_id(
        &self,
        id: &Self::Identifier,
    ) -> Result<(), starcoin_types::delayed_fields::PanicError> {
        let unique_index = id.extract_unique_index();
        let start = self.hashmap_view.delayed_field_id_start();
        let current = self.hashmap_view.delayed_field_id_counter();
        if unique_index < start || unique_index >= current {
            return Err(code_invariant_error(format!(
                "Invalid delayed field id: {:?} with index: {} (started from {} and reached {})",
                id, unique_index, start, current
            )));
        }
        Ok(())
    }

    fn get_reads_needing_exchange(
        &self,
        delayed_write_set_ids: &HashSet<Self::Identifier>,
        skip: &HashSet<Self::ResourceKey>,
    ) -> Result<
        BTreeMap<Self::ResourceKey, (StateValueMetadata, u64, Arc<MoveTypeLayout>)>,
        starcoin_types::delayed_fields::PanicError,
    > {
        let mut result = BTreeMap::new();
        for (key, info) in self.resource_reads.borrow().iter() {
            if skip.contains(key) {
                continue;
            }
            if info
                .delayed_ids
                .iter()
                .any(|id| delayed_write_set_ids.contains(id))
            {
                result.insert(
                    key.clone(),
                    (info.metadata.clone(), info.size, info.layout.clone()),
                );
            }
        }
        Ok(result)
    }

    fn get_group_reads_needing_exchange(
        &self,
        delayed_write_set_ids: &HashSet<Self::Identifier>,
        skip: &HashSet<Self::ResourceKey>,
    ) -> PartialVMResult<BTreeMap<Self::ResourceKey, (StateValueMetadata, u64)>> {
        let mut result = BTreeMap::new();
        for (key, info) in self.group_reads.borrow().iter() {
            if skip.contains(key) {
                continue;
            }
            if info
                .delayed_ids
                .iter()
                .any(|id| delayed_write_set_ids.contains(id))
            {
                result.insert(key.clone(), (info.metadata.clone(), info.size));
            }
        }
        Ok(result)
    }
}
