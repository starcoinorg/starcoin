// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_service_registry::ServiceStatus;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "check")]
pub struct CheckOpt {
    #[structopt(name = "name")]
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
