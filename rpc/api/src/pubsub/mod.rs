// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

//! PUB-SUB rpc interface.

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use jsonrpc_pubsub::{typed, SubscriptionId};

use crate::types::pubsub;

/// Starcoin PUB-SUB rpc interface.
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

    /// Unsubscribe from existing Starcoin subscription.
    #[pubsub(
        subscription = "starcoin_subscription",
        unsubscribe,
        name = "starcoin_unsubscribe"
    )]
    fn unsubscribe(&self, meta: Option<Self::Metadata>, id: SubscriptionId) -> Result<bool>;
}
