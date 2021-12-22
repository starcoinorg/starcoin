use crate::HashValue;
use dashmap::DashMap;
use starcoin_state_api::ChainStateWriter;
use starcoin_types::state_set::ChainStateSet;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::write_set::{WriteOp, WriteSet, WriteSetMut};

pub struct InMemoryStateCache<V> {
    data_map: DashMap<AccessPath, Option<Vec<u8>>>,
    data_view: V,
}

impl<V> InMemoryStateCache<V> {
    /// Create a `StateViewCache` give a `StateView`. Hold updates to the data store and
    /// forward data request to the `StateView` if not in the local cache.
    pub fn new(data_view: V) -> Self {
        InMemoryStateCache {
            data_view,
            data_map: DashMap::new(),
        }
    }

    // Publishes a `WriteSet` computed at the end of a transaction.
    // The effect is to build a layer in front of the `StateView` which keeps
    // track of the data as if the changes were applied immediately.
    pub(crate) fn push_write_set(&self, write_set: &WriteSet) {
        for (ref ap, ref write_op) in write_set.iter() {
            match write_op {
                WriteOp::Value(blob) => {
                    self.data_map.insert(ap.clone(), Some(blob.clone()));
                }
                WriteOp::Deletion => {
                    self.data_map.remove(ap);
                    self.data_map.insert(ap.clone(), None);
                }
            }
        }
    }
}

impl<V> ChainStateWriter for InMemoryStateCache<V> {
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> anyhow::Result<()> {
        self.apply_write_set(
            WriteSetMut::new(vec![(access_path.clone(), WriteOp::Value(value))]).freeze()?,
        )
    }

    fn remove(&self, access_path: &AccessPath) -> anyhow::Result<()> {
        self.apply_write_set(
            WriteSetMut::new(vec![(access_path.clone(), WriteOp::Deletion)]).freeze()?,
        )
    }

    fn apply(&self, _state_set: ChainStateSet) -> anyhow::Result<()> {
        unimplemented!()
    }

    fn apply_write_set(&self, write_set: WriteSet) -> anyhow::Result<()> {
        self.push_write_set(&write_set);
        Ok(())
    }

    fn commit(&self) -> anyhow::Result<HashValue> {
        Ok(HashValue::random())
    }

    fn flush(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
impl<V: StateView> StateView for InMemoryStateCache<V> {
    // Get some data either through the cache or the `StateView` on a cache miss.
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        match self.data_map.get(access_path) {
            Some(opt_data) => Ok(opt_data.clone()),
            None => match self.data_view.get(access_path) {
                Ok(remote_data) => Ok(remote_data),
                // TODO: should we forward some error info?
                Err(e) => {
                    // error!("[VM] Error getting data from storage for {:?}", access_path);
                    Err(e)
                }
            },
        }
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> anyhow::Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        self.data_view.is_genesis()
    }
}
