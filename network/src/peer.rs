// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;

pub struct PeerInfo {
    pub max_block_number: u64,
}

pub struct Peer {
    peer_info: PeerInfo,
}

pub struct PeerActor {
    peer: Peer,
}

impl Actor for PeerActor {
    type Context = Context<Self>;
}
