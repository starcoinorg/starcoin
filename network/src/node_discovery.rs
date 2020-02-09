// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;

pub struct NodeDiscovery {}

pub struct NodeDiscoveryActor {
    discovery: NodeDiscovery,
}

impl Actor for NodeDiscoveryActor {
    type Context = Context<Self>;
}
