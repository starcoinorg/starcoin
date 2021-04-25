// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use network_api::messages::{
    NotificationMessage, PeerCompactBlockMessage, PeerMessage, PeerTransactionsMessage,
};
use network_api::PeerMessageHandler;
use starcoin_block_relayer::BlockRelayer;
use starcoin_logger::prelude::*;
use starcoin_network::PeerAnnouncementMessage;
use starcoin_service_registry::ServiceRef;
use starcoin_sync::announcement::AnnouncementService;
use starcoin_txpool::TxPoolActorService;
use std::sync::mpsc::TrySendError;

pub struct NodePeerMessageHandler {
    txpool_service: ServiceRef<TxPoolActorService>,
    block_relayer: ServiceRef<BlockRelayer>,
    announcement_service: ServiceRef<AnnouncementService>,
}

impl NodePeerMessageHandler {
    pub fn new(
        txpool_service: ServiceRef<TxPoolActorService>,
        block_relayer: ServiceRef<BlockRelayer>,
        announcement_service: ServiceRef<AnnouncementService>,
    ) -> Self {
        Self {
            txpool_service,
            block_relayer,
            announcement_service,
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
            NotificationMessage::Announcement(message) => {
                if let Err(e) = self
                    .announcement_service
                    .notify(PeerAnnouncementMessage::new(peer_message.peer_id, message))
                {
                    match e {
                        TrySendError::Full(_) => {
                            warn!("Handle PeerAnnouncementMessage error, AnnouncementService is too busy.");
                        }
                        TrySendError::Disconnected(_) => {
                            error!("Handle PeerAnnouncementMessage error, AnnouncementService is shutdown.");
                        }
                    }
                }
            }
        }
    }
}
