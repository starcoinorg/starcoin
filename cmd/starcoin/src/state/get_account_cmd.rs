// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_types::account_address::AccountAddress;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "get_account")]
pub struct GetOpt {
    #[structopt(name = "account_address")]
    account_address: AccountAddress,
}

pub struct GetAccountCommand;

impl CommandAction for GetAccountCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetOpt;
    type ReturnItem = Vec<HashValue>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let account_state =
            client
                .state_get_account_state(opt.account_address)?
                .ok_or(format_err!(
                    "Account with address {} state not exist.",
                    opt.account_address
                ))?;

        let mut result = vec![];
        for hash in account_state.storage_roots() {
            result.push(hash.unwrap());
        }
        Ok(result)
    }
}
