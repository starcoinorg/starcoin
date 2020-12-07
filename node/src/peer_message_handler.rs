// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::metrics::NODE_METRICS;
use network_api::messages::{
    NotificationMessage, PeerCompactBlockMessage, PeerMessage, PeerTransactionsMessage,
};
use network_api::PeerMessageHandler;
use starcoin_block_relayer::BlockRelayer;
use starcoin_logger::prelude::*;
use starcoin_service_registry::ServiceRef;
use starcoin_txpool::TxPoolActorService;
use starcoin_types::time::duration_since_epoch;
use std::sync::mpsc::TrySendError;

pub struct NodePeerMessageHandler {
    txpool_service: ServiceRef<TxPoolActorService>,
    block_relayer: ServiceRef<BlockRelayer>,
}

impl NodePeerMessageHandler {
    pub fn new(
        txpool_service: ServiceRef<TxPoolActorService>,
        block_relayer: ServiceRef<BlockRelayer>,
    ) -> Self {
        Self {
            txpool_service,
            block_relayer,
        }
    }
}

impl PeerMessageHandler for NodePeerMessageHandler {
    fn handle_message(&self, peer_message: PeerMessage) {
        match peer_message.notification {
            NotificationMessage::Transactions(message) => {
                if let Err(e) = self
                    .txpool_service
                    .notify(PeerTransactionsMessage::new(peer_message.peer_id, message))
                {
                    match e {
                        TrySendError::Full(_) => {
                            warn!("Handle PeerTransaction error, TxPoolService is too busy.");
                        }
                        TrySendError::Disconnected(_) => {
                            error!("Handle PeerTransaction error, TxPoolService is shutdown.");
                        }
                    }
                }
            }
            NotificationMessage::CompactBlock(message) => {
                let header_time = message.compact_block.header.timestamp;
                NODE_METRICS.block_latency.observe(
                    duration_since_epoch()
                        .as_millis()
                        .saturating_sub(header_time as u128) as f64,
                );
                if let Err(e) = self
                    .block_relayer
                    .notify(PeerCompactBlockMessage::new(peer_message.peer_id, *message))
                {
                    match e {
                        TrySendError::Full(_) => {
                            warn!("Handle PeerCmpctBlock error, BlockRelayer is too busy.");
                        }
                        TrySendError::Disconnected(_) => {
                            error!("Handle PeerCmpctBlock error, BlockRelayer is shutdown.");
                        }
                    }
                }
            }
        }
    }
}
