// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use actix::prelude::*;
use config::NodeConfig;

pub struct Network {}

impl Network {
    pub fn new(_node_config: &NodeConfig) -> Self {
        Network {}
    }
}

impl Actor for Network {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {}
}
