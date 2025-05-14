// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use starcoin_config::{NodeConfig, TimeService};
use starcoin_logger::prelude::*;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler,
};
use starcoin_storage::{BlockStore, Storage, Store};
use starcoin_types::system_events::NewHeadBlock;
use starcoin_vm2_crypto::HashValue;
use starcoin_vm2_state_api::{
    message::{StateRequest, StateResponse},
    ChainStateReader, StateNodeStore, StateReaderExt, StateWithProof, StateWithTableItemProof,
};
use starcoin_vm2_state_tree::AccountStateSetIterator;
use starcoin_vm2_statedb::ChainStateDB;
use starcoin_vm2_storage::Storage as Storage2;
use starcoin_vm2_types::{
    account_address::AccountAddress, account_state::AccountState, state_set::AccountStateSet,
    state_set::ChainStateSet,
};
use starcoin_vm2_vm_types::state_store::{
    errors::StateviewError,
    state_key::inner::StateKeyInner,
    state_key::StateKey,
    state_storage_usage::StateStorageUsage,
    state_value::StateValue,
    table::{TableHandle, TableInfo},
    TStateView,
};
use std::sync::Arc;

pub struct ChainStateService {
    service: Inner,
}

impl ChainStateService {
    pub fn new(
        store: Arc<dyn StateNodeStore>,
        root_hash: Option<HashValue>,
        time_service: Arc<dyn TimeService>,
    ) -> Self {
        Self {
            service: Inner::new(store, root_hash, time_service),
        }
    }
}

impl ServiceFactory<Self> for ChainStateService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let storage2 = ctx.get_shared::<Arc<Storage2>>()?;
        let startup_info = storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("Startup info should exist at service init."))?;
        let multi_state = storage.get_vm_multi_state(startup_info.main)?;
        Ok(Self::new(
            storage2,
            Some(multi_state.state_root2()),
            config.net().time_service(),
        ))
    }
}

impl ActorService for ChainStateService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<NewHeadBlock>();
        self.service.adjust_time();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<NewHeadBlock>();
        Ok(())
    }
}

impl ServiceHandler<Self, StateRequest> for ChainStateService {
    fn handle(
        &mut self,
        msg: StateRequest,
        _ctx: &mut ServiceContext<Self>,
    ) -> Result<StateResponse> {
        let response = match msg {
            StateRequest::Get(state_key) => {
                StateResponse::State(self.service.get_state_value_bytes(&state_key)?)
            }
            StateRequest::GetWithProof(state_key) => {
                let _access_path = match state_key.inner() {
                    StateKeyInner::AccessPath(access_path) => access_path,
                    _ => {
                        return Err(format_err!("Invalid StateKey."));
                    }
                };
                StateResponse::StateWithProof(Box::new(self.service.get_with_proof(&state_key)?))
            }
            StateRequest::GetAccountState(address) => {
                StateResponse::AccountState(Option::from(self.service.get_account_state(&address)?))
            }
            StateRequest::StateRoot() => StateResponse::StateRoot(self.service.state_root()),
            StateRequest::GetWithProofByRoot(state_key, state_root) => {
                let _access_path = match state_key.inner() {
                    StateKeyInner::AccessPath(access_path) => access_path,
                    _ => {
                        return Err(format_err!("Invalid StateKey."));
                    }
                };
                StateResponse::StateWithProof(Box::new(
                    self.service
                        .get_with_proof_by_root(&state_key, state_root)?,
                ))
            }
            StateRequest::GetAccountStateByRoot(account, state_root) => {
                StateResponse::AccountState(Some(
                    self.service
                        .get_account_state_by_root(account, state_root)?,
                ))
            }
            StateRequest::GetAccountStateSet {
                address,
                state_root,
            } => StateResponse::AccountStateSet(
                self.service
                    .get_account_state_set_with_root(address, state_root)?,
            ),
            StateRequest::GetWithTableItemProof(handle, key) => {
                StateResponse::StateWithTableItemProof(Box::new(
                    self.service.get_with_table_item_proof(&handle, &key)?,
                ))
            }
            StateRequest::GetWithTableItemProofByRoot(handle, key, state_root) => {
                StateResponse::StateWithTableItemProof(Box::new(
                    self.service
                        .get_with_table_item_proof_by_root(handle, key, state_root)?,
                ))
            }
            StateRequest::GetTableInfo(address) => {
                StateResponse::TableInfo(Some(self.service.get_table_info(address)?))
            }
        };
        Ok(response)
    }
}

impl EventHandler<Self, NewHeadBlock> for ChainStateService {
    fn handle_event(&mut self, msg: NewHeadBlock, _ctx: &mut ServiceContext<Self>) {
        let state_root = msg.0.multi_state();
        debug!("VM2 ChainStateActor change StateRoot to : {:?}", state_root);
        self.service.change_root(state_root.state_root2());
    }
}

pub struct Inner {
    state_db: ChainStateDB,
    //for adjust local time by on chain time.
    time_service: Arc<dyn TimeService>,
}

impl Inner {
    pub fn new(
        store: Arc<dyn StateNodeStore>,
        root_hash: Option<HashValue>,
        time_service: Arc<dyn TimeService>,
    ) -> Self {
        Self {
            state_db: ChainStateDB::new(store, root_hash),
            time_service,
        }
    }

    pub(crate) fn get_account_state_set_with_root(
        &self,
        address: AccountAddress,
        state_root: Option<HashValue>,
    ) -> Result<Option<AccountStateSet>> {
        match state_root {
            Some(root) => {
                let reader = self.state_db.fork_at(root);
                reader.get_account_state_set(&address)
            }
            None => self.get_account_state_set(&address),
        }
    }

    pub(crate) fn get_with_proof_by_root(
        &self,
        state_key: &StateKey,
        state_root: HashValue,
    ) -> Result<StateWithProof> {
        let reader = self.state_db.fork_at(state_root);
        reader.get_with_proof(state_key)
    }

    pub(crate) fn get_with_table_item_proof_by_root(
        &self,
        handle: TableHandle,
        key: Vec<u8>,
        state_root: HashValue,
    ) -> Result<StateWithTableItemProof> {
        let reader = self.state_db.fork_at(state_root);
        reader.get_with_table_item_proof(&handle, &key)
    }

    pub(crate) fn get_account_state_by_root(
        &self,
        account: AccountAddress,
        state_root: HashValue,
    ) -> Result<AccountState> {
        let reader = self.state_db.fork_at(state_root);
        reader.get_account_state(&account)
    }

    pub(crate) fn change_root(&mut self, state_root: HashValue) {
        self.state_db = self.state_db.fork_at(state_root);
        self.adjust_time();
    }

    pub fn adjust_time(&self) {
        match self.state_db.get_timestamp() {
            Ok(on_chain_time) => {
                self.time_service.adjust(on_chain_time.microseconds / 1000);
            }
            Err(e) => {
                error!("Get global time on chain fail: {:?}", e);
            }
        }
    }
}

impl ChainStateReader for Inner {
    fn get_with_proof(&self, state_key: &StateKey) -> Result<StateWithProof> {
        self.state_db.get_with_proof(state_key)
    }

    fn get_account_state(&self, address: &AccountAddress) -> Result<AccountState> {
        self.state_db.get_account_state(address)
    }
    fn get_account_state_set(&self, address: &AccountAddress) -> Result<Option<AccountStateSet>> {
        self.state_db.get_account_state_set(address)
    }

    fn state_root(&self) -> HashValue {
        self.state_db.state_root()
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
        self.state_db.get_with_table_item_proof(handle, key)
    }

    fn get_table_info(&self, address: AccountAddress) -> Result<TableInfo> {
        self.state_db.get_table_info(address)
    }
}

impl TStateView for Inner {
    type Key = StateKey;
    fn get_state_value(&self, state_key: &StateKey) -> Result<Option<StateValue>, StateviewError> {
        self.state_db.get_state_value(state_key)
    }

    fn get_usage(&self) -> Result<StateStorageUsage, StateviewError> {
        unimplemented!("get_usage not implemented for ChainStateService")
    }

    fn is_genesis(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_config::NodeConfig;
    use starcoin_service_registry::{RegistryAsyncService, RegistryService};
    use starcoin_vm2_state_api::ChainStateAsyncService;
    use starcoin_vm2_types::account_config::genesis_address;

    #[stest::test]
    async fn test_actor_launch() -> Result<()> {
        let config = Arc::new(NodeConfig::random_for_test());
        let (storage, storage2, _startup_info, _) =
            test_helper::Genesis::init_storage_for_test_v2(config.net())?;
        let registry = RegistryService::launch();
        registry.put_shared(config).await?;
        registry.put_shared(storage).await?;
        registry.put_shared(storage2).await?;
        let service_ref = registry.register::<ChainStateService>().await?;
        let account_state = service_ref.get_account_state(genesis_address()).await;
        assert!(account_state.is_ok());
        Ok(())
    }
}
