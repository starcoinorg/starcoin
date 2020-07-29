// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_txpool_api::TxPoolStatus;
use structopt::StructOpt;

///Get tx pool status
#[derive(Debug, StructOpt)]
#[structopt(name = "txpool_status")]
pub struct TxPoolStatusOpt {}

pub struct TxPoolStatusCommand;

impl CommandAction for TxPoolStatusCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = TxPoolStatusOpt;
    type ReturnItem = TxPoolStatus;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        Ok(client.txpool_status()?)
    }
}
