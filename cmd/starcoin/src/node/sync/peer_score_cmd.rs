// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_sync_api::PeerScoreResponse;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "peer_score")]
pub struct PeerScoreOpt {}

pub struct PeerScoreCommand;

impl CommandAction for PeerScoreCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = PeerScoreOpt;
    type ReturnItem = PeerScoreResponse;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.sync_peer_score()
    }
}
