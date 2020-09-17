// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

//! PUB-SUB rpc interface.

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use jsonrpc_pubsub::{typed, SubscriptionId};

use crate::types::pubsub;

/// Starcoin PUB-SUB rpc interface.
/// Example:
/// ```bash
/// $ netcat localhost 3030
/// {"id":1,"jsonrpc":"2.0","method":"starcoin_subscribe","params":["newPendingTransactions"]}
/// {"id":1,"jsonrpc":"2.0","method":"starcoin_subscribe","params":["events", {}]}
#[allow(clippy::needless_return)]
#[rpc(server)]
pub trait StarcoinPubSub {
    /// RPC Metadata
    type Metadata;

    /// Subscribe to Starcoin subscription.
    #[pubsub(
        subscription = "starcoin_subscription",
        subscribe,
        name = "starcoin_subscribe"
    )]
    fn subscribe(
        &self,
        meta: Self::Metadata,
        subscriber: typed::Subscriber<pubsub::Result>,
        kind: pubsub::Kind,
        params: Option<pubsub::Params>,
    );

    #[pubsub(
        subscription = "starcoin_subscription_hex",
        subscribe,
        name = "starcoin_subscribe_hex"
    )]
    fn subscribe_hex(
        &self,
        meta: Self::Metadata,
        subscriber: typed::Subscriber<pubsub::Result>,
        kind: String,
        params: Option<String>,
    );

    /// Unsubscribe from existing Starcoin subscription.
    #[pubsub(
        subscription = "starcoin_subscription",
        unsubscribe,
        name = "starcoin_unsubscribe"
    )]
    fn unsubscribe(&self, meta: Option<Self::Metadata>, id: SubscriptionId) -> Result<bool>;

    /// Unsubscribe from existing Starcoin subscription.
    #[pubsub(
        subscription = "starcoin_subscription_hex",
        unsubscribe,
        name = "starcoin_unsubscribe_hex"
    )]
    fn unsubscribe_hex(&self, meta: Option<Self::Metadata>, id: SubscriptionId) -> Result<bool>;
}
