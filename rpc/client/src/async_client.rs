use crate::{map_err, RpcClient};
use futures::{TryStream, TryStreamExt};
use starcoin_account_api::AccountInfo;
use starcoin_rpc_api::node::NodeInfo;
use starcoin_rpc_api::types::MintedBlockView;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::system_events::MintBlockEvent;

impl RpcClient {
    pub async fn account_default_async(&self) -> anyhow::Result<Option<AccountInfo>> {
        self.call_rpc_async(|inner| inner.account_client.default())
            .await
            .map_err(map_err)
    }

    pub async fn account_get_async(
        &self,
        address: AccountAddress,
    ) -> anyhow::Result<Option<AccountInfo>> {
        self.call_rpc_async(|inner| inner.account_client.get(address))
            .await
            .map_err(map_err)
    }
    pub async fn node_info_async(&self) -> anyhow::Result<NodeInfo> {
        self.call_rpc_async(|inner| inner.node_client.info())
            .await
            .map_err(map_err)
    }

    pub async fn miner_submit_async(
        &self,
        minting_blob: String,
        nonce: u32,
        extra: String,
    ) -> anyhow::Result<MintedBlockView> {
        self.call_rpc_async(|inner| inner.miner_client.submit(minting_blob, nonce, extra))
            .await
            .map_err(map_err)
    }

    pub async fn subscribe_new_mint_blocks_async(
        &self,
    ) -> anyhow::Result<impl TryStream<Ok = MintBlockEvent, Error = anyhow::Error>> {
        self.call_rpc_async(|inner| async move {
            let res = inner.pubsub_client.subscribe_new_mint_block().await;
            res.map(|s| s.map_err(map_err))
        })
        .await
        .map_err(map_err)
    }
}
