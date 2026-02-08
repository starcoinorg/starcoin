// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! PUB-SUB rpc interface.

use anyhow::Result;

use crate::types::pubsub;

/// Starcoin PUB-SUB rpc interface.
pub trait StarcoinPubSub {
    /// RPC Metadata
    type Metadata;

    /// Subscribe to Starcoin subscription.
    fn subscribe(
        &self,
        meta: Self::Metadata,
        kind: pubsub::Kind,
        params: Option<pubsub::Params>,
    );

    /// Unsubscribe from existing Starcoin subscription.
    fn unsubscribe(&self, meta: Option<Self::Metadata>, id: String) -> Result<bool>;
}
