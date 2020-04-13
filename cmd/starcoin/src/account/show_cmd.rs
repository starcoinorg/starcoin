// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "show")]
pub struct ShowOpt {
    #[structopt(short = "a")]
    address: AccountAddress,
}

pub struct ShowCommand {}

impl CommandAction for ShowCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ShowOpt;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<()> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let account = client.account_get(opt.address)?;
        match account {
            Some(account) => {
                let auth_key = AuthenticationKey::ed25519(&account.public_key);
                println!("account: {}", account.address);
                println!("is_default: {}", account.is_default);
                println!("public_key: {}", account.public_key);
                println!("authentication_key: {}", auth_key);
                println!(
                    "authentication_key_prefix: {}",
                    hex::encode(auth_key.prefix())
                );

                let chain_state_reader = RemoteStateReader::new(client);
                let account_state_reader = AccountStateReader::new(&chain_state_reader);
                if let Some(account_resource) =
                    account_state_reader.get_account_resource(account.address())?
                {
                    println!("On chain data");
                    println!("-----------------");
                    println!("sequence_number: {}", account_resource.sequence_number());
                    let balance = account_state_reader
                        .get_balance(account.address())?
                        .unwrap_or(0);
                    println!("balance: {}", balance);
                };
            }
            None => println!("Account with address {} not exist.", opt.address),
        }

        Ok(())
    }
}
