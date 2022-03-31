// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_service_registry::ServiceInfo;

#[derive(Debug, Parser, Default)]
#[clap(name = "list")]
pub struct ListOpt {}

pub struct ListCommand;

impl CommandAction for ListCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ListOpt;
    type ReturnItem = Vec<ServiceInfo>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.node_list_service()
    }
}
