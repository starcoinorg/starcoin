// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_dag::consensusdb::consensus_state::DagStateView;

/// Get block info by number
#[derive(Debug, Parser)]
#[clap(name = "get-dag-state", alias = "get_dag_state")]
pub struct GetDagStateOpt {}

pub struct GetDagStateCommand;

impl CommandAction for GetDagStateCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetDagStateOpt;
    type ReturnItem = DagStateView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        ctx.state().client().get_dag_state()
    }
}
