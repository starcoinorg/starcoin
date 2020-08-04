// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::{StateRequest, StateResponse};
use crate::service::ChainStateServiceImpl;
use actix::prelude::*;
use anyhow::{Error, Result};
use starcoin_bus::{Bus, BusActor};
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_state_api::{
    ChainStateAsyncService, ChainStateReader, ChainStateService, StateNodeStore, StateView,
    StateWithProof,
};
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::system_events::NewHeadBlock;
use std::sync::Arc;

pub struct ChainStateActor {
    bus: Addr<BusActor>,
    service: ChainStateServiceImpl,
}

impl ChainStateActor {
    pub fn launch(
        bus: Addr<BusActor>,
        store: Arc<dyn StateNodeStore>,
        root_hash: Option<HashValue>,
    ) -> Result<ChainStateActorRef> {
        let actor = ChainStateActor {
            bus,
            service: ChainStateServiceImpl::new(store, root_hash),
        };
        Ok(ChainStateActorRef(actor.start()))
    }
}

impl Actor for ChainStateActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient = ctx.address().recipient::<NewHeadBlock>();
        self.bus
            .clone()
            .subscribe(recipient)
            .into_actor(self)
            .then(|res, act, ctx| {
                if let Err(e) = res {
                    error!("fail to subscribe system events, err: {:?}", e);
                    ctx.terminate();
                }
                async {}.into_actor(act)
            })
            .wait(ctx);
        info!("ChainStateActor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("ChainStateActor stopped");
    }
}

impl Handler<StateRequest> for ChainStateActor {
    type Result = Result<StateResponse>;

    fn handle(&mut self, msg: StateRequest, _ctx: &mut Self::Context) -> Self::Result {
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
        };
        Ok(response)
    }
}

impl actix::Handler<NewHeadBlock> for ChainStateActor {
    type Result = ();

    fn handle(&mut self, msg: NewHeadBlock, _ctx: &mut Self::Context) -> Self::Result {
        let NewHeadBlock(block) = msg;

        let state_root = block.header().state_root();
        debug!("ChainStateActor change StateRoot to : {:?}", state_root);
        self.service.change_root(state_root);
    }
}

#[derive(Clone)]
pub struct ChainStateActorRef(pub Addr<ChainStateActor>);

impl Into<Addr<ChainStateActor>> for ChainStateActorRef {
    fn into(self) -> Addr<ChainStateActor> {
        self.0
    }
}

impl Into<ChainStateActorRef> for Addr<ChainStateActor> {
    fn into(self) -> ChainStateActorRef {
        ChainStateActorRef(self)
    }
}

#[async_trait::async_trait]
impl ChainStateAsyncService for ChainStateActorRef {
    async fn get(self, access_path: AccessPath) -> Result<Option<Vec<u8>>> {
        let response = self
            .0
            .send(StateRequest::Get(access_path))
            .await
            .map_err(Into::<Error>::into)??;
        if let StateResponse::State(state) = response {
            Ok(state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_with_proof(self, access_path: AccessPath) -> Result<StateWithProof> {
        let response = self
            .0
            .send(StateRequest::GetWithProof(access_path))
            .await
            .map_err(Into::<Error>::into)??;
        if let StateResponse::StateWithProof(state) = response {
            Ok(*state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_account_state(self, address: AccountAddress) -> Result<Option<AccountState>> {
        let response = self
            .0
            .send(StateRequest::GetAccountState(address))
            .await
            .map_err(Into::<Error>::into)??;
        if let StateResponse::AccountState(state) = response {
            Ok(state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn state_root(self) -> Result<HashValue> {
        let response = self
            .0
            .send(StateRequest::StateRoot())
            .await
            .map_err(Into::<Error>::into)??;
        if let StateResponse::StateRoot(root) = response {
            Ok(root)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_with_proof_by_root(
        self,
        access_path: AccessPath,
        state_root: HashValue,
    ) -> Result<StateWithProof> {
        let response = self
            .0
            .send(StateRequest::GetWithProofByRoot(access_path, state_root))
            .await
            .map_err(Into::<Error>::into)??;
        if let StateResponse::StateWithProof(state) = response {
            Ok(*state)
        } else {
            panic!("Unexpect response type.")
        }
    }

    async fn get_account_state_by_root(
        self,
        account_address: AccountAddress,
        state_root: HashValue,
    ) -> Result<Option<AccountState>> {
        let response = self
            .0
            .send(StateRequest::GetAccountStateByRoot(
                account_address,
                state_root,
            ))
            .await
            .map_err(Into::<Error>::into)??;
        if let StateResponse::AccountState(state) = response {
            Ok(state)
        } else {
            panic!("Unexpect response type.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_bus::BusActor;
    use starcoin_state_api::mock::MockStateNodeStore;

    #[stest::test]
    async fn test_actor_launch() -> Result<()> {
        let mock_store = Arc::new(MockStateNodeStore::new());
        let bus_actor = BusActor::launch();
        let actor = ChainStateActor::launch(bus_actor, mock_store, None)?;
        let _state_root = actor.state_root().await?;
        //assert!(account.is_some());
        Ok(())
    }
}
