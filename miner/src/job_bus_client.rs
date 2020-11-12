use crate::{MintBlockEvent, SubmitSealEvent};
use anyhow::Result;
use futures::executor::block_on;
use futures::stream::BoxStream;
use futures::stream::StreamExt;
use starcoin_miner_client::JobClient;
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::ServiceRef;
use starcoin_vm_types::time::TimeService;
use std::sync::Arc;

#[derive(Clone)]
pub struct JobBusClient {
    bus: ServiceRef<BusService>,
    time_service: Arc<dyn TimeService>,
}

impl JobBusClient {
    pub fn new(bus: ServiceRef<BusService>, time_service: Arc<dyn TimeService>) -> Self {
        Self { bus, time_service }
    }
}

impl JobClient for JobBusClient {
    fn subscribe(&self) -> Result<BoxStream<'static, MintBlockEvent>> {
        let bus = self.bus.clone();
        block_on(async move { bus.channel::<MintBlockEvent>().await.map(|s| s.boxed()) })
    }

    fn submit_seal(&self, minting_blob: Vec<u8>, nonce: u32) -> Result<()> {
        self.bus
            .broadcast(SubmitSealEvent::new(minting_blob, nonce))?;
        Ok(())
    }

    fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }
}
