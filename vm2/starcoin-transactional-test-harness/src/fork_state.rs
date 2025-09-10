// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::HashValue;
use anyhow::{anyhow, Result};
use jsonrpc_http_server::hyper::body::Bytes;
use starcoin_state_tree::StateNode;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{
    state_node::StateStorage,
    storage::{CodecKVStore, CodecWriteBatch, StorageInstance},
};

use starcoin_state_api::ChainStateReader as ChainStateReader1;
use starcoin_storage::table_info::TableInfoStore;
use starcoin_types::table::{StcTableHandle, StcTableInfo};
use starcoin_vm2_rpc_api::state_api::StateApiClient as StateApiClient2;
use starcoin_vm2_state_api::{
    ChainStateAsyncService, StateNodeStore, StateWithProof, StateWithTableItemProof,
};
use starcoin_vm2_types::account_address::AccountAddress as AccountAddress2;
use starcoin_vm2_types::account_state::AccountState;
use starcoin_vm2_vm_types::state_store::{state_key::StateKey, table::TableHandle};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use tokio::runtime::Runtime;

pub struct MockStateNodeStore {
    local_storage: StateStorage,
    remote: Arc<StateApiClient2>,
    rt: Arc<Runtime>,
}

impl MockStateNodeStore {
    pub fn new(remote: Arc<StateApiClient2>, rt: Arc<Runtime>) -> Self {
        let storage_instance = StorageInstance::new_cache_instance();
        let storage = StateStorage::new(storage_instance);

        Self {
            local_storage: storage,
            remote,
            rt,
        }
    }
}

impl StateNodeStore for MockStateNodeStore {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>> {
        match self.local_storage.get(*hash)? {
            Some(sn) => Ok(Some(sn)),
            None => {
                let client = self.remote.clone();
                let handle = self.rt.handle().clone();
                let hash = *hash;
                let blob = handle
                    .block_on(client.get_state_node_by_node_hash(hash))
                    .map(|res| res.map(|b| StateNode(b.to_vec())))
                    .map_err(|e| anyhow!("{}", e))?;

                if let Some(node) = blob.clone() {
                    self.put(hash, node)?;
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

impl TableInfoStore for MockStateNodeStore {
    fn get_table_info(&self, _key: StcTableHandle) -> Result<Option<StcTableInfo>> {
        Ok(None)
    }
    fn save_table_info(&self, _key: StcTableHandle, _table_info: StcTableInfo) -> Result<()> {
        Ok(())
    }
    fn get_table_infos(&self, _keys: Vec<StcTableHandle>) -> Result<Vec<Option<StcTableInfo>>> {
        Ok(vec![])
    }
    fn save_table_infos(&self, _table_infos: Vec<(StcTableHandle, StcTableInfo)>) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct MockChainStateAsyncService {
    state_store: Arc<dyn StateNodeStore>,
    root: Arc<Mutex<HashValue>>,
}

impl MockChainStateAsyncService {
    pub fn new(state_store: Arc<dyn StateNodeStore>, root: Arc<Mutex<HashValue>>) -> Self {
        Self { state_store, root }
    }

    fn state_db(&self) -> ChainStateDB {
        let root = self.root.lock().unwrap();
        ChainStateDB::new(self.state_store.clone(), Some(*root))
    }
}

#[async_trait::async_trait]
impl ChainStateAsyncService for MockChainStateAsyncService {
    async fn get(self, _state_key: StateKey) -> Result<Option<Bytes>> {
        unimplemented!()
    }

    async fn get_with_proof(self, _state_key: StateKey) -> Result<StateWithProof> {
        unimplemented!()
    }

    async fn get_account_state(self, _address: AccountAddress2) -> Result<AccountState> {
        unimplemented!()
    }

    async fn get_account_state_set(
        self,
        _address: AccountAddress2,
        _state_root: Option<HashValue>,
    ) -> Result<starcoin_vm2_types::state_set::AccountStateSet> {
        unimplemented!()
    }
    async fn state_root(self) -> Result<HashValue> {
        Ok(self.state_db().state_root())
    }

    async fn get_with_proof_by_root(
        self,
        _state_key: StateKey,
        _state_root: HashValue,
    ) -> Result<StateWithProof> {
        unimplemented!()
    }

    async fn get_account_state_by_root(
        self,
        _account_address: AccountAddress2,
        _state_root: HashValue,
    ) -> Result<AccountState> {
        unimplemented!()
    }

    async fn get_with_table_item_proof(
        self,
        handle: TableHandle,
        key: Vec<u8>,
    ) -> Result<StateWithTableItemProof> {
        unimplemented!()
    }

    async fn get_with_table_item_proof_by_root(
        self,
        _handle: TableHandle,
        _key: Vec<u8>,
        _state_root: HashValue,
    ) -> Result<StateWithTableItemProof> {
        unimplemented!()
    }
}
