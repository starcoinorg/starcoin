// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::NodeCommand;
use crate::node;
use actix::prelude::*;
use anyhow::Result;
use starcoin_config::NodeConfig;
use starcoin_logger::prelude::*;
use starcoin_traits::{Consensus, ConsensusHeader};
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::runtime::Handle;

pub struct NodeActor<C, H>
where
    C: Consensus + 'static,
    H: ConsensusHeader + 'static,
{
    config: Arc<NodeConfig>,
    handle: Handle,
    consensus: PhantomData<C>,
    consensus_header: PhantomData<H>,
}

impl<C, H> NodeActor<C, H>
where
    C: Consensus,
    H: ConsensusHeader,
{
    pub fn new(config: Arc<NodeConfig>, handle: Handle) -> Self {
        Self {
            config,
            handle,
            consensus: Default::default(),
            consensus_header: Default::default(),
        }
    }
}

impl<C, H> Actor for NodeActor<C, H>
where
    C: Consensus,
    H: ConsensusHeader,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        let config = self.config.clone();
        let handle = self.handle.clone();
        Arbiter::spawn(async {
            match node::start::<C, H>(config, handle).await {
                Err(e) => {
                    error!("Node start fail: {}, exist.", e);
                    System::current().stop();
                }
                _ => {}
            }
        });
    }
}

impl<C, H> Handler<NodeCommand> for NodeActor<C, H>
where
    C: Consensus,
    H: ConsensusHeader,
{
    type Result = Result<()>;

    fn handle(&mut self, _msg: NodeCommand, _ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}

#[derive(Clone)]
pub struct NodeRef<C, H>(pub Addr<NodeActor<C, H>>)
where
    C: Consensus + 'static,
    H: ConsensusHeader + 'static;

impl<C, H> Into<Addr<NodeActor<C, H>>> for NodeRef<C, H>
where
    C: Consensus,
    H: ConsensusHeader,
{
    fn into(self) -> Addr<NodeActor<C, H>> {
        self.0
    }
}

impl<C, H> Into<NodeRef<C, H>> for Addr<NodeActor<C, H>>
where
    C: Consensus,
    H: ConsensusHeader,
{
    fn into(self) -> NodeRef<C, H> {
        NodeRef(self)
    }
}
