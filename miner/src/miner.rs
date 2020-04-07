// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use bus::{Broadcast, BusActor};
use config::NodeConfig;
use consensus::ConsensusHeader;
use crypto::HashValue;
use logger::prelude::*;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::Mutex;
use types::{block::BlockTemplate, system_events::SystemEvents};

#[derive(Clone)]
pub struct Miner<H>
where
    H: ConsensusHeader + Sync + Send + 'static + Clone,
{
    state: Arc<Mutex<Option<MineCtx>>>,
    bus: Addr<BusActor>,
    config: Arc<NodeConfig>,
    phantom_h: PhantomData<H>,
}

#[derive(Clone)]
pub struct MineCtx {
    header_hash: HashValue,
    block_template: BlockTemplate,
}

impl MineCtx {
    pub fn new(block_template: BlockTemplate) -> MineCtx {
        let header_hash = block_template.clone().into_block_header(vec![]).id();

        MineCtx {
            header_hash,
            block_template,
        }
    }
}

impl<H> Miner<H>
where
    H: ConsensusHeader + Sync + Send + 'static + Clone,
{
    pub fn new(bus: Addr<BusActor>, config: Arc<NodeConfig>) -> Miner<H> {
        Self {
            state: Arc::new(Mutex::new(None)),
            bus,
            config,
            phantom_h: PhantomData,
        }
    }
    pub fn set_mint_job(&mut self, t: MineCtx) {
        let mut state = self.state.lock().unwrap();
        *state = Some(t)
    }

    pub fn get_mint_job(&mut self) -> String {
        let state = self.state.lock().unwrap();
        let x = state.as_ref().unwrap().to_owned();
        format!(
            r#"["{:x}","{:x}"]"#,
            x.header_hash, x.block_template.difficult
        )
    }

    pub fn submit(&self, payload: Vec<u8>) {
        let state = self.state.lock().unwrap();
        let block_template = state.as_ref().unwrap().block_template.clone();
        let consensus_header = match H::try_from(payload) {
            Ok(header) => header,
            _ => panic!("failed to parse header"),
        };
        let block = block_template.into_block(consensus_header);
        info!("Miner new block: {:?}", block);
        self.bus.do_send(Broadcast {
            msg: SystemEvents::MinedBlock(block),
        });
    }
}
