// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::CliState;
use anyhow::{format_err, Result};
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_config::StarcoinOpt;
use starcoin_vm2_account_api::AccountInfo;
use starcoin_vm2_types::account_address::AccountAddress;

/// Set or show the default account
#[derive(Debug, Parser, Default)]
#[clap(name = "default")]
pub struct DefaultOpt {
    #[clap(
        name = "account_address",
        help = "set default address to this, if not provided, display current default address"
    )]
    account_address: Option<AccountAddress>,
}

pub struct DefaultCommand;

impl CommandAction for DefaultCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = DefaultOpt;
    type ReturnItem = AccountInfo;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt: &DefaultOpt = ctx.opt();
        let account_client = ctx.state().vm2()?.account_client();
        match opt.account_address.as_ref() {
            None => {
                let default_account = account_client.get_default_account()?.ok_or_else(|| {
                    format_err!("Can not find default account, Please input from account.")
                })?;
                Ok(default_account)
            }
            Some(addr) => {
                let default_account = account_client.set_default_account(*addr)?;
                Ok(default_account)
            }
        }
    }
}
