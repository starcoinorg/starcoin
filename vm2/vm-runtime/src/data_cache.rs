// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Scratchpad for on chain values during the execution.

use crate::default_gas_schedule;
use crate::move_vm_ext::{resource_state_key, AsExecutorView, ResourceGroupResolver};
use bytes::Bytes;
use move_binary_format::deserializer::DeserializerConfig;
use move_binary_format::CompiledModule;
use move_bytecode_utils::compiled_module_viewer::CompiledModuleView;
use move_core_types::metadata::Metadata;
use move_core_types::resolver::{resource_size, ModuleResolver, ResourceResolver};
use move_core_types::value::{IdentifierMappingKind, MoveStructLayout, MoveTypeLayout};
use move_table_extension::{TableHandle, TableResolver};
use move_vm_runtime::config::DEFAULT_MAX_VALUE_NEST_DEPTH;
use move_vm_types::delayed_values::delayed_field_id::{
    DelayedFieldID, ExtractUniqueIndex, ExtractWidth, TryFromMoveValue,
};
use move_vm_types::loaded_data::runtime_types::TypeBuilder;
use move_vm_types::value_serde::{ValueSerDeContext, ValueToIdentifierMapping};
use move_vm_types::value_traversal::find_identifiers_in_value;
use starcoin_aggregator::bounded_math::{BoundedMath, SignedU128};
use starcoin_aggregator::resolver::TDelayedFieldView;
use starcoin_aggregator::types::{
    code_invariant_error, DelayedFieldValue, DelayedFieldsSpeculativeError, PanicOr, ReadPosition,
};
use starcoin_logger::prelude::*;
use starcoin_mvhashmap::types::MVDelayedFieldsError;
use starcoin_mvhashmap::versioned_delayed_fields::{
    TVersionedDelayedFieldView, VersionedDelayedFields,
};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::vm::config::{starcoin_prod_deserializer_config, starcoin_prod_vm_config};
use starcoin_vm_runtime_types::resolver::{
    ExecutorView, ResourceGroupSize, TResourceGroupView, TResourceView,
};
use starcoin_vm_runtime_types::resource_group_adapter::ResourceGroupAdapter;
use starcoin_vm_types::on_chain_config::{Features, OnChainConfig, TimedFeaturesBuilder, VMConfig};
use starcoin_vm_types::state_store::{
    errors::StateviewError,
    state_key::StateKey,
    state_storage_usage::StateStorageUsage,
    state_value::{StateValue, StateValueMetadata},
    StateView, TStateView,
};
use starcoin_vm_types::{
    errors::*,
    language_storage::{ModuleId, StructTag},
    transaction::TransactionOutput,
    vm_status::{StatusCode, VMStatus},
    write_set::{TransactionWrite, WriteOp, WriteSet},
};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::{
    cell::RefCell,
    collections::btree_map::BTreeMap,
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use crate::parallel_executor::{
    materialize_events, materialize_resource_write_set, storage_wrapper::DelayedFieldCache,
};
use starcoin_types::delayed_fields::PanicError;
use starcoin_vm_runtime_types::output::VMOutput;
use std::sync::atomic::AtomicU64;
use std::sync::LazyLock;
use std::time::Instant;

#[derive(Debug, Default, Clone)]
pub(crate) struct ResourceGroupStatsSnapshot {
    pub group_accesses: u64,
    pub group_cache_hits: u64,
    pub group_member_calls: u64,
    pub group_member_ns: u64,
    pub group_size_calls: u64,
    pub group_size_ns: u64,
}

#[derive(Debug, Default)]
struct ResourceGroupStats {
    group_accesses: AtomicU64,
    group_cache_hits: AtomicU64,
    group_member_calls: AtomicU64,
    group_member_ns: AtomicU64,
    group_size_calls: AtomicU64,
    group_size_ns: AtomicU64,
}

static RESOURCE_GROUP_STATS: LazyLock<ResourceGroupStats> =
    LazyLock::new(ResourceGroupStats::default);
#[cfg(debug_assertions)]
static EXCHANGE_DUMP_SEQ: AtomicU64 = AtomicU64::new(0);

#[doc(hidden)]
pub fn nested_native_u64_kind_for_manual_exchange(
    layout: &MoveTypeLayout,
) -> Option<IdentifierMappingKind> {
    let (kind, native_layout) = match layout {
        MoveTypeLayout::Struct(outer) => {
            let outer_fields = outer.fields();
            if outer_fields.len() != 1 {
                return None;
            }
            match &outer_fields[0] {
                MoveTypeLayout::Struct(inner) => {
                    let inner_fields = inner.fields();
                    if inner_fields.len() != 2 {
                        return None;
                    }
                    match (&inner_fields[0], &inner_fields[1]) {
                        (MoveTypeLayout::Native(kind, native_layout), MoveTypeLayout::U64) => {
                            (kind, native_layout.as_ref())
                        }
                        _ => return None,
                    }
                }
                _ => return None,
            }
        }
        _ => return None,
    };

    if !matches!(native_layout, MoveTypeLayout::U64) {
        return None;
    }

    match kind {
        IdentifierMappingKind::Aggregator => Some(IdentifierMappingKind::Aggregator),
        IdentifierMappingKind::Snapshot => Some(IdentifierMappingKind::Snapshot),
        _ => None,
    }
}

#[doc(hidden)]
pub fn manual_exchange_bytes_for_nested_native_u64(
    kind: IdentifierMappingKind,
    bytes: &[u8],
    delayed_field_id: DelayedFieldID,
) -> Result<(Vec<u8>, DelayedFieldValue), StateviewError> {
    let width = delayed_field_id.extract_width();
    if width != 8 {
        return Err(StateviewError::Other(format!(
            "Manual exchange expected delayed field width 8, got {}",
            width
        )));
    }

    if bytes.len() != 16 {
        return Err(StateviewError::Other(format!(
            "Manual exchange expected 16 bytes, got {}",
            bytes.len()
        )));
    }

    let native_bytes: [u8; 8] = bytes[0..8]
        .try_into()
        .map_err(|_| StateviewError::Other("Failed to parse native u64 bytes".to_string()))?;
    let native_value = u64::from_le_bytes(native_bytes);
    let delayed_value = match kind {
        IdentifierMappingKind::Aggregator => DelayedFieldValue::Aggregator(native_value as u128),
        IdentifierMappingKind::Snapshot => DelayedFieldValue::Snapshot(native_value as u128),
        _ => {
            return Err(StateviewError::Other(format!(
                "Unsupported mapping kind for manual exchange: {:?}",
                kind
            )));
        }
    };

    let mut exchanged = bytes.to_vec();
    exchanged[0..8].copy_from_slice(&delayed_field_id.as_u64().to_le_bytes());
    Ok((exchanged, delayed_value))
}

pub(crate) fn take_resource_group_stats() -> ResourceGroupStatsSnapshot {
    ResourceGroupStatsSnapshot {
        group_accesses: RESOURCE_GROUP_STATS
            .group_accesses
            .swap(0, Ordering::Relaxed),
        group_cache_hits: RESOURCE_GROUP_STATS
            .group_cache_hits
            .swap(0, Ordering::Relaxed),
        group_member_calls: RESOURCE_GROUP_STATS
            .group_member_calls
            .swap(0, Ordering::Relaxed),
        group_member_ns: RESOURCE_GROUP_STATS
            .group_member_ns
            .swap(0, Ordering::Relaxed),
        group_size_calls: RESOURCE_GROUP_STATS
            .group_size_calls
            .swap(0, Ordering::Relaxed),
        group_size_ns: RESOURCE_GROUP_STATS
            .group_size_ns
            .swap(0, Ordering::Relaxed),
    }
}

pub(crate) fn effective_max_value_nest_depth<S: StateView>(state_view: &S) -> Option<u64> {
    let features = Features::fetch_config(state_view).unwrap_or_default();
    let timed_features = TimedFeaturesBuilder::enable_all().build();
    starcoin_prod_vm_config(&features, &timed_features, TypeBuilder::Legacy)
        .max_value_nest_depth
        .or(Some(DEFAULT_MAX_VALUE_NEST_DEPTH))
}

pub fn get_resource_group_member_from_metadata(
    struct_tag: &StructTag,
    metadata: &[Metadata],
) -> Option<StructTag> {
    let metadata = starcoin_framework::get_metadata(metadata)?;
    metadata
        .struct_attributes
        .get(struct_tag.name.as_ident_str().as_str())?
        .iter()
        .find_map(|attr| attr.get_resource_group_member())
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

/// Adapter to convert a `ExecutorView` into a `MoveResolver`.
///
/// Resources in groups are handled either through dedicated interfaces of executor_view
/// (that tie to specialized handling in block executor), or via 'standard' interfaces
/// for (non-group) resources and subsequent handling in the StorageAdapter itself.
pub struct StorageAdapter<'e, E> {
    executor_view: &'e E,
    deserializer_config: DeserializerConfig,
    max_value_nest_depth: Option<u64>,
    resource_group_view: ResourceGroupAdapter<'e>,
    delayed_fields_enabled: bool,
    accessed_groups: RefCell<HashSet<StateKey>>,
    delayed_field_cache: DelayedFieldCache,
    delayed_fields: VersionedDelayedFields<DelayedFieldID>,
    resource_reads: RefCell<HashMap<StateKey, ResourceReadInfo>>,
    group_reads: RefCell<HashMap<StateKey, GroupReadInfo>>,
    delayed_field_id_start: u32,
    delayed_field_id_counter: AtomicU32,
}

/// A local cache for a given a `StateView`. The cache is private to the Diem layer
/// but can be used as a one shot cache for systems that need a simple `RemoteCache`
/// implementation (e.g. tests or benchmarks).
///
/// The cache is responsible to track all changes to the `StateView` that are the result
/// of transaction execution. Those side effects are published at the end of a transaction
/// execution via `StateViewCache::push_write_set`.
///
/// `StateViewCache` is responsible to give an up to date view over the data store,
/// so that changes executed but not yet committed are visible to subsequent transactions.
///
/// If a system wishes to execute a block of transaction on a given view, a cache that keeps
/// track of incremental changes is vital to the consistency of the data store and the system.
pub struct StateViewCache<'a, S> {
    data_view: &'a S,
    data_map: BTreeMap<StateKey, WriteOp>,
}

impl<'a, S: StateView> StateViewCache<'a, S> {
    /// Create a `StateViewCache` give a `StateView`. Hold updates to the data store and
    /// forward data request to the `StateView` if not in the local cache.
    pub fn new(data_view: &'a S) -> Self {
        StateViewCache {
            data_view,
            data_map: BTreeMap::new(),
        }
    }

    // Publishes a `WriteSet` computed at the end of a transaction.
    // The effect is to build a layer in front of the `StateView` which keeps
    // track of the data as if the changes were applied immediately.
    pub(crate) fn push_write_set(&mut self, write_set: &WriteSet) -> Result<(), StateviewError> {
        for (key, write_op) in write_set.iter() {
            use std::collections::btree_map::Entry::*;
            match self.data_map.entry(key.clone()) {
                Vacant(entry) => {
                    entry.insert(write_op.clone());
                }
                Occupied(mut entry) => {
                    if !WriteOp::squash(entry.get_mut(), write_op.clone())? {
                        entry.remove();
                    }
                }
            }
        }
        Ok(())
    }
}

impl<S: StateView> TStateView for StateViewCache<'_, S> {
    type Key = StateKey;

    // Get some data either through the cache or the `StateView` on a cache miss.
    fn get_state_value(&self, state_key: &Self::Key) -> Result<Option<StateValue>, StateviewError> {
        match self.data_map.get(state_key) {
            Some(opt_data) => Ok(opt_data.bytes().map(|bytes| {
                StateValue::new_with_metadata(bytes.clone(), opt_data.metadata().clone())
            })),
            None => match self.data_view.get_state_value(state_key) {
                Ok(remote_data) => Ok(remote_data),
                // TODO: should we forward some error info?
                Err(e) => {
                    error!("[VM] Error getting data from storage for {:?}", state_key);
                    Err(e)
                }
            },
        }
    }

    fn get_usage(&self) -> Result<StateStorageUsage, StateviewError> {
        todo!()
    }

    fn is_genesis(&self) -> bool {
        self.data_view.is_genesis()
    }
}

impl<'a, S: StateView> StorageAdapter<'a, S> {
    fn layout_has_identifier_mappings(layout: &MoveTypeLayout) -> bool {
        match layout {
            MoveTypeLayout::Native(..) => true,
            MoveTypeLayout::Vector(inner) => Self::layout_has_identifier_mappings(inner),
            MoveTypeLayout::Struct(struct_layout) => match struct_layout {
                MoveStructLayout::Runtime(fields) => {
                    fields.iter().any(Self::layout_has_identifier_mappings)
                }
                MoveStructLayout::WithFields(fields) => fields
                    .iter()
                    .any(|field| Self::layout_has_identifier_mappings(&field.layout)),
                MoveStructLayout::WithTypes { fields, .. } => fields
                    .iter()
                    .any(|field| Self::layout_has_identifier_mappings(&field.layout)),
            },
            _ => false,
        }
    }

    pub fn new(
        state_store: &'a S,
        deserializer_config: DeserializerConfig,
        max_value_nest_depth: Option<u64>,
        resource_group_view: ResourceGroupAdapter<'a>,
        delayed_fields_enabled: bool,
    ) -> Self {
        let delayed_field_id_start = 0;
        Self {
            executor_view: state_store,
            deserializer_config,
            max_value_nest_depth,
            resource_group_view,
            delayed_fields_enabled,
            accessed_groups: RefCell::new(HashSet::new()),
            delayed_field_cache: DelayedFieldCache::default(),
            delayed_fields: VersionedDelayedFields::empty(),
            resource_reads: RefCell::new(HashMap::new()),
            group_reads: RefCell::new(HashMap::new()),
            delayed_field_id_start,
            delayed_field_id_counter: AtomicU32::new(delayed_field_id_start),
        }
    }

    pub fn get(&self, key: &StateKey) -> Result<Option<StateValue>, PartialVMError> {
        self.executor_view
            .get_state_value(key)
            .map_err(|_| PartialVMError::new(StatusCode::STORAGE_ERROR))
    }

    #[allow(dead_code)]
    pub(crate) fn delayed_field_cache(&self) -> &DelayedFieldCache {
        &self.delayed_field_cache
    }

    pub fn delayed_fields(&self) -> &VersionedDelayedFields<DelayedFieldID> {
        &self.delayed_fields
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

    fn generate_delayed_field_id(&self, width: u32) -> DelayedFieldID {
        let index = self.delayed_field_id_counter.fetch_add(1, Ordering::SeqCst);
        DelayedFieldID::new_with_width(index, width)
    }

    #[allow(dead_code)]
    fn delayed_ids_from_bytes(
        &self,
        bytes: &Bytes,
        layout: &MoveTypeLayout,
    ) -> Result<HashSet<DelayedFieldID>, StateviewError> {
        if !Self::layout_has_identifier_mappings(layout) {
            return Ok(HashSet::new());
        }
        let value = ValueSerDeContext::<DelayedFieldID>::new(self.max_value_nest_depth)
            .with_delayed_fields_serde()
            .deserialize(bytes, layout)
            .ok_or_else(|| {
                StateviewError::Other(
                    "Failed to deserialize value for delayed field scan".to_string(),
                )
            })?;
        let mut ids: HashSet<u64> = HashSet::new();
        find_identifiers_in_value(&value, &mut ids).map_err(|e| {
            StateviewError::Other(format!("Failed to scan delayed field identifiers: {:?}", e))
        })?;
        Ok(ids.into_iter().map(DelayedFieldID::from).collect())
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

    fn exchange_state_value(
        &self,
        state_value: &StateValue,
        layout: &MoveTypeLayout,
    ) -> Result<(StateValue, HashSet<DelayedFieldID>), StateviewError> {
        if let Some(exchanged) =
            self.try_manual_exchange_for_nested_native_u64(layout, state_value)?
        {
            return Ok(exchanged);
        }

        struct Mapping<'a, S: StateView> {
            adapter: &'a StorageAdapter<'a, S>,
            delayed_ids: RefCell<HashSet<DelayedFieldID>>,
        }

        impl<S: StateView> ValueToIdentifierMapping for Mapping<'_, S> {
            type Identifier = DelayedFieldID;

            fn value_to_identifier(
                &self,
                kind: &move_core_types::value::IdentifierMappingKind,
                layout: &MoveTypeLayout,
                value: move_vm_types::values::Value,
            ) -> Result<Self::Identifier, PartialVMError> {
                let (base_value, width) =
                    DelayedFieldValue::try_from_move_value(layout, value, kind)?;
                let id = self.adapter.generate_delayed_field_id(width);
                self.adapter.delayed_fields.set_base_value(id, base_value);
                self.delayed_ids.borrow_mut().insert(id);
                Ok(id)
            }

            fn identifier_to_value(
                &self,
                _layout: &MoveTypeLayout,
                _identifier: Self::Identifier,
            ) -> Result<move_vm_types::values::Value, PartialVMError> {
                Err(PartialVMError::new(
                    StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                ))
            }
        }

        let mapping = Mapping {
            adapter: self,
            delayed_ids: RefCell::new(HashSet::new()),
        };

        let value = ValueSerDeContext::<DelayedFieldID>::new(self.max_value_nest_depth)
            .with_delayed_fields_replacement(&mapping)
            .deserialize(state_value.bytes(), layout)
            .ok_or_else(|| {
                StateviewError::Other("Failed to replace delayed values with ids".to_string())
            })?;
        let serialized = ValueSerDeContext::<DelayedFieldID>::new(self.max_value_nest_depth)
            .with_delayed_fields_serde()
            .serialize(&value, layout)
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

    // Fallback for the known crashing release path in move-vm-types deserialization:
    // Struct([Struct([Native(Aggregator|Snapshot, U64), U64])]).
    // This performs the same exchange as deserialize_and_replace_values_with_ids:
    // replace the first field with delayed id bytes and keep the second field as-is.
    fn try_manual_exchange_for_nested_native_u64(
        &self,
        layout: &MoveTypeLayout,
        state_value: &StateValue,
    ) -> Result<Option<(StateValue, HashSet<DelayedFieldID>)>, StateviewError> {
        let Some(kind) = nested_native_u64_kind_for_manual_exchange(layout) else {
            return Ok(None);
        };
        let id = self.generate_delayed_field_id(8);
        let (exchanged, delayed_value) =
            manual_exchange_bytes_for_nested_native_u64(kind, state_value.bytes(), id)?;
        self.delayed_fields.set_base_value(id, delayed_value);

        let exchanged_state = StateValue::new_with_metadata(
            Bytes::from(exchanged),
            state_value.clone().into_metadata(),
        );
        let mut delayed_ids = HashSet::new();
        delayed_ids.insert(id);

        warn!("[vm2-delayed] manual exchange fallback applied for nested native U64 layout");
        Ok(Some((exchanged_state, delayed_ids)))
    }

    fn maybe_exchange_state_value(
        &self,
        state_value: &StateValue,
        layout: &MoveTypeLayout,
    ) -> Result<(StateValue, HashSet<DelayedFieldID>, bool), StateviewError> {
        if !Self::layout_has_identifier_mappings(layout) {
            return Ok((state_value.clone(), HashSet::new(), false));
        }
        let (exchanged, delayed_ids) = self.exchange_state_value(state_value, layout)?;
        Ok((exchanged, delayed_ids, true))
    }

    #[cfg(debug_assertions)]
    fn maybe_dump_exchange_input(
        &self,
        scope: &str,
        state_key: &StateKey,
        struct_tag: &StructTag,
        state_value: &StateValue,
        layout: &MoveTypeLayout,
    ) {
        if std::env::var_os("STARCOIN_VM2_DUMP_EXCHANGE_INPUT").is_none() {
            return;
        }

        // Keep dumps focused on the crashing path by default.
        if !(struct_tag.address == AccountAddress::ONE
            && struct_tag.module.as_ident_str().as_str() == "fungible_asset"
            && struct_tag.name.as_ident_str().as_str() == "ConcurrentFungibleBalance")
        {
            return;
        }

        let dump_dir = std::env::var("STARCOIN_VM2_DUMP_DIR")
            .unwrap_or_else(|_| "/tmp/starcoin_vm2_exchange_dump".to_string());
        if let Err(err) = std::fs::create_dir_all(&dump_dir) {
            warn!(
                "[vm2-delayed] failed to create dump dir {}: {:?}",
                dump_dir, err
            );
            return;
        }

        let seq = EXCHANGE_DUMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let base = format!("{}/case-{}-{}", dump_dir, std::process::id(), seq);
        let value_path = format!("{}.value.bin", base);
        let layout_bcs_path = format!("{}.layout.bcs", base);
        let layout_debug_path = format!("{}.layout.debug.txt", base);
        let context_path = format!("{}.context.txt", base);

        if let Err(err) = std::fs::write(&value_path, state_value.bytes()) {
            warn!(
                "[vm2-delayed] failed to write dump file {}: {:?}",
                value_path, err
            );
            return;
        }

        let layout_bcs = match bcs::to_bytes(layout) {
            Ok(bytes) => bytes,
            Err(err) => {
                warn!(
                    "[vm2-delayed] failed to serialize layout for dump {}: {:?}",
                    layout_bcs_path, err
                );
                return;
            }
        };
        if let Err(err) = std::fs::write(&layout_bcs_path, layout_bcs) {
            warn!(
                "[vm2-delayed] failed to write dump file {}: {:?}",
                layout_bcs_path, err
            );
            return;
        }

        let layout_debug = format!("{:#?}\n", layout);
        if let Err(err) = std::fs::write(&layout_debug_path, layout_debug) {
            warn!(
                "[vm2-delayed] failed to write dump file {}: {:?}",
                layout_debug_path, err
            );
        }

        let context = format!(
            "scope={}\nstate_key={:?}\nstruct_tag={:?}\nvalue_len={}\nmetadata={:?}\n",
            scope,
            state_key,
            struct_tag,
            state_value.bytes().len(),
            state_value.clone().into_metadata()
        );
        if let Err(err) = std::fs::write(&context_path, context) {
            warn!(
                "[vm2-delayed] failed to write dump file {}: {:?}",
                context_path, err
            );
        }
    }

    pub fn materialize_output(&self, mut output: VMOutput) -> Result<TransactionOutput, VMStatus> {
        if !self.delayed_fields_enabled && output.contains_delayed_fields() {
            return Err(VMStatus::error(
                StatusCode::FEATURE_UNDER_GATING,
                Some("delayed fields feature disabled".to_string()),
            ));
        }

        let delayed_changes: Vec<_> = output
            .delayed_field_change_set()
            .iter()
            .map(|(id, change)| (*id, change.clone()))
            .collect();

        for (id, change) in &delayed_changes {
            let entry = change.clone().into_entry_no_additional_history();
            self.delayed_fields
                .record_change(*id, 0, entry)
                .map_err(|err| {
                    VMStatus::error(
                        StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                        Some(format!(
                            "Delayed field record failed for {:?}: {:?}",
                            id, err
                        )),
                    )
                })?;
        }

        if !delayed_changes.is_empty() {
            self.delayed_fields
                .try_commit(0, delayed_changes.iter().map(|(id, _)| *id))
                .map_err(|err| {
                    VMStatus::error(
                        StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                        Some(format!("Delayed field commit failed: {:?}", err)),
                    )
                })?;
        }

        let has_delayed = output.contains_delayed_fields();
        let has_agg_v1 = !output.aggregator_v1_delta_set().is_empty();
        if has_delayed || has_agg_v1 {
            output.try_materialize(self)?;
        }

        let mapping = DelayedFieldValueMapping {
            delayed_fields: &self.delayed_fields,
            txn_idx: 0,
        };
        let group_read_layouts = self.take_group_read_layouts();
        let mut group_cache: HashMap<StateKey, BTreeMap<StructTag, Bytes>> = HashMap::new();
        let patched_resource_write_set = materialize_resource_write_set(
            &output,
            &mapping,
            &self.delayed_field_cache,
            &group_read_layouts,
            self.executor_view,
            &mut group_cache,
            has_delayed,
            self.max_value_nest_depth,
        )?;
        let patched_events = if has_delayed {
            materialize_events(&output, &mapping, self.max_value_nest_depth)?
        } else {
            output
                .events()
                .iter()
                .map(|(event, _)| event.clone())
                .collect()
        };

        output
            .into_transaction_output_with_materialized_write_set(
                Vec::new(),
                patched_resource_write_set,
                patched_events,
            )
            .map_err(|err| {
                VMStatus::error(
                    StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
                    Some(err.to_string()),
                )
            })
    }
}

struct DelayedFieldValueMapping<'a> {
    delayed_fields: &'a VersionedDelayedFields<DelayedFieldID>,
    txn_idx: usize,
}

impl ValueToIdentifierMapping for DelayedFieldValueMapping<'_> {
    type Identifier = DelayedFieldID;

    fn value_to_identifier(
        &self,
        _kind: &move_core_types::value::IdentifierMappingKind,
        _layout: &MoveTypeLayout,
        _value: move_vm_types::values::Value,
    ) -> Result<Self::Identifier, PartialVMError> {
        Err(PartialVMError::new(
            StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR,
        ))
    }

    fn identifier_to_value(
        &self,
        layout: &MoveTypeLayout,
        identifier: Self::Identifier,
    ) -> Result<move_vm_types::values::Value, PartialVMError> {
        let value = self
            .delayed_fields
            .read_latest_predicted_value(&identifier, self.txn_idx, ReadPosition::AfterCurrentTxn)
            .map_err(|err| {
                PartialVMError::new(StatusCode::DELAYED_MATERIALIZATION_CODE_INVARIANT_ERROR)
                    .with_message(format!(
                        "Failed to read delayed field {:?} at txn {}: {:?}",
                        identifier, self.txn_idx, err
                    ))
            })?;
        value.try_into_move_value(layout, identifier.extract_width())
    }
}

impl<S: StateView> TDelayedFieldView for StorageAdapter<'_, S> {
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
        match self.delayed_fields.read(id, 0) {
            Ok(value) => Ok(value),
            Err(PanicOr::CodeInvariantError(err)) => Err(PanicOr::CodeInvariantError(err)),
            Err(PanicOr::Or(err)) => match err {
                MVDelayedFieldsError::NotFound => {
                    Err(PanicOr::Or(DelayedFieldsSpeculativeError::NotFound(*id)))
                }
                MVDelayedFieldsError::Dependency(_)
                | MVDelayedFieldsError::DeltaApplicationFailure => {
                    Err(PanicOr::Or(DelayedFieldsSpeculativeError::InconsistentRead))
                }
            },
        }
    }

    fn delayed_field_try_add_delta_outcome(
        &self,
        id: &Self::Identifier,
        base_delta: &SignedU128,
        delta: &SignedU128,
        max_value: u128,
    ) -> Result<bool, PanicOr<DelayedFieldsSpeculativeError>> {
        if delta.abs() > max_value {
            return Ok(false);
        }

        let base_value = self
            .get_delayed_field_value(id)?
            .into_aggregator_value()
            .map_err(PanicOr::from)?;
        let math = BoundedMath::new(max_value);
        let before = math
            .unsigned_add_delta(base_value, base_delta)
            .map_err(|e| PanicOr::from(code_invariant_error(e)))?;
        Ok(math.unsigned_add_delta(before, delta).is_ok())
    }

    fn generate_delayed_field_id(&self, width: u32) -> Self::Identifier {
        StorageAdapter::generate_delayed_field_id(self, width)
    }

    fn validate_delayed_field_id(&self, id: &Self::Identifier) -> Result<(), PanicError> {
        let unique_index = id.extract_unique_index();
        let current = self.delayed_field_id_counter.load(Ordering::SeqCst);
        if unique_index < self.delayed_field_id_start || unique_index >= current {
            return Err(code_invariant_error(format!(
                "Invalid delayed field id: {:?} with index: {} (started from {} and reached {})",
                id, unique_index, self.delayed_field_id_start, current
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
        PanicError,
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

impl<S: StateView> ModuleResolver for StorageAdapter<'_, S> {
    type Error = PartialVMError;

    fn get_module_metadata(&self, module_id: &ModuleId) -> Vec<Metadata> {
        info!("get_module_metadata | module_id: {:?}", module_id);

        let module = match self.get_module(module_id) {
            Ok(Some(module)) => module,
            _ => return vec![],
        };

        let compiled_module =
            match CompiledModule::deserialize_with_config(&module, &self.deserializer_config) {
                Ok(module) => module,
                _ => return vec![],
            };

        compiled_module.metadata
    }

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Bytes>, Self::Error> {
        // REVIEW: cache this?
        let key = StateKey::module_id(module_id);
        self.get(&key).map(|r| r.map(|v| v.bytes().clone()))
    }
}
impl<S: StateView> ResourceResolver for StorageAdapter<'_, S> {
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
            RESOURCE_GROUP_STATS
                .group_accesses
                .fetch_add(1, Ordering::Relaxed);
            let key = StateKey::resource_group(address, &resource_group);
            let buf = if let Some(layout) = maybe_layout {
                if self.delayed_fields_enabled {
                    if let Some(cached) = self
                        .delayed_field_cache
                        .get_group_member_value(&key, struct_tag)
                    {
                        RESOURCE_GROUP_STATS
                            .group_cache_hits
                            .fetch_add(1, Ordering::Relaxed);
                        Some(cached)
                    } else {
                        RESOURCE_GROUP_STATS
                            .group_member_calls
                            .fetch_add(1, Ordering::Relaxed);
                        let member_start = Instant::now();
                        let raw = self.resource_group_view.get_resource_from_group(
                            &key,
                            struct_tag,
                            maybe_layout,
                        )?;
                        RESOURCE_GROUP_STATS
                            .group_member_ns
                            .fetch_add(member_start.elapsed().as_nanos() as u64, Ordering::Relaxed);
                        let Some(raw_bytes) = raw else {
                            return Ok((None, 0));
                        };
                        let metadata = self
                            .executor_view
                            .get_state_value(&key)
                            .map_err(|e| {
                                PartialVMError::new(StatusCode::STORAGE_ERROR)
                                    .with_message(format!("{:?}", e))
                            })?
                            .map(|v| v.into_metadata())
                            .unwrap_or_else(StateValueMetadata::none);
                        let group_size = self.resource_group_view.resource_group_size(&key)?.get();
                        let state_value =
                            StateValue::new_with_metadata(raw_bytes.clone(), metadata);
                        #[cfg(debug_assertions)]
                        self.maybe_dump_exchange_input(
                            "group_fresh",
                            &key,
                            struct_tag,
                            &state_value,
                            layout,
                        );
                        let (exchanged, ids, _) = self
                            .maybe_exchange_state_value(&state_value, layout)
                            .map_err(|e| {
                                PartialVMError::new(StatusCode::STORAGE_ERROR)
                                    .with_message(format!("{:?}", e))
                            })?;
                        self.record_group_read(
                            &key,
                            exchanged.clone().into_metadata(),
                            group_size,
                            struct_tag.clone(),
                            layout,
                            ids,
                        );
                        let exchanged_bytes = exchanged.bytes().clone();
                        self.delayed_field_cache.insert_group_member_value(
                            key.clone(),
                            struct_tag.clone(),
                            exchanged_bytes.clone(),
                        );
                        Some(exchanged_bytes)
                    }
                } else {
                    RESOURCE_GROUP_STATS
                        .group_member_calls
                        .fetch_add(1, Ordering::Relaxed);
                    let member_start = Instant::now();
                    let raw = self.resource_group_view.get_resource_from_group(
                        &key,
                        struct_tag,
                        maybe_layout,
                    )?;
                    RESOURCE_GROUP_STATS
                        .group_member_ns
                        .fetch_add(member_start.elapsed().as_nanos() as u64, Ordering::Relaxed);
                    raw
                }
            } else {
                RESOURCE_GROUP_STATS
                    .group_member_calls
                    .fetch_add(1, Ordering::Relaxed);
                let member_start = Instant::now();
                let raw = self.resource_group_view.get_resource_from_group(
                    &key,
                    struct_tag,
                    maybe_layout,
                )?;
                RESOURCE_GROUP_STATS
                    .group_member_ns
                    .fetch_add(member_start.elapsed().as_nanos() as u64, Ordering::Relaxed);
                raw
            };

            let first_access = self.accessed_groups.borrow_mut().insert(key.clone());
            let group_size = if first_access {
                RESOURCE_GROUP_STATS
                    .group_size_calls
                    .fetch_add(1, Ordering::Relaxed);
                let size_start = Instant::now();
                let size = self.resource_group_view.resource_group_size(&key)?.get();
                RESOURCE_GROUP_STATS
                    .group_size_ns
                    .fetch_add(size_start.elapsed().as_nanos() as u64, Ordering::Relaxed);
                size
            } else {
                0
            };

            let buf_size = resource_size(&buf);
            Ok((buf, buf_size + group_size as usize))
        } else {
            let state_key = resource_state_key(address, struct_tag)?;
            let buf = if let Some(layout) = maybe_layout {
                if self.delayed_fields_enabled {
                    if let Some(cached) = self.delayed_field_cache.get_base_value(&state_key) {
                        if !self.delayed_field_cache.is_base_value_exchanged(&state_key) {
                            let state_value = cached.as_state_value().ok_or_else(|| {
                                PartialVMError::new(StatusCode::STORAGE_ERROR)
                                    .with_message("Cached base value missing bytes".to_string())
                            })?;
                            #[cfg(debug_assertions)]
                            self.maybe_dump_exchange_input(
                                "base_cached",
                                &state_key,
                                struct_tag,
                                &state_value,
                                layout,
                            );
                            let (exchanged, ids, exchanged_flag) = self
                                .maybe_exchange_state_value(&state_value, layout)
                                .map_err(|e| {
                                    PartialVMError::new(StatusCode::STORAGE_ERROR)
                                        .with_message(format!("{:?}", e))
                                })?;
                            self.record_resource_read(&state_key, &exchanged, layout, ids);
                            self.delayed_field_cache.insert_base_value(
                                state_key.clone(),
                                WriteOp::from_state_value(Some(exchanged.clone())),
                                exchanged_flag,
                            );
                            Some(exchanged.bytes().clone())
                        } else {
                            cached.bytes().cloned()
                        }
                    } else {
                        let state_value =
                            self.executor_view
                                .get_state_value(&state_key)
                                .map_err(|e| {
                                    PartialVMError::new(StatusCode::STORAGE_ERROR)
                                        .with_message(format!("{:?}", e))
                                })?;
                        let Some(state_value) = state_value else {
                            return Ok((None, 0));
                        };
                        #[cfg(debug_assertions)]
                        self.maybe_dump_exchange_input(
                            "base_fresh",
                            &state_key,
                            struct_tag,
                            &state_value,
                            layout,
                        );
                        let (exchanged, ids, exchanged_flag) = self
                            .maybe_exchange_state_value(&state_value, layout)
                            .map_err(|e| {
                                PartialVMError::new(StatusCode::STORAGE_ERROR)
                                    .with_message(format!("{:?}", e))
                            })?;
                        self.record_resource_read(&state_key, &exchanged, layout, ids);
                        let cached = self.delayed_field_cache.get_or_insert_base_value(
                            state_key.clone(),
                            exchanged_flag,
                            || Ok(WriteOp::from_state_value(Some(exchanged.clone()))),
                        )?;
                        cached.bytes().cloned()
                    }
                } else {
                    self.executor_view
                        .get_resource_bytes(&state_key, maybe_layout)?
                }
            } else {
                self.executor_view
                    .get_resource_bytes(&state_key, maybe_layout)?
            };
            let buf_size = resource_size(&buf);
            Ok((buf, buf_size))
        }
    }
}

impl<S: StateView> ResourceGroupResolver for StorageAdapter<'_, S> {
    fn release_resource_group_cache(
        &self,
    ) -> Option<HashMap<StateKey, BTreeMap<StructTag, Bytes>>> {
        self.resource_group_view.release_group_cache()
    }

    fn resource_group_size(&self, group_key: &StateKey) -> PartialVMResult<ResourceGroupSize> {
        self.resource_group_view.resource_group_size(group_key)
    }

    fn resource_size_in_group(
        &self,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> PartialVMResult<usize> {
        self.resource_group_view
            .resource_size_in_group(group_key, resource_tag)
    }

    fn resource_exists_in_group(
        &self,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> PartialVMResult<bool> {
        self.resource_group_view
            .resource_exists_in_group(group_key, resource_tag)
    }
}

impl<S> Deref for StorageAdapter<'_, S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.executor_view
    }
}

impl<S: StateView> TableResolver for StorageAdapter<'_, S> {
    // TODO(simon): don't ignore maybe_layout
    fn resolve_table_entry_bytes_with_layout(
        &self,
        handle: &TableHandle,
        key: &[u8],
        _maybe_layout: Option<&MoveTypeLayout>,
    ) -> Result<Option<Bytes>, PartialVMError> {
        self.executor_view
            .get_state_value(&StateKey::table_item(&(*handle).into(), key))
            .map(|r| r.map(|v| v.bytes().clone()))
            .map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR).with_message(format!("{:?}", e))
            })
    }
}
impl<S: StateView> CompiledModuleView for StorageAdapter<'_, S> {
    type Item = CompiledModule;
    fn view_compiled_module(&self, id: &ModuleId) -> anyhow::Result<Option<Self::Item>> {
        let module = match self.get_module(id) {
            Ok(Some(module)) => module,
            _ => return Ok(None),
        };
        Ok(Some(CompiledModule::deserialize(&module)?))
    }
}

pub trait AsMoveResolver<S> {
    fn as_move_resolver(&self) -> StorageAdapter<S>;
}

impl<S: StateView> AsMoveResolver<S> for S {
    fn as_move_resolver(&self) -> StorageAdapter<S> {
        let features = Features::fetch_config(self).unwrap_or_default();
        let deserializer_config = starcoin_prod_deserializer_config(&features);
        let delayed_fields_enabled = features.is_aggregator_v2_delayed_fields_enabled();
        let vm_config = VMConfig::fetch_config(self);
        let max_value_nest_depth = effective_max_value_nest_depth(self);
        let gas_feature_version = vm_config
            .as_ref()
            .map(|config| config.gas_schedule.feature_version)
            .unwrap_or(default_gas_schedule().feature_version);
        let resource_group_adapter = ResourceGroupAdapter::new(
            None,
            self,
            gas_feature_version,
            // todo(simon): Currently it is disabled, make it real.
            features.is_resource_groups_split_in_vm_change_set_enabled(),
            delayed_fields_enabled,
        );
        StorageAdapter::new(
            self,
            deserializer_config,
            max_value_nest_depth,
            resource_group_adapter,
            delayed_fields_enabled,
        )
    }
}

impl<S: StateView> AsExecutorView for StorageAdapter<'_, S> {
    fn as_executor_view(&self) -> &dyn ExecutorView {
        self
    }
}

pub struct RemoteStorageOwned<S> {
    state_view: S,
}

impl<S> Deref for RemoteStorageOwned<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.state_view
    }
}

impl<S> DerefMut for RemoteStorageOwned<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state_view
    }
}

impl<S: StateView> ModuleResolver for RemoteStorageOwned<S> {
    type Error = PartialVMError;

    fn get_module_metadata(&self, module_id: &ModuleId) -> Vec<Metadata> {
        self.as_move_resolver().get_module_metadata(module_id)
    }

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Bytes>, Self::Error> {
        self.as_move_resolver().get_module(module_id)
    }
}

impl<S: StateView> ResourceResolver for RemoteStorageOwned<S> {
    type Error = PartialVMError;

    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
        metadata: &[Metadata],
        layout: Option<&MoveTypeLayout>,
    ) -> Result<(Option<Bytes>, usize), Self::Error> {
        self.as_move_resolver()
            .get_resource_bytes_with_metadata_and_layout(address, struct_tag, metadata, layout)
    }
}

impl<S: StateView> ResourceGroupResolver for RemoteStorageOwned<S> {
    fn release_resource_group_cache(
        &self,
    ) -> Option<HashMap<StateKey, BTreeMap<StructTag, Bytes>>> {
        self.as_move_resolver().release_resource_group_cache()
    }

    fn resource_group_size(&self, group_key: &StateKey) -> PartialVMResult<ResourceGroupSize> {
        self.as_move_resolver().resource_group_size(group_key)
    }

    fn resource_size_in_group(
        &self,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> PartialVMResult<usize> {
        self.as_move_resolver()
            .resource_size_in_group(group_key, resource_tag)
    }

    fn resource_exists_in_group(
        &self,
        group_key: &StateKey,
        resource_tag: &StructTag,
    ) -> PartialVMResult<bool> {
        self.as_move_resolver()
            .resource_exists_in_group(group_key, resource_tag)
    }
}

impl<S: StateView> TableResolver for RemoteStorageOwned<S> {
    fn resolve_table_entry_bytes_with_layout(
        &self,
        handle: &TableHandle,
        key: &[u8],
        maybe_layout: Option<&MoveTypeLayout>,
    ) -> Result<Option<Bytes>, PartialVMError> {
        self.as_move_resolver()
            .resolve_table_entry_bytes_with_layout(handle, key, maybe_layout)
    }
}

impl<S: ExecutorView> AsExecutorView for RemoteStorageOwned<S> {
    fn as_executor_view(&self) -> &dyn ExecutorView {
        &self.state_view
    }
}

pub trait IntoMoveResolver<S> {
    fn into_move_resolver(self) -> RemoteStorageOwned<S>;
}

impl<S: StateView> IntoMoveResolver<S> for S {
    fn into_move_resolver(self) -> RemoteStorageOwned<S> {
        RemoteStorageOwned { state_view: self }
    }
}
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use starcoin_vm_runtime_types::resource_group_adapter::GroupSizeKind;
    //use starcoin_vm_types::on_chain_config::{Features, OnChainConfig};

    // Expose a method to create a storage adapter with a provided group size kind.
    #[allow(dead_code)]
    pub(crate) fn as_resolver_with_group_size_kind<S: StateView>(
        state_view: &S,
        group_size_kind: GroupSizeKind,
    ) -> StorageAdapter<S> {
        assert_ne!(group_size_kind, GroupSizeKind::AsSum, "not yet supported");

        let (gas_feature_version, resource_groups_split_in_vm_change_set_enabled) =
            match group_size_kind {
                GroupSizeKind::AsSum => (12, true),
                GroupSizeKind::AsBlob => (10, false),
                GroupSizeKind::None => (1, false),
            };

        let _group_adapter = ResourceGroupAdapter::new(
            // TODO[agg_v2](test) add a converter for StateView for tests that implements ResourceGroupView
            None,
            state_view,
            gas_feature_version,
            resource_groups_split_in_vm_change_set_enabled,
            false,
        );

        state_view.as_move_resolver()
    }
}
