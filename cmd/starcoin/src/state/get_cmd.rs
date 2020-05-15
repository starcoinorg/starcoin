// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "get")]
pub struct GetOpt {
    #[structopt(name = "account_address")]
    account_address: AccountAddress,
}

pub struct GetCommand;

impl CommandAction for GetCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetOpt;
    type ReturnItem = String;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let state = client
            .state_get(AccessPath::new_for_account(opt.account_address))?
            .ok_or_else(|| {
                format_err!(
                    "Account with address {} state not exist.",
                    opt.account_address
                )
            })?;
        Ok(hex::encode(state))
    }
}
