// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::{StateRequest, StateResponse};
use crate::service::ChainStateServiceImpl;
use actix::prelude::*;
use anyhow::{Error, Result};
use starcoin_bus::{Bus, BusActor};
use starcoin_config::NodeConfig;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_state_api::{
    ChainStateAsyncService, ChainStateReader, ChainStateService, StateNodeStore, StateWithProof,
};
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_state::AccountState;
use starcoin_types::system_events::SystemEvents;
use std::sync::Arc;

pub struct ChainStateActor {
    bus: Addr<BusActor>,
    service: ChainStateServiceImpl,
}

impl ChainStateActor {
    pub fn launch(
        _config: Arc<NodeConfig>,
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
        let recipient = ctx.address().recipient::<SystemEvents>();
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
        };
        Ok(response)
    }
}

impl actix::Handler<SystemEvents> for ChainStateActor {
    type Result = ();

    fn handle(&mut self, msg: SystemEvents, _ctx: &mut Self::Context) -> Self::Result {
        if let SystemEvents::NewHeadBlock(block) = msg {
            let state_root = block.header().state_root();
            info!("ChainStateActor change StateRoot to : {:?}", state_root);
            self.service.change_root(state_root);
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_bus::BusActor;
    use starcoin_state_api::mock::MockStateNodeStore;

    #[stest::test]
    async fn test_actor_launch() -> Result<()> {
        let config = Arc::new(NodeConfig::random_for_test());
        let mock_store = Arc::new(MockStateNodeStore::new());
        let bus_actor = BusActor::launch();
        let actor = ChainStateActor::launch(config, bus_actor, mock_store, None)?;
        let _state_root = actor.state_root().await?;
        //assert!(account.is_some());
        Ok(())
    }
}
