// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::{Chain, Result};
use bus::{Broadcast, BusActor};
use chain::ChainActor;
use consensus::{Consensus, ConsensusHeader};
use futures::channel::oneshot;
use logger::prelude::*;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use traits::ChainReader;
use types::{system_events::SystemEvents, transaction::SignedUserTransaction, block::BlockTemplate};
use futures::channel::mpsc;
use futures::AsyncReadExt;
use futures::{Future, TryFutureExt};
use futures::prelude::*;

#[derive(Clone)]
pub struct Miner {
    state: Arc<Mutex<Option<BlockTemplate>>>,
}

impl Miner {
    pub fn new() -> Miner {
        Self {
            state: Arc::new(Mutex::new(None))
        }
    }
    pub fn set_mint_job(&mut self, t: BlockTemplate) {
        let mut state = self.state.lock().unwrap();
        *state = Some(t)
    }

    pub fn get_mint_job(&mut self) ->BlockTemplate {
        println!("hello");
        let mut state = self.state.lock().unwrap();
        let x = state.as_ref().unwrap().to_owned();
        x
    }

    pub fn submit(&self, payload: Vec<u8>) {
        // verify payload
        // create block
        // notify chain mined block
    }
}


pub fn mint<C>(
    txns: Vec<SignedUserTransaction>,
    chain: &dyn ChainReader,
    bus: Addr<BusActor>,
) -> Result<()>
    where
        C: Consensus,
{
    let block_template = chain.create_block_template(txns)?;
    let (_sender, receiver) = oneshot::channel();
    /// spawn a async task, maintain a task list, when new task coming, cancel old task.
    let block = C::create_block(chain, block_template, receiver)?;
    //println!("miner new block: {:?}", block);
    ///fire SystemEvents::MinedBlock.
    //TODO handle result.
    info!("broadcast new block: {:?}.", block.header().id());
    bus.do_send(Broadcast {
        msg: SystemEvents::MinedBlock(block),
    });
    Ok(())
}
