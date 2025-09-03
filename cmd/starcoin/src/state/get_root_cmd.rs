// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;

#[derive(Debug, Parser)]
#[clap(name = "get-root", alias = "get_root")]
pub struct GetRootOpt {}

pub struct GetRootCommand;

impl CommandAction for GetRootCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetRootOpt;
    type ReturnItem = HashValue;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.state_get_state_root2()
    }
}
