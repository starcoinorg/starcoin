// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use anyhow::Result;
use bus::{Broadcast, BusActor};
use config::NodeConfig;
use crypto::HashValue;
use logger::prelude::*;
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Mutex;
use traits::Consensus;
use types::{block::BlockTemplate, system_events::SystemEvents, U256};

#[derive(Clone)]
pub struct Miner<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    state: Arc<Mutex<Option<MineCtx>>>,
    bus: Addr<BusActor>,
    config: Arc<NodeConfig>,
    phantom: PhantomData<C>,
}

#[derive(Clone)]
pub struct MineCtx {
    header_hash: HashValue,
    block_template: BlockTemplate,
    difficulty: U256,
}

impl MineCtx {
    pub fn new(block_template: BlockTemplate, difficulty: U256) -> MineCtx {
        let header_hash = block_template.parent_hash;
        MineCtx {
            header_hash,
            block_template,
            difficulty,
        }
    }
}

impl<C> Miner<C>
where
    C: Consensus + Sync + Send + 'static + Clone,
{
    pub fn new(bus: Addr<BusActor>, config: Arc<NodeConfig>) -> Miner<C> {
        Self {
            state: Arc::new(Mutex::new(None)),
            bus,
            config,
            phantom: PhantomData,
        }
    }
    pub fn set_mint_job(&mut self, t: MineCtx) {
        let mut state = self.state.lock().unwrap();
        *state = Some(t)
    }

    pub fn get_mint_job(&mut self) -> String {
        let state = self.state.lock().unwrap();
        let x = state.as_ref().unwrap().to_owned();
        format!(r#"["{:x}","{:x}"]"#, x.header_hash, x.difficulty)
    }

    pub fn submit(&self, payload: String) -> Result<()> {
        let state = self.state.lock().unwrap();
        let block_template = state.as_ref().unwrap().block_template.clone();
        let payload = hex::decode(payload).unwrap();
        let consensus_header = match C::ConsensusHeader::try_from(payload) {
            Ok(h) => h,
            Err(_) => return Err(anyhow::anyhow!("Invalid payload submit")),
        };
        let difficulty = state.as_ref().unwrap().difficulty;
        let block = block_template.into_block(consensus_header, difficulty);
        info!("Miner new block: {:?}", block);
        self.bus.do_send(Broadcast {
            msg: SystemEvents::MinedBlock(Arc::new(block)),
        });
        Ok(())
    }
}
