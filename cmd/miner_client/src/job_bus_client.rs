use crate::{BlockHeaderExtra, JobClient, MintBlockEvent, SealEvent};
use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
use futures::stream::StreamExt;
use starcoin_miner::{MinerService, SubmitSealRequest};
use starcoin_service_registry::bus::{Bus, BusService};
use starcoin_service_registry::ServiceRef;
use starcoin_time_service::TimeService;
use std::sync::Arc;

#[derive(Clone)]
pub struct JobBusClient {
    bus: ServiceRef<BusService>,
    time_service: Arc<dyn TimeService>,
    miner_service: ServiceRef<MinerService>,
}

impl JobBusClient {
    pub fn new(
        miner_service: ServiceRef<MinerService>,
        bus: ServiceRef<BusService>,
        time_service: Arc<dyn TimeService>,
    ) -> Self {
        Self {
            bus,
            time_service,
            miner_service,
        }
    }
}

#[async_trait]
impl JobClient for JobBusClient {
    async fn subscribe(&self) -> Result<BoxStream<'static, MintBlockEvent>> {
        let bus = self.bus.clone();
        bus.channel::<MintBlockEvent>().await.map(|s| s.boxed())
    }

    async fn submit_seal(&self, seal: SealEvent) -> Result<()> {
        let extra = match &seal.extra {
            None => BlockHeaderExtra::default(),
            Some(extra) => extra.extra,
        };
        let _ = self
            .miner_service
            .send(SubmitSealRequest::new(seal.minting_blob, seal.nonce, extra))
            .await??;
        Ok(())
    }

    fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }
}
