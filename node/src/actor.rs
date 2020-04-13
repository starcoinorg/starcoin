// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::message::NodeCommand;
use actix::prelude::*;
use anyhow::Result;
use starcoin_config::NodeConfig;
use starcoin_traits::Consensus;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::runtime::Handle;

pub struct NodeActor<C>
where
    C: Consensus + 'static,
{
    consensus: PhantomData<C>,
}

impl<C> NodeActor<C>
where
    C: Consensus,
{
    pub fn new(_config: Arc<NodeConfig>, _handle: Handle) -> Self {
        Self {
            consensus: Default::default(),
        }
    }
}

impl<C> Actor for NodeActor<C>
where
    C: Consensus,
{
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {}
}

impl<C> Handler<NodeCommand> for NodeActor<C>
where
    C: Consensus,
{
    type Result = Result<()>;

    fn handle(&mut self, _msg: NodeCommand, _ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}

#[derive(Clone)]
pub struct NodeRef<C>(pub Addr<NodeActor<C>>)
where
    C: Consensus + 'static;

impl<C> Into<Addr<NodeActor<C>>> for NodeRef<C>
where
    C: Consensus,
{
    fn into(self) -> Addr<NodeActor<C>> {
        self.0
    }
}

impl<C> Into<NodeRef<C>> for Addr<NodeActor<C>>
where
    C: Consensus,
{
    fn into(self) -> NodeRef<C> {
        NodeRef(self)
    }
}
