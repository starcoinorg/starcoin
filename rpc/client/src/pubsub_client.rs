use futures::compat::Future01CompatExt;
use jsonrpc_core_client::*;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::{event::Event, pubsub::EventFilter, pubsub::Kind};
use starcoin_types::block::BlockHeader;

#[derive(Clone)]
pub struct PubSubClient {
    client: TypedClient,
}

impl From<RpcChannel> for PubSubClient {
    fn from(channel: RpcChannel) -> Self {
        PubSubClient {
            client: channel.into(),
        }
    }
}

impl PubSubClient {
    pub async fn subscribe_events(
        &self,
        filter: EventFilter,
    ) -> Result<TypedSubscriptionStream<Event>, RpcError> {
        self.client
            .subscribe(
                "starcoin_subscribe",
                (Kind::Events, filter),
                "starcoin_subscription",
                "starcoin_unsubscribe",
                "Event",
            )
            .compat()
            .await
    }
    pub async fn subscribe_new_block(
        &self,
    ) -> Result<TypedSubscriptionStream<BlockHeader>, RpcError> {
        self.client
            .subscribe(
                "starcoin_subscribe",
                vec![Kind::NewHeads],
                "starcoin_subscription",
                "starcoin_unsubscribe",
                "BlockHeader",
            )
            .compat()
            .await
    }
    pub async fn subscribe_new_transactions(
        &self,
    ) -> Result<TypedSubscriptionStream<Vec<HashValue>>, RpcError> {
        self.client
            .subscribe(
                "starcoin_subscribe",
                vec![Kind::NewPendingTransactions],
                "starcoin_subscription",
                "starcoin_unsubscribe",
                "Vec<HashValue>",
            )
            .compat()
            .await
    }
}
