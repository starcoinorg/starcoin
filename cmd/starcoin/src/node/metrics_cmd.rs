// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use std::collections::HashMap;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "metrics")]
pub struct MetricsOpt {}

pub struct MetricsCommand;

impl CommandAction for MetricsCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = MetricsOpt;
    type ReturnItem = HashMap<String, String>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let node_info = client.node_metrics()?;
        Ok(node_info)
    }
}
