// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_service_registry::ServiceInfo;
use std::thread::sleep;
use std::time::Duration;
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
        let client = ctx.state().client();
        client.node_stop_service(ctx.opt().name.clone())?;
        //wait service registry update service status.
        sleep(Duration::from_millis(3000));
        client.node_list_service()
    }
}
