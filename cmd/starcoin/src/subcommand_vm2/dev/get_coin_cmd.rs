// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{cli_state::CliState, view::TransactionOptions, StarcoinOpt};
use anyhow::{bail, Result};
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_vm2_types::{
    account_address::AccountAddress, account_config::STCUnit, view::TransactionInfoView,
};
use starcoin_vm2_vm_types::{
    account_config::association_address, account_config::G_STC_TOKEN_CODE,
    token::token_value::TokenValue,
};

use starcoin_types::account_address::AccountAddress as AccountAddressV1;
use starcoin_vm2_transaction_builder::encode_transfer_script_by_token_code;
use std::time::Duration;

/// Get stc to default account.
/// This command only available in dev network.
#[derive(Debug, Parser)]
#[clap(name = "get-coin", alias = "get_coin")]
pub struct GetCoinOpt {
    #[clap(short = 'v', default_value = "1STC")]
    /// the amount of stc, eg: 1STC
    amount: TokenValue<STCUnit>,
    #[clap(
        name = "no-blocking-mode",
        long = "no-blocking",
        help = "not blocking wait transaction(txn) mined"
    )]
    no_blocking: bool,

    #[clap(name = "address_or_receipt")]
    /// The account's address or receipt to send coin, if absent, send to the default account.
    address_or_receipt: Option<AccountAddress>,
}

pub struct GetCoinCommand;

impl CommandAction for GetCoinCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetCoinOpt;
    type ReturnItem = Option<TransactionInfoView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let state = ctx.state().vm2()?;
        let net = ctx.state().net();
        let account_client = ctx.state().vm2()?.account_client();
        let to = if let Some(account_address) = opt.address_or_receipt {
            account_address
        } else {
            ctx.state().vm2()?.default_account()?.address
        };

        let transaction_info = if net.is_test_or_dev() {
            let sender = association_address();
            let txn_opt = TransactionOptions {
                sender: Some(AccountAddressV1::new(sender.into_bytes())),
                blocking: !opt.no_blocking,
                ..Default::default()
            };
            account_client.unlock_account(sender, "".to_string(), Duration::from_secs(300))?;
            state
                .build_and_execute_transaction(
                    txn_opt,
                    encode_transfer_script_by_token_code(
                        to,
                        opt.amount.scaling(),
                        G_STC_TOKEN_CODE.clone(),
                    ),
                )?
                .get_transaction_info()
        } else {
            bail!(
                "The network {} is not support get-coin command, please go to https://faucet.starcoin.org/",
                net
            );
        };

        Ok(transaction_info)
    }
}
