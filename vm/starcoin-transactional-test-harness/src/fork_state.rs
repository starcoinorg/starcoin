use std::collections::BTreeMap;
use std::sync::Arc;

use crate::HashValue;
use anyhow::{anyhow, Result};
use starcoin_state_api::StateNodeStore;
use starcoin_storage::state_node::StateStorage;
use starcoin_storage::storage::{CodecKVStore, CodecWriteBatch, StorageInstance};

use starcoin_rpc_api::state::StateApiClient;
use starcoin_state_tree::StateNode;
use tokio::runtime::Runtime;

pub struct MockStateNodeStore {
    local_storage: StateStorage,
    remote: Arc<StateApiClient>,
    rt: Arc<Runtime>,
}

impl MockStateNodeStore {
    pub fn new(remote: Arc<StateApiClient>, rt: Arc<Runtime>) -> Result<Self> {
        let storage_instance = StorageInstance::new_cache_instance();
        let storage = StateStorage::new(storage_instance);

        Ok(Self {
            local_storage: storage,
            remote,
            rt,
        })
    }
}

impl StateNodeStore for MockStateNodeStore {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        match self.local_storage.get(*hash)? {
            Some(sn) => Ok(Some(sn)),
            None => {
                let handle = self.rt.handle().clone();
                let blob = handle
                    .block_on(async move { self.remote.get_by_hash(*hash).await })
                    .map(|res| res.map(|b| StateNode(b)))
                    .map_err(|e| anyhow!("{}", e))?;

                // Put result to local storage to accelerate the following getting.
                if let Some(node) = blob.clone() {
                    self.put(*hash, node)?;
                };
                Ok(blob)
            }
        }
    }

    fn put(&self, key: HashValue, node: StateNode) -> Result<()> {
        self.local_storage.put(key, node)
    }

    fn write_nodes(&self, nodes: BTreeMap<HashValue, StateNode>) -> Result<()> {
        let batch = CodecWriteBatch::new_puts(nodes.into_iter().collect());
        self.local_storage.write_batch(batch)
    }
}
