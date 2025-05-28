// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{anyhow, Result};
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_dag::types::ghostdata::GhostdagData;

/// Get block info by number
#[derive(Debug, Parser)]
#[clap(name = "get-ghost-dag-data", alias = "get_ghost_dag_data")]
pub struct GetGhostdagData {
    #[clap(name = "block-hash")]
    id: HashValue,
}

pub struct GetGhostDagDataCommand;

impl CommandAction for GetGhostDagDataCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetGhostdagData;
    type ReturnItem = GhostdagData;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        ctx.state()
            .client()
            .get_ghost_dag_data(vec![ctx.opt().id])?
            .into_iter()
            .next()
            .flatten()
            .ok_or_else(|| anyhow!("Ghostdag data not found for block hash: {}", ctx.opt().id))
    }
}
