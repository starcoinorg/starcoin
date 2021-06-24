// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_executor::{DEFAULT_EXPIRATION_TIME, DEFAULT_MAX_GAS_AMOUNT};
use starcoin_rpc_api::types::SignedUserTransactionView;
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config;
use std::convert::TryInto;
use structopt::StructOpt;
use tokio::time::Duration;

/// Get stc to default account.
/// This command only available in dev network.
#[derive(Debug, StructOpt, Default)]
#[structopt(name = "get-coin", alias = "get_coin")]
pub struct GetCoinOpt {
    #[structopt(short = "v")]
    /// if amount absent, transfer 20% of association_address's balance.
    amount: Option<u128>,
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
    type ReturnItem = SignedUserTransactionView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let net = ctx.state().net();
        if !net.is_test_or_dev() {
            bail!(
                "This command only available in test or dev network, current network is: {}",
                net
            );
        }
        let client = ctx.state().client();
        let node_info = client.node_info()?;
        let to = ctx.state().get_account_or_default(opt.address_or_receipt)?;

        let association_address = account_config::association_address();
        let chain_state_reader = RemoteStateReader::new(client)?;
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let account_resource = account_state_reader
            .get_account_resource(&association_address)?
            .ok_or_else(|| {
                format_err!(
                    "association_address address {} must exist",
                    association_address
                )
            })?;
        let balance = account_state_reader
            .get_balance(&association_address)?
            .ok_or_else(|| {
                format_err!(
                    "association_address address {} balance must exist",
                    association_address
                )
            })?;
        let amount = opt.amount.unwrap_or(balance * 20 / 100);
        let raw_txn = starcoin_executor::build_transfer_txn(
            association_address,
            to.address,
            account_resource.sequence_number(),
            amount,
            1,
            DEFAULT_MAX_GAS_AMOUNT,
            node_info.now_seconds + DEFAULT_EXPIRATION_TIME,
            ctx.state().net().chain_id(),
        );
        client.account_unlock(
            association_address,
            "".to_string(),
            Duration::from_secs(300),
        )?;
        let txn = client.account_sign_txn(raw_txn)?;
        let id = txn.id();
        client.submit_transaction(txn.clone())?;
        if !opt.no_blocking {
            ctx.state().watch_txn(id)?;
        }
        txn.try_into()
    }
}
