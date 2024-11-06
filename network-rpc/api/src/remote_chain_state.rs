// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    gen_client::NetworkRpcClient, GetAccountState, GetStateWithProof, GetStateWithTableItemProof,
    GetTableInfo,
};
use anyhow::{anyhow, Result};
use network_p2p_types::peer_id::PeerId;
use starcoin_crypto::HashValue;
use starcoin_state_api::{ChainStateReader, StateWithProof, StateWithTableItemProof};
use starcoin_state_tree::AccountStateSetIterator;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::state_set::{AccountStateSet, ChainStateSet};
use starcoin_vm_types::state_store::errors::StateviewError;
use starcoin_vm_types::state_store::state_key::inner::StateKeyInner;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::state_storage_usage::StateStorageUsage;
use starcoin_vm_types::state_store::state_value::StateValue;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use starcoin_vm_types::state_store::TStateView;

#[derive(Clone)]
pub struct RemoteChainStateReader {
    peer_id: Option<PeerId>,
    state_root: Option<HashValue>,
    client: NetworkRpcClient,
}

impl RemoteChainStateReader {
    pub fn new(client: NetworkRpcClient) -> Self {
        Self {
            peer_id: None,
            state_root: None,
            client,
        }
    }
    pub fn with(&self, peer_id: PeerId, state_root: HashValue) -> Self {
        Self {
            peer_id: Some(peer_id),
            state_root: Some(state_root),
            client: self.client.clone(),
        }
    }
}

impl ChainStateReader for RemoteChainStateReader {
    fn get_with_proof(&self, state_key: &StateKey) -> Result<StateWithProof> {
        let access_path = match state_key.inner() {
            StateKeyInner::AccessPath(access_path) => access_path,
            _ => {
                return Err(anyhow!("Only support AccessPath key"));
            }
        };
        let peer_id = self
            .peer_id
            .clone()
            .ok_or_else(|| anyhow!("peer id not set"))?;
        let state_root = self
            .state_root
            .ok_or_else(|| anyhow!("state root not set"))?;
        let req = GetStateWithProof {
            state_root,
            access_path: access_path.clone(),
        };
        let client = self.client.clone();
        let state_proof: StateWithProof =
            futures::executor::block_on(client.get_state_with_proof(peer_id, req))?;
        state_proof.proof.verify(
            state_root,
            access_path.clone(),
            state_proof.state.as_deref(),
        )?;
        Ok(state_proof)
    }

    fn get_account_state(&self, account_address: &AccountAddress) -> Result<AccountState> {
        //TODO: How to verify it
        let peer_id = self
            .peer_id
            .clone()
            .ok_or_else(|| anyhow!("peer id not set"))?;
        let state_root = self
            .state_root
            .ok_or_else(|| anyhow!("state root not set"))?;
        let req = GetAccountState {
            state_root,
            account_address: account_address.to_owned(),
        };
        let client = self.client.clone();
        futures::executor::block_on(async { client.get_account_state(peer_id, req).await })
    }

    fn get_account_state_set(&self, _address: &AccountAddress) -> Result<Option<AccountStateSet>> {
        unimplemented!()
    }
    fn state_root(&self) -> HashValue {
        match self.state_root {
            Some(state_root) => state_root,
            None => unreachable!(),
        }
    }
    fn dump(&self) -> Result<ChainStateSet> {
        unimplemented!()
    }

    fn dump_iter(&self) -> Result<AccountStateSetIterator> {
        unimplemented!()
    }

    fn get_with_table_item_proof(
        &self,
        handle: &TableHandle,
        key: &[u8],
    ) -> Result<StateWithTableItemProof> {
        let peer_id = self
            .peer_id
            .clone()
            .ok_or_else(|| anyhow!("peer id not set"))?;
        let state_root = self
            .state_root
            .ok_or_else(|| anyhow!("state root not set"))?;
        let req = GetStateWithTableItemProof {
            state_root,
            handle: *handle,
            key: key.to_vec(),
        };
        let client = self.client.clone();
        let state_table_item_proof: StateWithTableItemProof =
            futures::executor::block_on(client.get_state_with_table_item_proof(peer_id, req))?;
        state_table_item_proof.verify(handle, key)?;
        Ok(state_table_item_proof)
    }

    fn get_table_info(&self, address: AccountAddress) -> Result<TableInfo> {
        let peer_id = self
            .peer_id
            .clone()
            .ok_or_else(|| anyhow!("peer id not set"))?;
        let req = GetTableInfo(address);
        let client = self.client.clone();
        let table_info = futures::executor::block_on(client.get_state_table_info(peer_id, req))?;
        Ok(table_info)
    }
}

impl TStateView for RemoteChainStateReader {
    type Key = StateKey;
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<StateValue>, StateviewError> {
        match state_key.inner() {
            StateKeyInner::AccessPath(_access_path) => {
                let state_proof = self.get_with_proof(state_key)?;
                Ok(state_proof.state.map(StateValue::from))
            }
            StateKeyInner::TableItem { handle, key } => {
                let state_proof = self.get_with_table_item_proof(handle, key)?;
                Ok(state_proof.key_proof.0.map(StateValue::from))
            }
            StateKeyInner::Raw(_) => unimplemented!(),
        }
    }

    fn get_usage(&self) -> Result<StateStorageUsage, StateviewError> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
