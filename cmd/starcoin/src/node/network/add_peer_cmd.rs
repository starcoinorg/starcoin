// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "add_peer")]
///Add a known peer
pub struct AddPeerOpt {
    #[structopt(name = "peer")]
    /// format: multiaddr/p2p/peer_id
    peer: String,
}

pub struct AddPeerCommand;

impl CommandAction for AddPeerCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = AddPeerOpt;
    type ReturnItem = ();

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.network_add_peer(ctx.opt().peer.clone())
    }
}
