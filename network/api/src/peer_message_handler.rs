// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_block_relayer_api::PeerCmpctBlockEvent;
use starcoin_tx_relay::PeerTransactions;

//TODO unify peer events
/// Handle broadcast message from peer
pub trait PeerMessageHandler: Send + Sync {
    fn handle_transaction(&self, transaction: PeerTransactions);
    fn handle_block(&self, block: PeerCmpctBlockEvent);
}
