// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::CliState;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_config::StarcoinOpt;
use starcoin_vm2_account_api::AccountInfo;

/// List all accounts in the node.
#[derive(Debug, Parser, Default)]
#[clap(name = "list")]
pub struct ListOpt {}

pub struct ListCommand;

impl CommandAction for ListCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ListOpt;
    type ReturnItem = Vec<AccountInfo>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().vm2()?.account_client();
        let accounts = client.get_accounts()?;
        Ok(accounts)
    }
}
