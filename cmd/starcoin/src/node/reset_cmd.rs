// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "reset")]
pub struct ResetOpt {
    #[structopt(name = "block-hash")]
    block_hash: HashValue,
}

pub struct ResetCommand;

impl CommandAction for ResetCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ResetOpt;
    type ReturnItem = ();

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.reset_node(ctx.opt().block_hash)?;
        Ok(())
    }
}
