// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;

pub struct PeerManager {}

pub struct PeerManagerActor {
    manager: PeerManager,
}

impl Actor for PeerManagerActor {
    type Context = Context<Self>;
}
