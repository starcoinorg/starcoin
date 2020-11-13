// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_types::peer_info::PeerId;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "connected_peers")]
pub struct ConnectedPeersOpt {}

pub struct ConnectedPeersCommand;

impl CommandAction for ConnectedPeersCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ConnectedPeersOpt;
    type ReturnItem = Vec<PeerId>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.network_connected_peers()
    }
}
