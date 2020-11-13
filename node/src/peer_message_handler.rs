// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use network_api::PeerMessageHandler;
use starcoin_block_relayer::BlockRelayer;
use starcoin_block_relayer_api::PeerCmpctBlockEvent;
use starcoin_logger::prelude::*;
use starcoin_service_registry::ServiceRef;
use starcoin_tx_relay::PeerTransactions;
use starcoin_txpool::TxPoolActorService;
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
    fn handle_transaction(&self, transaction: PeerTransactions) {
        if let Err(e) = self.txpool_service.notify(transaction) {
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

    fn handle_block(&self, block: PeerCmpctBlockEvent) {
        if let Err(e) = self.block_relayer.notify(block) {
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
