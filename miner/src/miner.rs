// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::{Chain, Result};
use bus::{Broadcast, BusActor};
use chain::ChainActor;
use consensus::{Consensus, ConsensusHeader};
use futures::channel::oneshot;
use std::marker::PhantomData;
use std::sync::Arc;
use traits::ChainReader;
use types::{system_events::SystemEvents, transaction::SignedUserTransaction};

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
    bus.do_send(Broadcast {
        msg: SystemEvents::MinedBlock(block),
    });
    println!("broadcast new block.");
    Ok(())
}
