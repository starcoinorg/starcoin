// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use starcoin_config::{NodeConfig, TimeService};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler,
};
use starcoin_state_api::message::{StateRequest, StateResponse};
use starcoin_state_api::{
    ChainStateReader, StateNodeStore, StateReaderExt, StateView, StateWithProof,
};
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{BlockStore, Storage};
use starcoin_types::state_set::AccountStateSet;
use starcoin_types::system_events::NewHeadBlock;
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
    state_set::ChainStateSet,
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
    fn create(ctx: &mut ServiceContext<ChainStateService>) -> Result<ChainStateService> {
        let config = ctx.get_shared::<Arc<NodeConfig>>()?;
        let storage = ctx.get_shared::<Arc<Storage>>()?;
        let startup_info = storage
            .get_startup_info()?
            .ok_or_else(|| format_err!("Startup info should exist at service init."))?;
        let head_block = storage.get_block(startup_info.main)?.ok_or_else(|| {
            format_err!("Can not find head block by hash:{:?}", startup_info.main)
        })?;
        Ok(Self::new(
            storage,
            Some(head_block.header().state_root()),
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
        _ctx: &mut ServiceContext<ChainStateService>,
    ) -> Result<StateResponse> {
        let response = match msg {
            StateRequest::Get(access_path) => StateResponse::State(self.service.get(&access_path)?),
            StateRequest::GetWithProof(access_path) => {
                StateResponse::StateWithProof(Box::new(self.service.get_with_proof(&access_path)?))
            }
            StateRequest::GetAccountState(address) => {
                StateResponse::AccountState(self.service.get_account_state(&address)?)
            }
            StateRequest::StateRoot() => StateResponse::StateRoot(self.service.state_root()),
            StateRequest::GetWithProofByRoot(access_path, state_root) => {
                StateResponse::StateWithProof(Box::new(
                    self.service
                        .get_with_proof_by_root(access_path, state_root)?,
                ))
            }
            StateRequest::GetAccountStateByRoot(account, state_root) => {
                StateResponse::AccountState(
                    self.service
                        .get_account_state_by_root(account, state_root)?,
                )
            }
            StateRequest::GetAccountStateSet {
                address,
                state_root,
            } => StateResponse::AccountStateSet(
                self.service
                    .get_account_state_set_with_root(address, state_root)?,
            ),
        };
        Ok(response)
    }
}

impl EventHandler<Self, NewHeadBlock> for ChainStateService {
    fn handle_event(&mut self, msg: NewHeadBlock, _ctx: &mut ServiceContext<ChainStateService>) {
        let NewHeadBlock(block) = msg;

        let state_root = block.header().state_root();
        debug!("ChainStateActor change StateRoot to : {:?}", state_root);
        self.service.change_root(state_root);
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
        access_path: AccessPath,
        state_root: HashValue,
    ) -> Result<StateWithProof> {
        let reader = self.state_db.fork_at(state_root);
        reader.get_with_proof(&access_path)
    }

    pub(crate) fn get_account_state_by_root(
        &self,
        account: AccountAddress,
        state_root: HashValue,
    ) -> Result<Option<AccountState>> {
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
                self.time_service.adjust(on_chain_time);
            }
            Err(e) => {
                error!("Get global time on chain fail: {:?}", e);
            }
        }
    }
}

impl ChainStateReader for Inner {
    fn get_with_proof(&self, access_path: &AccessPath) -> Result<StateWithProof> {
        self.state_db.get_with_proof(access_path)
    }

    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<AccountState>> {
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
}

impl StateView for Inner {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        self.state_db.get(access_path)
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
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
    use starcoin_state_api::ChainStateAsyncService;
    use starcoin_types::account_config::genesis_address;

    #[stest::test]
    async fn test_actor_launch() -> Result<()> {
        let config = Arc::new(NodeConfig::random_for_test());
        let (storage, _startup_info, _) =
            test_helper::Genesis::init_storage_for_test(config.net())?;
        let registry = RegistryService::launch();
        registry.put_shared(config).await?;
        registry.put_shared(storage).await?;
        let service_ref = registry.register::<ChainStateService>().await?;
        let account_state = service_ref.get_account_state(genesis_address()).await?;
        assert!(account_state.is_some());
        Ok(())
    }
}
