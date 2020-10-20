use crate::JobClient;
use anyhow::Result;
use crypto::HashValue;
use futures::stream::BoxStream;
use futures::{stream::StreamExt, TryStreamExt};
use logger::prelude::*;
use starcoin_config::{RealTimeService, TimeService};
use starcoin_rpc_client::RpcClient;
use starcoin_types::system_events::MintBlockEvent;
use std::sync::Arc;

#[derive(Clone)]
pub struct JobRpcClient {
    rpc_client: Arc<RpcClient>,
    time_service: Arc<dyn TimeService>,
}

impl JobRpcClient {
    pub fn new(rpc_client: RpcClient) -> Self {
        Self {
            rpc_client: Arc::new(rpc_client),
            time_service: Arc::new(RealTimeService::new()),
        }
    }
}

impl JobClient for JobRpcClient {
    fn subscribe(&self) -> Result<BoxStream<'static, MintBlockEvent>> {
        let stream = self.rpc_client.subscribe_new_mint_blocks()?.into_stream();
        Ok(stream
            .filter_map(|r| async move {
                match r {
                    Ok(b) => Some(MintBlockEvent::new(
                        b.strategy,
                        b.minting_hash,
                        b.difficulty,
                    )),
                    Err(e) => {
                        error!("Failed to subscribe mint block:{}", e);
                        None
                    }
                }
            })
            .boxed())
    }

    fn submit_seal(&self, pow_hash: HashValue, nonce: u64) -> Result<()> {
        self.rpc_client.miner_submit(pow_hash, nonce)
    }

    fn time_service(&self) -> Arc<dyn TimeService> {
        self.time_service.clone()
    }
}
