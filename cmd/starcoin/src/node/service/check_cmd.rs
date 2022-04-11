// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_service_registry::ServiceStatus;

#[derive(Debug, Parser, Default)]
#[clap(name = "check")]
pub struct CheckOpt {
    #[clap(name = "name")]
    name: String,
}

pub struct CheckCommand;

impl CommandAction for CheckCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = CheckOpt;
    type ReturnItem = ServiceStatus;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.node_check_service(ctx.opt().name.clone())
    }
}
