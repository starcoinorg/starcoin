use crate::JobClient;
use anyhow::Result;
use crypto::HashValue;
use futures::stream::BoxStream;
use futures::{stream::StreamExt, TryStreamExt};
use starcoin_rpc_client::RpcClient;
use starcoin_types::U256;

pub struct JobRpcClient {
    rpc_client: RpcClient,
}

impl JobRpcClient {
    pub fn new(rpc_client: RpcClient) -> Self {
        Self { rpc_client }
    }
}

impl JobClient for JobRpcClient {
    fn subscribe(&self) -> Result<BoxStream<Result<(HashValue, U256)>>> {
        self
            .rpc_client
            .subscribe_new_mint_blocks()
            .map(|stream| stream.map_ok(|b| (b.header_hash, b.difficulty)))
            .map(|s| s.boxed())
    }

    fn submit_seal(&self, pow_hash: HashValue, nonce: u64) -> Result<()> {
        self.rpc_client.miner_submit(pow_hash, nonce)
    }
}
