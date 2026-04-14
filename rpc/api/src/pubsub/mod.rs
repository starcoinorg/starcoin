// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! PUB-SUB rpc interface.

use anyhow::Result;
use jsonrpsee::core::SubscriptionResult;
use jsonrpsee::proc_macros::rpc;
use starcoin_crypto::HashValue;
use starcoin_types::system_events::MintBlockEvent;
use starcoin_vm2_types::view::TransactionEventView as TransactionEventView2;

use crate::types::pubsub::{self, EventFilter, EventFilterV2};
use crate::types::{BlockView, TransactionEventView};

#[rpc(client, server, namespace = "starcoin", namespace_separator = "_")]
pub trait StarcoinPubSubApi {
    #[subscription(
        name = "subscribeNewHeads",
        unsubscribe = "unsubscribeNewHeads",
        item = BlockView
    )]
    async fn subscribe_new_heads(&self) -> SubscriptionResult;

    #[subscription(
        name = "subscribeEvents",
        unsubscribe = "unsubscribeEvents",
        item = TransactionEventView
    )]
    async fn subscribe_events(&self, filter: EventFilter, decode: bool) -> SubscriptionResult;

    #[subscription(
        name = "subscribeEventsV2",
        unsubscribe = "unsubscribeEventsV2",
        item = TransactionEventView2
    )]
    async fn subscribe_events_v2(&self, filter: EventFilterV2, decode: bool) -> SubscriptionResult;

    #[subscription(
        name = "subscribeNewPendingTransactions",
        unsubscribe = "unsubscribeNewPendingTransactions",
        item = Vec<HashValue>
    )]
    async fn subscribe_new_pending_transactions(&self) -> SubscriptionResult;

    #[subscription(
        name = "subscribeNewMintBlock",
        unsubscribe = "unsubscribeNewMintBlock",
        item = MintBlockEvent
    )]
    async fn subscribe_new_mint_block(&self) -> SubscriptionResult;
}

/// Starcoin PUB-SUB rpc interface.
pub trait StarcoinPubSub {
    /// RPC Metadata
    type Metadata;

    /// Subscribe to Starcoin subscription.
    fn subscribe(&self, meta: Self::Metadata, kind: pubsub::Kind, params: Option<pubsub::Params>);

    /// Unsubscribe from existing Starcoin subscription.
    fn unsubscribe(&self, meta: Option<Self::Metadata>, id: String) -> Result<bool>;
}
