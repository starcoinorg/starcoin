// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::TransactionOptions;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::TransactionInfoView;
use starcoin_transaction_builder::encode_transfer_script_by_token_code;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config;
use starcoin_types::account_config::STCUnit;
use starcoin_vm_types::account_config::STC_TOKEN_CODE;
use starcoin_vm_types::token::token_value::TokenValue;
use starcoin_vm_types::transaction::TransactionPayload;
use std::time::Duration;
use structopt::StructOpt;

/// Get stc to default account.
/// This command only available in dev network.
#[derive(Debug, StructOpt)]
#[structopt(name = "get-coin", alias = "get_coin")]
pub struct GetCoinOpt {
    #[structopt(short = "v", default_value = "1STC")]
    /// the amount of stc, eg: 1STC
    amount: TokenValue<STCUnit>,
    #[structopt(
        name = "no-blocking-mode",
        long = "no-blocking",
        help = "not blocking wait txn mined"
    )]
    no_blocking: bool,

    #[structopt(name = "address_or_receipt")]
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
        let state = ctx.state();
        let net = ctx.state().net();
        let client = ctx.state().client();

        let to = ctx.state().get_account_or_default(opt.address_or_receipt)?;

        let transaction_info = if net.is_test_or_dev() {
            let sender = account_config::association_address();
            let txn_opt = TransactionOptions {
                sender: Some(sender),
                blocking: !opt.no_blocking,
                ..Default::default()
            };
            client.account_unlock(sender, "".to_string(), Duration::from_secs(300))?;
            state
                .build_and_execute_transaction(
                    txn_opt,
                    TransactionPayload::ScriptFunction(encode_transfer_script_by_token_code(
                        to.address,
                        opt.amount.scaling(),
                        STC_TOKEN_CODE.clone(),
                    )),
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
