// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_types::block::BlockNumber;
use starcoin_types::stress_test::TPS;
use structopt::StructOpt;

/// Get tps for an epoch.
#[derive(Debug, StructOpt)]
#[structopt(name = "tps")]
pub struct TPSOpt {
    #[structopt(
        name = "number",
        long,
        short = "n",
        help = "block number, if absent return tps for the latest epoch"
    )]
    number: Option<BlockNumber>,
}

pub struct TPSCommand;

impl CommandAction for TPSCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = TPSOpt;
    type ReturnItem = TPS;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.tps(ctx.opt().number)
    }
}
