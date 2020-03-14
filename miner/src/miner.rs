// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::Result;
use bus::{Broadcast, BusActor};
use config::NodeConfig;
use consensus::Consensus;
use futures::channel::oneshot;
use logger::prelude::*;
use std::sync::Arc;
use traits::ChainReader;
use types::{system_events::SystemEvents, transaction::SignedUserTransaction};

pub fn mint<C>(
    config: Arc<NodeConfig>,
    txns: Vec<SignedUserTransaction>,
    chain: &dyn ChainReader,
    bus: Addr<BusActor>,
) -> Result<()>
where
    C: Consensus,
{
    let block_template = chain.create_block_template(txns)?;
    let (_sender, receiver) = oneshot::channel();
    // spawn a async task, maintain a task list, when new task coming, cancel old task.
    let block = C::create_block(config, chain, block_template, receiver)?;
    // fire SystemEvents::MinedBlock.
    //TODO handle result.
    info!("broadcast new block: {:?}.", block.header().id());
    bus.do_send(Broadcast {
        msg: SystemEvents::MinedBlock(block),
    });
    Ok(())
}
