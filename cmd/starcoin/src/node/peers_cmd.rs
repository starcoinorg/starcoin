// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::PeerInfoView;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "peers")]
pub struct PeersOpt {}

pub struct PeersCommand;

impl CommandAction for PeersCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = PeersOpt;
    type ReturnItem = Vec<PeerInfoView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let peers = client.node_peers()?;
        Ok(peers)
    }
}
