// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use network_p2p_types::network_state::NetworkState;
use scmd::{CommandAction, ExecContext};
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "state")]
pub struct StateOpt {}

pub struct StateCommand;

impl CommandAction for StateCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = StateOpt;
    type ReturnItem = NetworkState;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.network_state()
    }
}
