// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::data_cache::get_resource_group_member_from_metadata;
use crate::default_gas_schedule;
use crate::move_vm_ext::{resource_state_key, AsExecutorView, ResourceGroupResolver};
use bytes::Bytes;
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::account_address::AccountAddress;
use move_core_types::language_storage::{ModuleId, StructTag};
use move_core_types::metadata::Metadata;
use move_core_types::resolver::{resource_size, ModuleResolver, ResourceResolver};
use move_core_types::value::MoveTypeLayout;
use move_core_types::vm_status::StatusCode;
use move_table_extension::{TableHandle, TableResolver};
use starcoin_parallel_executor::executor::MVHashMapView;
use starcoin_vm_runtime_types::resolver::{ExecutorView, ResourceGroupSize, TResourceGroupView};
use starcoin_vm_runtime_types::resource_group_adapter::{group_size_as_sum, GroupSizeKind};
use starcoin_vm_types::on_chain_config::{OnChainConfig, VMConfig};
use starcoin_vm_types::state_store::{
    errors::StateviewError, state_key::StateKey, state_storage_usage::StateStorageUsage,
    state_value::StateValue, StateView, TStateView,
};
use starcoin_vm_types::write_set::WriteOp;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ResourceGroupCacheEntry {
    data: BTreeMap<StructTag, Bytes>,
    size: ResourceGroupSize,
}

pub(crate) struct VersionedView<'a, S: StateView> {
    base_view: &'a S,
    hashmap_view: &'a MVHashMapView<'a, StateKey, WriteOp, ResourceGroupCacheEntry>,
    group_size_kind: GroupSizeKind,
    accessed_groups: RefCell<HashSet<StateKey>>,
    group_sizes: RefCell<HashMap<StateKey, ResourceGroupSize>>,
}

impl<'a, S: StateView> VersionedView<'a, S> {
    pub fn new_view(
        base_view: &'a S,
        hashmap_view: &'a MVHashMapView<'a, StateKey, WriteOp, ResourceGroupCacheEntry>,
    ) -> VersionedView<'a, S> {
        let gas_feature_version = VMConfig::fetch_config(base_view)
            .map(|config| config.gas_schedule)
            .unwrap_or(default_gas_schedule())
            .feature_version;
        let group_size_kind = GroupSizeKind::from_gas_feature_version(gas_feature_version, false);
        VersionedView {
            base_view,
            hashmap_view,
            group_size_kind,
            accessed_groups: RefCell::new(HashSet::new()),
            group_sizes: RefCell::new(HashMap::new()),
        }
    }

    fn empty_group_size(&self) -> ResourceGroupSize {
        match self.group_size_kind {
            GroupSizeKind::AsSum => ResourceGroupSize::zero_combined(),
            GroupSizeKind::None | GroupSizeKind::AsBlob => ResourceGroupSize::zero_concrete(),
        }
    }

    fn parse_group_entry(
        &self,
        group_key: &StateKey,
        bytes: &Bytes,
    ) -> PartialVMResult<ResourceGroupCacheEntry> {
        let data: BTreeMap<StructTag, Bytes> = bcs_ext::from_bytes(bytes).map_err(|e| {
            PartialVMError::new(StatusCode::UNEXPECTED_DESERIALIZATION_ERROR).with_message(format!(
                "Failed to deserialize the resource group at {:?}: {:?}",
                group_key, e
            ))
        })?;
        let size = match self.group_size_kind {
            GroupSizeKind::None => ResourceGroupSize::Concrete(0),
            GroupSizeKind::AsBlob => ResourceGroupSize::Concrete(bytes.len() as u64),
            GroupSizeKind::AsSum => {
                group_size_as_sum(data.iter().map(|(tag, value)| (tag, value.len())))?
            }
        };
        Ok(ResourceGroupCacheEntry { data, size })
    }

    fn load_group_entry(
        &self,
        group_key: &StateKey,
    ) -> PartialVMResult<Option<Arc<ResourceGroupCacheEntry>>> {
        if let Some((version, write_op)) = self.hashmap_view.read_with_version(group_key) {
            if let Some(bytes) = write_op.bytes() {
                if let Some((cached_version, cached_entry)) =
                    self.hashmap_view.read_group_data(group_key)
                {
                    if cached_version == version {
                        return Ok(Some(cached_entry));
                    }
                }
                let entry = self.parse_group_entry(group_key, bytes)?;
                return Ok(Some(
                    self.hashmap_view
                        .write_group_data(group_key, version, entry),
                ));
            }
            return Ok(None);
        }

        if let Some(entry) = self.hashmap_view.read_group_base_data(group_key) {
            return Ok(Some(entry));
        }

        let entry = match self.base_view.get_state_value(group_key)? {
            Some(state_value) => self.parse_group_entry(group_key, state_value.bytes())?,
            None => ResourceGroupCacheEntry {
                data: BTreeMap::new(),
                size: self.empty_group_size(),
            },
        };

        Ok(Some(
            self.hashmap_view
                .write_group_base_data_if_absent(group_key, entry),
        ))
    }
}

impl<S: StateView> TStateView for VersionedView<'_, S> {
    type Key = StateKey;

    // Get some data either through the cache or the `StateView` on a cache miss.
    fn get_state_value(&self, state_key: &Self::Key) -> Result<Option<StateValue>, StateviewError> {
        match self.hashmap_view.read(state_key) {
            Some(v) => Ok(v
                .bytes()
                .map(|bytes| StateValue::new_with_metadata(bytes.clone(), v.metadata().clone()))),
            None => self.base_view.get_state_value(state_key),
        }
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

    fn resource_group_size(&self, _group_key: &StateKey) -> PartialVMResult<ResourceGroupSize> {
        unimplemented!("Currently resolved by ResourceGroupAdapter");
    }

    fn get_resource_from_group(
        &self,
        group_key: &Self::GroupKey,
        resource_tag: &Self::ResourceTag,
    ) -> PartialVMResult<Option<Bytes>> {
        let entry = self.load_group_entry(group_key)?;
        let group_size = entry
            .as_ref()
            .map(|entry| entry.size)
            .unwrap_or_else(|| self.empty_group_size());
        self.group_sizes
            .borrow_mut()
            .insert(group_key.clone(), group_size);
        Ok(entry.and_then(|entry| entry.data.get(resource_tag).cloned()))
    }

    fn resource_size_in_group(
        &self,
        _group_key: &StateKey,
        _resource_tag: &StructTag,
    ) -> PartialVMResult<usize> {
        unimplemented!("Currently resolved by ResourceGroupAdapter");
    }

    fn resource_exists_in_group(
        &self,
        _group_key: &StateKey,
        _resource_tag: &StructTag,
    ) -> PartialVMResult<bool> {
        unimplemented!("Currently resolved by ResourceGroupAdapter");
    }

    fn release_group_cache(
        &self,
    ) -> Option<HashMap<Self::GroupKey, BTreeMap<Self::ResourceTag, Bytes>>> {
        unimplemented!("Currently resolved by ResourceGroupAdapter");
    }
}

impl<S: StateView> AsExecutorView for VersionedView<'_, S> {
    fn as_executor_view(&self) -> &dyn ExecutorView {
        todo!()
    }
}

impl<S: StateView> TableResolver for VersionedView<'_, S> {
    fn resolve_table_entry_bytes_with_layout(
        &self,
        _handle: &TableHandle,
        _key: &[u8],
        _maybe_layout: Option<&MoveTypeLayout>,
    ) -> Result<Option<Bytes>, PartialVMError> {
        todo!()
    }
}

impl<S: StateView> ModuleResolver for VersionedView<'_, S> {
    type Error = PartialVMError;

    fn get_module_metadata(&self, _module_id: &ModuleId) -> Vec<Metadata> {
        todo!()
    }

    fn get_module(&self, _id: &ModuleId) -> Result<Option<Bytes>, Self::Error> {
        todo!()
    }
}

impl<S: StateView> ResourceResolver for VersionedView<'_, S> {
    type Error = PartialVMError;

    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        struct_tag: &StructTag,
        metadata: &[Metadata],
        _layout: Option<&MoveTypeLayout>,
    ) -> Result<(Option<Bytes>, usize), Self::Error> {
        assert!(
            _layout.is_none(),
            "Layout has not been supported, must be empty"
        );
        let resource_group = get_resource_group_member_from_metadata(struct_tag, metadata);
        if let Some(resource_group) = resource_group {
            let group_key = StateKey::resource_group(address, &resource_group);
            let buf = self.get_resource_from_group(&group_key, struct_tag)?;

            let first_access = self.accessed_groups.borrow_mut().insert(group_key.clone());
            let group_size = if first_access {
                self.group_sizes
                    .borrow()
                    .get(&group_key)
                    .map(|size| size.get())
                    .unwrap_or(0)
            } else {
                0
            };

            let buf_size = resource_size(&buf);
            Ok((buf, buf_size + group_size as usize))
        } else {
            let state_key = resource_state_key(address, struct_tag)?;
            let buf = self
                .get_state_value(&state_key)?
                .map(|state_value| state_value.bytes().clone());
            let buf_size = resource_size(&buf);
            Ok((buf, buf_size))
        }
    }
}

// todo(simon): Is it necessary?
impl<S: StateView> ResourceGroupResolver for VersionedView<'_, S> {
    fn release_resource_group_cache(
        &self,
    ) -> Option<HashMap<StateKey, BTreeMap<StructTag, Bytes>>> {
        todo!()
    }

    fn resource_group_size(&self, _group_key: &StateKey) -> PartialVMResult<ResourceGroupSize> {
        todo!()
    }

    fn resource_size_in_group(
        &self,
        _group_key: &StateKey,
        _resource_tag: &StructTag,
    ) -> PartialVMResult<usize> {
        todo!()
    }

    fn resource_exists_in_group(
        &self,
        _group_key: &StateKey,
        _resource_tag: &StructTag,
    ) -> PartialVMResult<bool> {
        todo!()
    }
}
