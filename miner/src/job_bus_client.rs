use crate::{MintBlockEvent, SubmitSealEvent};
use anyhow::Result;
use crypto::HashValue;
use futures::executor::block_on;
use futures::stream::BoxStream;
use futures::stream::StreamExt;
use starcoin_miner_client::JobClient;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::ServiceRef;
use starcoin_vm_types::genesis_config::ConsensusStrategy;

#[derive(Clone)]
pub struct JobBusClient {
    bus: ServiceRef<BusService>,
    consensus: ConsensusStrategy,
}

impl JobBusClient {
    pub fn new(bus: ServiceRef<BusService>, consensus: ConsensusStrategy) -> Self {
        Self { bus, consensus }
    }
}

impl JobClient for JobBusClient {
    fn subscribe(&self) -> Result<BoxStream<'static, MintBlockEvent>> {
        let bus = self.bus.clone();
        block_on(async move { bus.channel::<MintBlockEvent>().await.map(|s| s.boxed()) })
    }

    fn submit_seal(&self, pow_hash: HashValue, nonce: u64) -> Result<()> {
        self.bus.broadcast(SubmitSealEvent::new(pow_hash, nonce))?;
        Ok(())
    }

    fn consensus(&self) -> Result<ConsensusStrategy> {
        Ok(self.consensus)
    }
}
