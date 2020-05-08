use std::sync::Arc;

use jsonrpc_core;
use jsonrpc_pubsub::{PubSubMetadata, Session};

// use v1::types::Origin;

/// RPC methods metadata.
#[derive(Clone, Default, Debug)]
pub struct Metadata {
    // /// Request origin
    // pub origin: Origin,
    /// Request PubSub Session
    pub session: Option<Arc<Session>>,
}

impl jsonrpc_core::Metadata for Metadata {}
impl PubSubMetadata for Metadata {
    fn session(&self) -> Option<Arc<Session>> {
        self.session.clone()
    }
}
