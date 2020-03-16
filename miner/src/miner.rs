// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::{Chain, Result};
use bus::{Broadcast, BusActor};
use chain::ChainActor;
use consensus::{Consensus, ConsensusHeader, dummy::DummyHeader, consensus_impl};
use futures::channel::oneshot;
use logger::prelude::*;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::convert::TryFrom;
use traits::ChainReader;
use types::{system_events::SystemEvents, transaction::SignedUserTransaction, block::BlockTemplate};
use futures::channel::mpsc;
use futures::AsyncReadExt;
use futures::{Future, TryFutureExt};
use futures::prelude::*;
use sc_stratum::*;

#[derive(Clone)]
pub struct MineCtx {
    header_hash: Vec<u8>,
    block_template: BlockTemplate,
}

impl MineCtx {
    pub fn new(block_template: BlockTemplate) -> MineCtx {
        let header_hash = block_template.clone().into_block_header(DummyHeader {}).id().to_vec();
        MineCtx {
            header_hash,
            block_template,
        }
    }
}

#[derive(Clone)]
pub struct Miner {
    state: Arc<Mutex<Option<MineCtx>>>,
    bus: Addr<BusActor>,
}

impl Miner {
    pub fn new(bus: Addr<BusActor>) -> Miner {
        Self {
            state: Arc::new(Mutex::new(None)),
            bus,
        }
    }
    pub fn set_mint_job(&mut self, t: MineCtx) {
        let mut state = self.state.lock().unwrap();
        *state = Some(t)
    }

    pub fn get_mint_job(&mut self) -> String {
        let mut state = self.state.lock().unwrap();
        let x = state.as_ref().unwrap().to_owned();
        "".to_ascii_lowercase()
    }

    pub fn submit(&self, payload: Vec<u8>) {
        // verify payload
        // create block
        let state = self.state.lock().unwrap();
        let block_template = state.as_ref().unwrap().block_template.clone();
        let consensus_header = consensus_impl::ConsensusHeaderImpl::try_from(payload).unwrap();
        let block = block_template.into_block(consensus_header);
        // notify chain mined block
        println!("miner new block: {:?}", block);
        ///fire SystemEvents::MinedBlock.
        info!("broadcast new block: {:?}.", block.header().id());
        self.bus.do_send(Broadcast {
            msg: SystemEvents::MinedBlock(block),
        });
    }
}