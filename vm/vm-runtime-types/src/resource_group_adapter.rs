// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::resolver::{
    size_u32_as_uleb128, ResourceGroupSize, ResourceGroupView, TResourceGroupView, TResourceView,
};
use anyhow::Error;
use bytes::Bytes;
use move_core_types::{language_storage::StructTag, value::MoveTypeLayout};
use serde::Serialize;
use starcoin_vm_types::state_store::state_key::StateKey;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    fmt::Debug,
};

/// Corresponding to different gas features, methods for counting the 'size' of a
/// resource group. None leads to 0, while AsBlob provides the group size as the
/// size of the serialized blob of the BTreeMap corresponding to the group.
/// For AsSum, the size is summed for each resource contained in the group (of
/// the resource blob, and its corresponding tag, when serialized)
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum GroupSizeKind {
    None,
    AsBlob,
    AsSum,
}

impl GroupSizeKind {
    pub fn from_gas_feature_version(
        gas_feature_version: u64,
        resource_group_charge_as_size_sum_enabled: bool,
    ) -> Self {
        if resource_group_charge_as_size_sum_enabled {
            GroupSizeKind::AsSum
        } else if gas_feature_version >= 9 {
            // Keep old caching behavior for replay.
            GroupSizeKind::AsBlob
        } else {
            GroupSizeKind::None
        }
    }
}

pub fn group_tagged_resource_size<T: Serialize + Clone + Debug>(
    tag: &T,
    value_byte_len: usize,
) -> anyhow::Result<u64> {
    Ok((bcs::serialized_size(&tag)? + value_byte_len + size_u32_as_uleb128(value_byte_len)) as u64)
}

/// Utility method to compute the size of the group as GroupSizeKind::AsSum.
pub fn group_size_as_sum<T: Serialize + Clone + Debug>(
    mut group: impl Iterator<Item = (T, usize)>,
) -> anyhow::Result<ResourceGroupSize> {
    let (count, len) = group
        .try_fold((0, 0), |(count, len), (tag, value_byte_len)| {
            let delta = group_tagged_resource_size(&tag, value_byte_len)?;
            Ok((count + 1, len + delta))
        })
        .map_err(|_: Error| anyhow::Error::msg("Resource group member tag serialization error"))?;

    Ok(ResourceGroupSize::Combined {
        num_tagged_resources: count,
        all_tagged_resources_size: len,
    })
}

#[test]
fn test_group_size_same_as_bcs() {
    use aptos_types::PeerId;
    use move_core_types::identifier::Identifier;

    let reused_vec = Bytes::from(vec![5; 20000]);

    for i in [1, 2, 3, 5, 15, 100, 1000, 10000, 20000] {
        let mut map = BTreeMap::new();

        for j in 0..i {
            map.insert(
                StructTag {
                    address: PeerId::ONE,
                    module: Identifier::new("a").unwrap(),
                    name: Identifier::new(format!("a_{}", j)).unwrap(),
                    type_params: vec![],
                },
                reused_vec.slice(0..j),
            );
        }

        assert_eq!(
            bcs::serialized_size(&map).unwrap() as u64,
            group_size_as_sum(map.into_iter().map(|(k, v)| (k, v.len())))
                .unwrap()
                .get()
        );
    }
}

/// Handles the resolution of ResourceGroupView interfaces. If the gas feature version is
/// sufficiently new (corresponding to GroupSizeKind::AsSum), maybe_resource_group_view will
/// be used first, if set (this way, block executor provides the new resolution behavior).
///
/// If gas feature corresponding to AsSum is not enabled, maybe_resource_group_view is set
/// to None in any case, as block executor does not support older gas charging behavior.
/// When maybe_resource_group_view is None, group view resolution happens based on the
/// resource view interfaces, with an underlying cache. The cache is for efficiency, but
/// also released to the session for older feature versions (needed to prepare VM output).
pub struct ResourceGroupAdapter<'r> {
    maybe_resource_group_view: Option<&'r dyn ResourceGroupView>,
    resource_view: &'r dyn TResourceView<Key = StateKey, Layout = MoveTypeLayout>,
    group_size_kind: GroupSizeKind,
    group_cache: RefCell<HashMap<StateKey, (BTreeMap<StructTag, Bytes>, ResourceGroupSize)>>,
}

impl<'r> ResourceGroupAdapter<'r> {
    pub fn new(
        maybe_resource_group_view: Option<&'r dyn ResourceGroupView>,
        resource_view: &'r dyn TResourceView<Key = StateKey, Layout = MoveTypeLayout>,
        gas_feature_version: u64,
        resource_group_charge_as_size_sum_enabled: bool,
    ) -> Self {
        // TODO[agg_v2](fix) - when is_resource_group_split_in_change_set_capable is false,
        // but resource_group_charge_as_size_sum_enabled is true, we still don't set
        // group_size_kind to GroupSizeKind::AsSum, meaning that
        // is_resource_group_split_in_change_set_capable affects gas charging. make sure that is correct

        let group_size_kind = GroupSizeKind::from_gas_feature_version(
            gas_feature_version,
            // Even if flag is enabled, if we are in non-capable context, we cannot use AsSum,
            // and split resource groups in the VMChangeSet.
            // We are not capable if:
            // - Block contains single PayloadWriteSet::Direct transaction
            // - we are not executing blocks for a live network in a gas charging context
            //     (outside of BlockExecutor) i.e. unit tests, view functions, etc.
            //     In this case, disabled will lead to a different gas behavior,
            //     but gas is not relevant for those contexts.
            resource_group_charge_as_size_sum_enabled
                && maybe_resource_group_view
                    .map_or(false, |v| v.is_resource_group_split_in_change_set_capable()),
        );

        Self {
            maybe_resource_group_view: maybe_resource_group_view
                .filter(|_| group_size_kind == GroupSizeKind::AsSum),
            resource_view,
            group_size_kind,
            group_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn group_size_kind(&self) -> GroupSizeKind {
        self.group_size_kind.clone()
    }

    // Ensures that the resource group at state_key is cached in self.group_cache. Ok(true)
    // means the resource was already cached, while Ok(false) means it just got cached.
    fn load_to_cache(&self, group_key: &StateKey) -> anyhow::Result<bool> {
        let already_cached = self.group_cache.borrow().contains_key(group_key);
        if already_cached {
            return Ok(true);
        }

        let group_data = self.resource_view.get_resource_bytes(group_key, None)?;
        let (group_data, blob_len): (BTreeMap<StructTag, Bytes>, u64) = group_data.map_or_else(
            || Ok::<_, Error>((BTreeMap::new(), 0)),
            |group_data_blob| {
                let group_data = bcs::from_bytes(&group_data_blob)
                    .map_err(|_| anyhow::Error::msg("Resource group deserialization error"))?;
                Ok((group_data, group_data_blob.len() as u64))
            },
        )?;

        let group_size = match self.group_size_kind {
            GroupSizeKind::None => ResourceGroupSize::Concrete(0),
            GroupSizeKind::AsBlob => ResourceGroupSize::Concrete(blob_len),
            GroupSizeKind::AsSum => {
                group_size_as_sum(group_data.iter().map(|(t, v)| (t, v.len())))?
            }
        };
        self.group_cache
            .borrow_mut()
            .insert(group_key.clone(), (group_data, group_size));
        Ok(false)
    }
}

// TODO: Once R-before-W semantics is relaxed in the Move-VM, implement by forwarding
// to maybe_resource_group_view resource_size_in_group and resource_exists_in_group APIs
// (and provide corresponding implementation in the Block Executor).
impl TResourceGroupView for ResourceGroupAdapter<'_> {
    type GroupKey = StateKey;
    type Layout = MoveTypeLayout;
    type ResourceTag = StructTag;

    fn is_resource_group_split_in_change_set_capable(&self) -> bool {
        self.group_size_kind == GroupSizeKind::AsSum
    }

    fn resource_group_size(&self, group_key: &Self::GroupKey) -> anyhow::Result<ResourceGroupSize> {
        if self.group_size_kind == GroupSizeKind::None {
            return Ok(ResourceGroupSize::zero_concrete());
        }

        if let Some(group_view) = self.maybe_resource_group_view {
            return group_view.resource_group_size(group_key);
        }

        self.load_to_cache(group_key)?;
        Ok(self
            .group_cache
            .borrow()
            .get(group_key)
            .expect("Must be cached")
            .1)
    }

    fn get_resource_from_group(
        &self,
        group_key: &Self::GroupKey,
        resource_tag: &Self::ResourceTag,
        maybe_layout: Option<&MoveTypeLayout>,
    ) -> anyhow::Result<Option<Bytes>> {
        if let Some(group_view) = self.maybe_resource_group_view {
            return group_view.get_resource_from_group(group_key, resource_tag, maybe_layout);
        }
        self.load_to_cache(group_key)?;
        Ok(self
            .group_cache
            .borrow()
            .get(group_key)
            .expect("Must be cached")
            .0 // btreemap
            .get(resource_tag)
            .cloned())
    }

    fn release_group_cache(
        &self,
    ) -> Option<HashMap<Self::GroupKey, BTreeMap<Self::ResourceTag, Bytes>>> {
        if self.group_size_kind == GroupSizeKind::AsSum {
            // Clear the cache, but do not return the contents to the caller. This leads to
            // the VMChangeSet prepared in a new, granular format that the block executor
            // can handle (combined as a group update at the end).
            self.group_cache.borrow_mut().clear();
            None
        } else {
            // Returning the contents to the caller leads to preparing the VMChangeSet in the
            // backwards compatible way (containing the whole group update).
            Some(
                self.group_cache
                    .borrow_mut()
                    .drain()
                    .map(|(k, v)| (k, v.0))
                    .collect(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::utils::{mock_tag_0, mock_tag_1, mock_tag_2};
    use aptos_types::state_store::{
        state_storage_usage::StateStorageUsage, state_value::StateValue, TStateView,
    };
    use claims::{assert_gt, assert_none, assert_ok_eq, assert_some, assert_some_eq};
    use std::cmp::max;
    use test_case::test_case;

    struct MockGroup {
        blob: Vec<u8>,
        contents: BTreeMap<StructTag, Vec<u8>>,
        size_as_sum: ResourceGroupSize,
    }

    impl MockGroup {
        fn new(contents: BTreeMap<StructTag, Vec<u8>>) -> Self {
            let size_as_sum =
                group_size_as_sum(contents.iter().map(|(tag, v)| (tag, v.len()))).unwrap();
            let blob = bcs::to_bytes(&contents).unwrap();
            assert_eq!(
                size_as_sum.get(),
                if contents.is_empty() {
                    0
                } else {
                    blob.len() as u64
                }
            );

            Self {
                blob,
                contents,
                size_as_sum,
            }
        }
    }

    struct MockStateView {
        group: HashMap<StateKey, MockGroup>,
    }

    impl MockStateView {
        fn new() -> Self {
            let mut group = HashMap::new();

            let key_0 = StateKey::raw(vec![0]);
            let key_1 = StateKey::raw(vec![1]);

            // for testing purposes, o.w. state view should never contain an empty map.
            group.insert(key_0, MockGroup::new(BTreeMap::new()));
            group.insert(
                key_1,
                MockGroup::new(BTreeMap::from([
                    (mock_tag_0(), vec![0; 1000]),
                    (mock_tag_1(), vec![1; 500]),
                ])),
            );

            Self { group }
        }
    }

    impl TStateView for MockStateView {
        type Key = StateKey;

        fn get_state_value(&self, state_key: &Self::Key) -> anyhow::Result<Option<StateValue>> {
            Ok(self
                .group
                .get(state_key)
                .map(|entry| StateValue::new_legacy(entry.blob.clone().into())))
        }

        fn get_usage(&self) -> anyhow::Result<StateStorageUsage> {
            unimplemented!();
        }
    }

    impl TResourceGroupView for MockStateView {
        type GroupKey = StateKey;
        type Layout = MoveTypeLayout;
        type ResourceTag = StructTag;

        fn is_resource_group_split_in_change_set_capable(&self) -> bool {
            true
        }

        fn resource_group_size(
            &self,
            group_key: &Self::GroupKey,
        ) -> anyhow::Result<ResourceGroupSize> {
            Ok(self
                .group
                .get(group_key)
                .map(|entry| entry.size_as_sum)
                .unwrap_or(ResourceGroupSize::zero_combined()))
        }

        fn get_resource_from_group(
            &self,
            group_key: &Self::GroupKey,
            resource_tag: &Self::ResourceTag,
            _maybe_layout: Option<&Self::Layout>,
        ) -> anyhow::Result<Option<Bytes>> {
            Ok(self
                .group
                .get(group_key)
                .and_then(|entry| entry.contents.get(resource_tag).cloned().map(Into::into)))
        }

        fn resource_size_in_group(
            &self,
            _group_key: &Self::GroupKey,
            _resource_tag: &Self::ResourceTag,
        ) -> anyhow::Result<usize> {
            unimplemented!("Currently resolved by ResourceGroupAdapter");
        }

        fn resource_exists_in_group(
            &self,
            _group_key: &Self::GroupKey,
            _resource_tag: &Self::ResourceTag,
        ) -> anyhow::Result<bool> {
            unimplemented!("Currently resolved by ResourceGroupAdapter");
        }

        fn release_group_cache(
            &self,
        ) -> Option<HashMap<Self::GroupKey, BTreeMap<Self::ResourceTag, Bytes>>> {
            unimplemented!("Currently resolved by ResourceGroupAdapter");
        }
    }

    #[test]
    fn group_size_kind_from_gas_version() {
        for i in 0..9 {
            assert_eq!(
                GroupSizeKind::from_gas_feature_version(i, true),
                GroupSizeKind::AsSum
            );
            assert_eq!(
                GroupSizeKind::from_gas_feature_version(i, false),
                GroupSizeKind::None
            );
        }

        for i in 9..20 {
            assert_eq!(
                GroupSizeKind::from_gas_feature_version(i, true),
                GroupSizeKind::AsSum
            );
            assert_eq!(
                GroupSizeKind::from_gas_feature_version(i, false),
                GroupSizeKind::AsBlob
            );
        }
    }

    #[test]
    fn load_to_cache() {
        let state_view = MockStateView::new();
        let adapter = ResourceGroupAdapter::new(None, &state_view, 3, false);
        assert_eq!(adapter.group_size_kind, GroupSizeKind::None);

        let key_1 = StateKey::raw(vec![1]);
        let tag_0 = mock_tag_0();

        assert_ok_eq!(adapter.load_to_cache(&key_1), false);
        let _ = adapter.get_resource_from_group(&key_1, &tag_0, None);
        assert_ok_eq!(adapter.load_to_cache(&key_1), true);
    }

    #[test]
    fn test_get_resource_by_tag() {
        let state_view = MockStateView::new();
        let adapter = ResourceGroupAdapter::new(None, &state_view, 5, false);
        assert_eq!(adapter.group_size_kind, GroupSizeKind::None);

        let key_0 = StateKey::raw(vec![0]);
        let key_1 = StateKey::raw(vec![1]);
        let key_2 = StateKey::raw(vec![2]);
        let tag_0 = mock_tag_0();
        let tag_1 = mock_tag_1();
        let tag_2 = mock_tag_2();

        // key_0 / tag_0 does not exist.
        assert_none!(adapter
            .get_resource_from_group(&key_0, &tag_0, None)
            .unwrap());

        assert_some_eq!(
            adapter
                .get_resource_from_group(&key_1, &tag_0, None)
                .unwrap(),
            vec![0; 1000]
        );

        // key_2 / tag_1 does not exist.
        assert_none!(adapter
            .get_resource_from_group(&key_2, &tag_1, None)
            .unwrap());

        let key_1_blob = &state_view.group.get(&key_1).unwrap().blob;

        // Release the cache to test contents.
        let cache = adapter.release_group_cache().unwrap();
        assert_eq!(cache.len(), 3);
        assert!(cache.get(&key_0).expect("Must be Some(..)").is_empty());
        assert!(cache.get(&key_2).expect("Must be Some(..)").is_empty());
        let cache_key_1_contents = cache.get(&key_1).unwrap();
        assert_eq!(bcs::to_bytes(&cache_key_1_contents).unwrap(), *key_1_blob);

        assert_some_eq!(
            adapter
                .get_resource_from_group(&key_1, &tag_1, None)
                .unwrap(),
            vec![1; 500]
        );

        assert_none!(adapter
            .get_resource_from_group(&key_1, &tag_2, None)
            .unwrap());

        let cache = adapter.release_group_cache().unwrap();
        assert_eq!(cache.len(), 1);
        let cache_key_1_contents = cache.get(&key_1).unwrap();
        assert_eq!(bcs::to_bytes(&cache_key_1_contents).unwrap(), *key_1_blob);
    }

    #[test_case(9, false)]
    #[test_case(12, true)] // Without view, this falls back to as_blob
    fn size_as_blob_len(gas_feature_version: u64, resource_group_charge_as_size_sum_enabled: bool) {
        let state_view = MockStateView::new();
        let adapter = ResourceGroupAdapter::new(
            None,
            &state_view,
            gas_feature_version,
            resource_group_charge_as_size_sum_enabled,
        );
        assert_eq!(adapter.group_size_kind, GroupSizeKind::AsBlob);

        let key_0 = StateKey::raw(vec![0]);
        let key_1 = StateKey::raw(vec![1]);
        let key_2 = StateKey::raw(vec![2]);

        let key_0_blob_len =
            ResourceGroupSize::Concrete(state_view.group.get(&key_0).unwrap().blob.len() as u64);
        let key_1_blob_len =
            ResourceGroupSize::Concrete(state_view.group.get(&key_1).unwrap().blob.len() as u64);

        assert_ok_eq!(adapter.resource_group_size(&key_1), key_1_blob_len);

        // Release the cache via trait method and test contents.
        let cache = adapter.release_group_cache().unwrap();
        assert_eq!(cache.len(), 1);
        assert_some!(cache.get(&key_1));

        assert_ok_eq!(adapter.resource_group_size(&key_0), key_0_blob_len);
        assert_ok_eq!(adapter.resource_group_size(&key_1), key_1_blob_len);
        assert_ok_eq!(
            adapter.resource_group_size(&key_2),
            ResourceGroupSize::Concrete(0)
        );

        let cache = adapter.release_group_cache().unwrap();
        assert_eq!(cache.len(), 3);
        assert_some!(cache.get(&key_0));
        assert_some!(cache.get(&key_1));
        assert_some!(cache.get(&key_2));
    }

    #[test]
    fn set_group_view_forwarding() {
        let state_view = MockStateView::new();
        let adapter = ResourceGroupAdapter::new(Some(&state_view), &state_view, 12, true);
        assert_some!(adapter.maybe_resource_group_view);
        let adapter_with_forwarding =
            ResourceGroupAdapter::new(Some(&adapter), &state_view, 12, true);
        assert_some!(adapter_with_forwarding.maybe_resource_group_view);
    }

    #[test]
    fn size_as_sum() {
        let state_view = MockStateView::new();
        let adapter = ResourceGroupAdapter::new(Some(&state_view), &state_view, 12, true);
        assert_eq!(adapter.group_size_kind, GroupSizeKind::AsSum);

        let key_0 = StateKey::raw(vec![0]);
        let key_1 = StateKey::raw(vec![1]);
        let key_2 = StateKey::raw(vec![2]);

        let key_0_size_as_sum = state_view.group.get(&key_0).unwrap().size_as_sum;
        let key_1_size_as_sum = state_view.group.get(&key_1).unwrap().size_as_sum;

        assert_ok_eq!(adapter.resource_group_size(&key_1), key_1_size_as_sum);

        assert_eq!(adapter.group_cache.borrow().len(), 0);

        assert_ok_eq!(adapter.resource_group_size(&key_0), key_0_size_as_sum);
        assert_ok_eq!(adapter.resource_group_size(&key_1), key_1_size_as_sum);
        assert_ok_eq!(
            adapter.resource_group_size(&key_2),
            ResourceGroupSize::zero_combined()
        );

        assert_eq!(adapter.group_cache.borrow().len(), 0);

        // Sanity check the size numbers, at the time of writing the test 1587.
        let key_1_blob_size = state_view.group.get(&key_1).unwrap().blob.len() as u64;
        assert_eq!(
            key_1_size_as_sum.get(),
            key_1_blob_size,
            "size as sum must be equal to BTreeMap blob size",
        );
        assert_gt!(
            key_1_size_as_sum.get(),
            max(1000, key_1_blob_size - 100),
            "size as sum may not be too small"
        );
    }

    #[test]
    fn size_as_none() {
        let state_view = MockStateView::new();
        let adapter = ResourceGroupAdapter::new(None, &state_view, 8, false);
        assert_eq!(adapter.group_size_kind, GroupSizeKind::None);

        let key_0 = StateKey::raw(vec![0]);
        let key_1 = StateKey::raw(vec![1]);
        let key_2 = StateKey::raw(vec![2]);

        assert_ok_eq!(
            adapter.resource_group_size(&key_1),
            ResourceGroupSize::Concrete(0)
        );
        // Test releasing the cache via trait method.
        let cache = adapter.release_group_cache().unwrap();
        // GroupSizeKind::None does not cache on size queries.
        assert_eq!(cache.len(), 0);

        assert_ok_eq!(
            adapter.resource_group_size(&key_0),
            ResourceGroupSize::Concrete(0)
        );
        assert_ok_eq!(
            adapter.resource_group_size(&key_1),
            ResourceGroupSize::Concrete(0)
        );
        assert_ok_eq!(
            adapter.resource_group_size(&key_2),
            ResourceGroupSize::Concrete(0)
        );

        let cache = adapter.release_group_cache().unwrap();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn exists_resource_in_group() {
        let state_view = MockStateView::new();
        let adapter = ResourceGroupAdapter::new(None, &state_view, 0, false);
        assert_eq!(adapter.group_size_kind, GroupSizeKind::None);

        let key_0 = StateKey::raw(vec![0]);
        let key_1 = StateKey::raw(vec![1]);
        let key_2 = StateKey::raw(vec![2]);
        let tag_0 = mock_tag_0();
        let tag_1 = mock_tag_1();
        let tag_2 = mock_tag_2();

        assert_ok_eq!(adapter.resource_exists_in_group(&key_1, &tag_0), true);
        assert_ok_eq!(adapter.resource_exists_in_group(&key_1, &tag_1), true);
        assert_ok_eq!(adapter.resource_exists_in_group(&key_2, &tag_2), false);
        // Release the cache to test contents.
        let cache = adapter.release_group_cache().unwrap();
        assert_eq!(cache.len(), 2);
        assert_some!(cache.get(&key_1));
        assert_some!(cache.get(&key_2));

        assert_ok_eq!(adapter.resource_exists_in_group(&key_0, &tag_1), false);
        assert_ok_eq!(adapter.resource_exists_in_group(&key_1, &tag_2), false);

        let cache = adapter.release_group_cache().unwrap();
        assert_eq!(cache.len(), 2);
        assert_some!(cache.get(&key_0));
        assert_some!(cache.get(&key_1));
    }

    #[test]
    fn resource_size_in_group() {
        let state_view = MockStateView::new();
        let adapter = ResourceGroupAdapter::new(None, &state_view, 3, false);
        assert_eq!(adapter.group_size_kind, GroupSizeKind::None);

        let key_0 = StateKey::raw(vec![0]);
        let key_1 = StateKey::raw(vec![1]);
        let key_2 = StateKey::raw(vec![2]);
        let tag_0 = mock_tag_0();
        let tag_1 = mock_tag_1();
        let tag_2 = mock_tag_2();

        let key_1_tag_0_len = adapter
            .get_resource_from_group(&key_1, &tag_0, None)
            .unwrap()
            .unwrap()
            .len();
        let key_1_tag_1_len = adapter
            .get_resource_from_group(&key_1, &tag_1, None)
            .unwrap()
            .unwrap()
            .len();

        assert_ok_eq!(
            adapter.resource_size_in_group(&key_1, &tag_0),
            key_1_tag_0_len
        );
        assert_ok_eq!(
            adapter.resource_size_in_group(&key_1, &tag_1),
            key_1_tag_1_len
        );
        assert_ok_eq!(adapter.resource_size_in_group(&key_2, &tag_2), 0);
        // Release the cache to test contents.
        let cache = adapter.release_group_cache().unwrap();
        assert_eq!(cache.len(), 2);
        assert_some!(cache.get(&key_1));
        assert_some!(cache.get(&key_2));

        assert_ok_eq!(adapter.resource_size_in_group(&key_0, &tag_1), 0);
        assert_ok_eq!(adapter.resource_size_in_group(&key_1, &tag_2), 0);

        let cache = adapter.release_group_cache().unwrap();
        assert_eq!(cache.len(), 2);
        assert_some!(cache.get(&key_0));
        assert_some!(cache.get(&key_1));
    }
}
