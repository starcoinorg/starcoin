// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::messages::PeerMessage;

//TODO unify peer events
/// Handle broadcast message from peer
pub trait PeerMessageHandler: Send + Sync {
    fn handle_message(&self, peer_message: PeerMessage);
}
