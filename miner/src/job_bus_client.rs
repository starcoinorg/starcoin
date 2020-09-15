use crate::{MintBlockEvent, SubmitSealEvent};
use actix::Addr;
use anyhow::Result;
use bus::{Bus, BusActor};
use crypto::HashValue;
use futures::executor::block_on;
use futures::stream::BoxStream;
use futures::stream::StreamExt;
use starcoin_miner_client::JobClient;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use types::U256;

#[derive(Clone)]
pub struct JobBusClient {
    bus: Addr<BusActor>,
    consensus: ConsensusStrategy,
}

impl JobBusClient {
    pub fn new(bus: Addr<BusActor>, consensus: ConsensusStrategy) -> Self {
        Self { bus, consensus }
    }
}

impl JobClient for JobBusClient {
    fn subscribe(&self) -> Result<BoxStream<Result<(HashValue, U256)>>> {
        let bus = self.bus.clone();
        block_on(async move {
            let receiver = bus.channel::<MintBlockEvent>().await;
            receiver
                .map(|r| r.map(|b| Ok((b.header_hash, b.difficulty))))
                .map(|s| s.boxed())
        })
    }

    fn submit_seal(&self, pow_hash: HashValue, nonce: u64) -> Result<()> {
        let bus = self.bus.clone();
        block_on(async move { bus.broadcast(SubmitSealEvent::new(pow_hash, nonce)).await })
    }

    fn consensus(&self) -> Result<ConsensusStrategy> {
        Ok(self.consensus)
    }
}
