use anyhow::Result;
use parking_lot::Mutex;
use starcoin_crypto::HashValue;
use starcoin_state_store_api::StateNode;
use std::collections::BTreeMap;

// Fixme, remove me and the implementations, use Storage instead.
#[derive(Default)]
pub struct StateStorageMock {
    inner: Mutex<BTreeMap<HashValue, StateNode>>,
}

impl StateStorageMock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &HashValue) -> Result<Option<StateNode>> {
        Ok(self.inner.lock().get(key).cloned())
    }

    pub fn put(&self, key: HashValue, value: StateNode) -> Result<()> {
        self.inner.lock().insert(key, value);
        Ok(())
    }

    pub fn write_batch(&self, mut nodes: BTreeMap<HashValue, StateNode>) -> Result<()> {
        self.inner.lock().append(&mut nodes);
        Ok(())
    }
}
