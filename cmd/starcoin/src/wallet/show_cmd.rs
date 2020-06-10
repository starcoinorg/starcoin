// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::AccountWithStateView;
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "show")]
pub struct ShowOpt {
    #[structopt(name = "account_address")]
    account_address: Option<AccountAddress>,
}

pub struct ShowCommand;

impl CommandAction for ShowCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ShowOpt;
    type ReturnItem = AccountWithStateView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let account_address = if let Some(account_address) = opt.account_address {
            account_address
        } else {
            let default_account = client
                .wallet_default()?
                .ok_or_else(|| format_err!("Default account should exist."))?;
            default_account.address
        };
        let account = client
            .wallet_get(account_address)?
            .ok_or_else(|| format_err!("Account with address {} not exist.", account_address))?;

        let chain_state_reader = RemoteStateReader::new(client);
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let sequence_number = account_state_reader
            .get_account_resource(account.address())?
            .map(|res| res.sequence_number());

        let balances = account_state_reader.get_balances(account.address())?;

        let auth_key_prefix = hex::encode(AuthenticationKey::ed25519(&account.public_key).prefix());
        Ok(AccountWithStateView {
            auth_key_prefix,
            account,
            sequence_number,
            balances,
        })
    }
}
