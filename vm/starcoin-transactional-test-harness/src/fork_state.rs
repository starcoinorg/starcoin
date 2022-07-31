use std::sync::Arc;

use crate::HashValue;
use anyhow::anyhow;
use dashmap::DashMap;
use starcoin_rpc_client::{RemoteStateReader, RpcClient, StateRootOption};
use starcoin_state_api::{ChainStateReader, ChainStateWriter, StateWithProof};
use starcoin_statedb::ChainStateDB;
use starcoin_types::state_set::ChainStateSet;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::state_view::StateView;
use starcoin_vm_types::write_set::{WriteOp, WriteSet, WriteSetMut};

use starcoin_rpc_api::state::StateApiClient;
use starcoin_state_tree::AccountStateSetIterator;
use tokio::runtime::Runtime;

pub struct ForkStateDB {
    local: ChainStateDB,
    remote: Arc<StateApiClient>,
    rt: Arc<Runtime>,
}

impl ForkStateDB {
    pub fn new(
        local_db: ChainStateDB,
        state_client: Arc<StateApiClient>,
        rt: Arc<Runtime>,
    ) -> Self {
        ForkStateDB {
            local: local_db,
            remote: state_client,
            rt,
        }
    }

    fn call_rpc_blocking<F, T>(
        &self,
        f: impl FnOnce(Arc<StateApiClient>) -> F + Send,
    ) -> anyhow::Result<T>
    where
        T: Send,
        F: std::future::Future<Output = Result<T, jsonrpc_client_transports::RpcError>> + Send,
    {
        let client = self.remote.clone();
        self.rt
            .handle()
            .clone()
            .block_on(f(client))
            .map_err(|e| anyhow!(format!("{}", e)))
    }
}

impl ChainStateWriter for ForkStateDB {
    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> anyhow::Result<()> {
        self.local.set(access_path, value)
    }

    fn remove(&self, access_path: &AccessPath) -> anyhow::Result<()> {
        self.local.remove(access_path)
    }

    fn apply(&self, state_set: ChainStateSet) -> anyhow::Result<()> {
        self.local.apply(state_set)
    }

    fn apply_write_set(&self, write_set: WriteSet) -> anyhow::Result<()> {
        self.local.apply_write_set(write_set)
    }

    fn commit(&self) -> anyhow::Result<HashValue> {
        self.local.commit()
    }

    fn flush(&self) -> anyhow::Result<()> {
        self.local.flush()
    }
}

impl ChainStateReader for ForkStateDB {
    fn get_with_proof(&self, access_path: &AccessPath) -> anyhow::Result<StateWithProof> {
        let local = self.local.get_with_proof(access_path)?;
        match local.clone().state {
            Some(_st) => Ok(local),
            None => {
                let access_path = access_path.clone();
                self.call_rpc_blocking(|client| client.get_with_proof(access_path))
                    .map(Into::into)
            }
        }
    }

    fn get_account_state(
        &self,
        address: &move_core_types::account_address::AccountAddress,
    ) -> anyhow::Result<Option<starcoin_types::account_state::AccountState>> {
        todo!()
    }

    fn get_account_state_set(
        &self,
        address: &move_core_types::account_address::AccountAddress,
    ) -> anyhow::Result<Option<starcoin_types::state_set::AccountStateSet>> {
        todo!()
    }

    fn state_root(&self) -> HashValue {
        todo!()
    }

    fn dump(&self) -> anyhow::Result<ChainStateSet> {
        todo!()
    }

    fn dump_iter(&self) -> anyhow::Result<AccountStateSetIterator> {
        todo!()
    }
}

impl StateView for ForkStateDB {
    fn get(&self, access_path: &AccessPath) -> anyhow::Result<Option<Vec<u8>>> {
        match self.local.get(access_path)? {
            Some(opt_data) => Ok(Some(opt_data.clone())),
            None => self.call_rpc_blocking(|client| client.get(access_path.clone())),
        }
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
