use crate::JobClient;
use anyhow::Result;
use crypto::HashValue;
use futures::stream::BoxStream;
use futures::{stream::StreamExt, TryStreamExt};
use logger::prelude::*;
use starcoin_rpc_client::RpcClient;
use starcoin_types::genesis_config::ConsensusStrategy;
use starcoin_types::system_events::MintBlockEvent;

pub struct JobRpcClient {
    rpc_client: RpcClient,
}

impl JobRpcClient {
    pub fn new(rpc_client: RpcClient) -> Self {
        Self { rpc_client }
    }
}

impl JobClient for JobRpcClient {
    fn subscribe(&self) -> Result<BoxStream<'static, MintBlockEvent>> {
        let stream = self.rpc_client.subscribe_new_mint_blocks()?.into_stream();
        Ok(stream
            .filter_map(|r| async move {
                match r {
                    Ok(b) => Some(MintBlockEvent::new(b.minting_hash, b.difficulty)),
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

    fn consensus(&self) -> Result<ConsensusStrategy> {
        self.rpc_client
            .node_info()
            .map(|node_info| node_info.consensus)
    }
}
