// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use jsonrpc_pubsub::{PubSubMetadata, Session};
use std::sync::Arc;

/// RPC methods metadata.
#[derive(Clone, Default, Debug)]
pub struct Metadata {
    // /// Request origin
    // pub origin: Origin,
    /// Request PubSub Session
    pub session: Option<Arc<Session>>,
    pub user: Option<String>,
}

impl Metadata {
    pub fn new(session: Arc<Session>) -> Self {
        Self {
            session: Some(session),
            user: None,
        }
    }
}

impl jsonrpc_core::Metadata for Metadata {}
impl PubSubMetadata for Metadata {
    fn session(&self) -> Option<Arc<Session>> {
        self.session.clone()
    }
}
