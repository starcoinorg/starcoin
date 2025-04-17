// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{inner::Inner, inner_vm2::InnerVM2};
use anyhow::{format_err, Result};
use starcoin_config::{NodeConfig, TimeService};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{
    ActorService, EventHandler, ServiceContext, ServiceFactory, ServiceHandler,
};
use starcoin_state_api::message::StateRequestVMType::{MoveVm1, MoveVm2};
use starcoin_state_api::message::{StateRequest, StateResponse};
use starcoin_state_api::{
    ChainStateReader, StateNodeStore, StateReaderExt, StateView, StateWithProof,
    StateWithTableItemProof,
};
use starcoin_state_tree::AccountStateSetIterator;
use starcoin_statedb::ChainStateDB;
use starcoin_storage::{BlockStore, Storage};
use starcoin_types::state_set::AccountStateSet;
use starcoin_types::system_events::NewHeadBlock;
use starcoin_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
    state_set::ChainStateSet,
};
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::table::{TableHandle, TableInfo};
use std::sync::Arc;

pub struct ChainStateService {
    service: (Inner, InnerVM2),
}

impl ChainStateService {
    pub fn new(
        store: Arc<dyn StateNodeStore>,
        root_hash: Option<HashValue>,
        store_vm2: Arc<dyn StateNodeStore>,
        root_hash_vm2: Option<HashValue>,
        time_service: Arc<dyn TimeService>,
    ) -> Self {
        Self {
            service: (
                Inner::new(store, root_hash, time_service.clone()),
                InnerVM2::new(store_vm2, root_hash_vm2, time_service.clone()),
            ),
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
        // self.service.0.adjust_time();
        // TODO(BobOng): [dual-vm] Here the timestamp of the new VM is used as the reference
        self.service.1.adjust_time();
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
            StateRequest::Get(vm_type, access_path) => StateResponse::State(match vm_type {
                MoveVm1 => self
                    .service
                    .0
                    .get_state_value(&StateKey::AccessPath(access_path))?,
                MoveVm2 => self
                    .service
                    .1
                    .get_state_value(&StateKey::AccessPath(access_path))?,
            }),
            StateRequest::GetWithProof(vm_type, access_path) => match vm_type {
                MoveVm1 => StateResponse::StateWithProof(Box::new(
                    self.service.0.get_with_proof(&access_path)?,
                )),
                MoveVm2 => StateResponse::StateWithProof(Box::new(
                    self.service.1.get_with_proof(&access_path)?,
                )),
            },
            StateRequest::GetAccountState(vm_type, address) => match vm_type {
                MoveVm1 => StateResponse::AccountState(self.service.0.get_account_state(&address)?),
                MoveVm2 => StateResponse::AccountState(self.service.1.get_account_state(&address)?),
            },
            StateRequest::StateRoot(vm_type) => match vm_type {
                MoveVm1 => StateResponse::StateRoot(self.service.0.state_root()),
                MoveVm2 => StateResponse::StateRoot(self.service.1.state_root()),
            },
            StateRequest::GetWithProofByRoot(vm_type, access_path, state_root) => match vm_type {
                MoveVm1 => StateResponse::StateWithProof(Box::new(
                    self.service
                        .0
                        .get_with_proof_by_root(access_path, state_root)?,
                )),
                MoveVm2 => StateResponse::StateWithProof(Box::new(
                    self.service
                        .1
                        .get_with_proof_by_root(access_path, state_root)?,
                )),
            },
            StateRequest::GetAccountStateByRoot(vm_type, account, state_root) => match vm_type {
                MoveVm1 => StateResponse::AccountState(
                    self.service
                        .0
                        .get_account_state_by_root(account, state_root)?,
                ),
                MoveVm2 => StateResponse::AccountState(
                    self.service
                        .1
                        .get_account_state_by_root(account, state_root)?,
                ),
            },
            StateRequest::GetAccountStateSet {
                vm_type,
                address,
                state_root,
            } => match vm_type {
                MoveVm1 => StateResponse::AccountStateSet(
                    self.service
                        .0
                        .get_account_state_set_with_root(address, state_root)?,
                ),
                MoveVm2 => StateResponse::AccountStateSet(
                    self.service
                        .1
                        .get_account_state_set_with_root(address, state_root)?,
                ),
            },
            StateRequest::GetWithTableItemProof(vm_type, handle, key) => match vm_type {
                MoveVm1 => StateResponse::StateWithTableItemProof(Box::new(
                    self.service.0.get_with_table_item_proof(&handle, &key)?,
                )),
                MoveVm2 => StateResponse::StateWithTableItemProof(Box::new(
                    self.service.1.get_with_table_item_proof(&handle, &key)?,
                )),
            },
            StateRequest::GetWithTableItemProofByRoot(vm_type, handle, key, state_root) => {
                match vm_type {
                    MoveVm1 => StateResponse::StateWithTableItemProof(Box::new(
                        self.service
                            .0
                            .get_with_table_item_proof_by_root(handle, key, state_root)?,
                    )),
                    MoveVm2 => StateResponse::StateWithTableItemProof(Box::new(
                        self.service
                            .1
                            .get_with_table_item_proof_by_root(handle, key, state_root)?,
                    )),
                }
            }
            StateRequest::GetTableInfo(vm_type, address) => match vm_type {
                MoveVm1 => StateResponse::TableInfo(self.service.0.get_table_info(address)?),
                MoveVm2 => StateResponse::TableInfo(self.service.1.get_table_info(address)?),
            },
        };
        Ok(response)
    }
}

impl EventHandler<Self, NewHeadBlock> for ChainStateService {
    fn handle_event(&mut self, msg: NewHeadBlock, _ctx: &mut ServiceContext<ChainStateService>) {
        let NewHeadBlock(block) = msg;

        let state_root = block.header().state_root();
        debug!("ChainStateActor change StateRoot to : {:?}", state_root);

        // TODO(BobOng): [dual-vm] There should be two state roots passed here
        self.service.0.change_root(state_root);
        self.service.1.change_root(state_root);
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
