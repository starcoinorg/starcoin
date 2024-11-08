// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::data_cache::{IntoMoveResolver, RemoteStorageOwned};
use starcoin_parallel_executor::executor::MVHashMapView;
use starcoin_vm_types::state_store::{
    errors::StateviewError, state_key::StateKey, state_storage_usage::StateStorageUsage,
    state_value::StateValue, StateView, TStateView,
};
use starcoin_vm_types::write_set::WriteOp;

pub(crate) struct VersionedView<'a, S: StateView> {
    base_view: &'a S,
    hashmap_view: &'a MVHashMapView<'a, StateKey, WriteOp>,
}

impl<'a, S: StateView> VersionedView<'a, S> {
    pub fn new_view(
        base_view: &'a S,
        hashmap_view: &'a MVHashMapView<'a, StateKey, WriteOp>,
    ) -> RemoteStorageOwned<VersionedView<'a, S>> {
        VersionedView {
            base_view,
            hashmap_view,
        }
        .into_move_resolver()
    }
}

impl<'a, S: StateView> TStateView for VersionedView<'a, S> {
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
