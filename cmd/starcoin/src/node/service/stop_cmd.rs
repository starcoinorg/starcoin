// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_service_registry::ServiceInfo;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "stop")]
pub struct StopOpt {
    #[structopt(name = "name")]
    name: String,
}

pub struct StopCommand;

impl CommandAction for StopCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = StopOpt;
    type ReturnItem = Vec<ServiceInfo>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let handle = ctx.state().node_handle();
        match handle {
            Some(handle) => {
                handle.stop_service(ctx.opt().name.clone())?;
                handle.list_service()
            }
            None => {
                bail!("Remote attached console not support node service command.");
            }
        }
    }
}
