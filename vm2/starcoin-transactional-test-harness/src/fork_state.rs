// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::remote_state::RemoteRpcAsyncClient;
use crate::HashValue;
use anyhow::{anyhow, format_err, Result};
use bytes::Bytes;
use starcoin_state_tree::StateNode;
use starcoin_storage::{
    state_node::StateStorage,
    storage::{CodecKVStore, CodecWriteBatch, StorageInstance},
    table_info::TableInfoStore,
};

use starcoin_types::table::{StcTableHandle, StcTableInfo};
use starcoin_vm2_state_api::{
    ChainStateAsyncService, ChainStateReader, StateNodeStore, StateWithProof,
    StateWithTableItemProof,
};
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_types::{
    account_address::AccountAddress, account_state::AccountState, state_set::AccountStateSet,
};
use starcoin_vm2_vm_types::state_store::{state_key::StateKey, table::TableHandle, TStateView};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use tokio::runtime::Runtime;

pub struct MockStateNodeStore {
    local_storage: StateStorage,
    remote: Arc<RemoteRpcAsyncClient>,
    rt: Arc<Runtime>,
}

impl MockStateNodeStore {
    pub fn new(remote: Arc<RemoteRpcAsyncClient>, rt: Arc<Runtime>) -> Self {
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

impl ChainStateAsyncService for MockChainStateAsyncService {
    async fn get(self, state_key: StateKey) -> Result<Option<Bytes>> {
        self.state_db()
            .get_state_value_bytes(&state_key)
            .map_err(|e| format_err!("get state value by key: {:?} error: {:?}", state_key, e))
    }

    async fn get_with_proof(self, state_key: StateKey) -> Result<StateWithProof> {
        self.state_db().get_with_proof(&state_key)
    }

    async fn get_account_state(self, address: AccountAddress) -> Result<AccountState> {
        self.state_db().get_account_state(&address)
    }

    async fn get_account_state_set(
        self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> Result<AccountStateSet> {
        let res = match state_root {
            Some(root) => {
                let reader = self.state_db().fork_at(root);
                reader.get_account_state_set(&address)
            }
            None => self.state_db().get_account_state_set(&address),
        };
        match res {
            Ok(Some(set)) => Ok(set),
            Ok(None) => Err(format_err!(
                "Can not find account state set by address: {}",
                address
            )),
            Err(e) => Err(e),
        }
    }
    async fn state_root(self) -> Result<HashValue> {
        Ok(self.state_db().state_root())
    }

    async fn get_with_proof_by_root(
        self,
        state_key: StateKey,
        state_root: HashValue,
    ) -> Result<StateWithProof> {
        let reader = self.state_db().fork_at(state_root);
        reader.get_with_proof(&state_key)
    }

    async fn get_account_state_by_root(
        self,
        account_address: AccountAddress,
        state_root: HashValue,
    ) -> Result<AccountState> {
        let reader = self.state_db().fork_at(state_root);
        reader.get_account_state(&account_address)
    }

    async fn get_with_table_item_proof(
        self,
        handle: TableHandle,
        key: Vec<u8>,
    ) -> Result<StateWithTableItemProof> {
        let reader = self.state_db();
        reader.get_with_table_item_proof(&handle, &key)
    }

    async fn get_with_table_item_proof_by_root(
        self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> Result<StateWithTableItemProof> {
        let reader = self.state_db().fork_at(state_root);
        reader.get_with_table_item_proof(&handle, &key)
    }
}
