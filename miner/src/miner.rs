// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::Result;
use bus::{Broadcast, BusActor};
use chain::ChainActor;
use consensus::{ChainReader, Consensus, ConsensusHeader};
use futures::channel::oneshot;
use std::marker::PhantomData;
use std::sync::Arc;
use types::system_events::SystemEvents;

pub(crate) struct Miner<C>
where
    C: Consensus,
{
    bus: Addr<BusActor>,
    chain: Arc<dyn ChainReader>,
    phantom: PhantomData<C>,
}

impl<C> Miner<C>
where
    C: Consensus,
{
    pub fn new(chain: Arc<dyn ChainReader>) -> Self {
        unimplemented!()
    }

    pub fn mint(&self) -> Result<()> {
        let block_template = self.chain.create_block_template()?;
        let (_sender, receiver) = oneshot::channel();
        /// spawn a async task, maintain a task list, when new task coming, cancel old task.
        let block = C::create_block(self.chain.as_ref(), block_template, receiver)?;
        ///fire SystemEvents::MinedBlock.
        //TODO handle result.
        self.bus.do_send(Broadcast {
            msg: SystemEvents::MinedBlock(block),
        });
        Ok(())
    }
}
