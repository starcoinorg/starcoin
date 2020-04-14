// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state::CliState;
use crate::view::TransactionView;
use crate::StarcoinOpt;
use anyhow::{bail, format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_executor::{executor::Executor, TransactionExecutor};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::{
    account_config,
    transaction::{authenticator::AuthenticationKey, helpers::TransactionSigner},
};
use structopt::StructOpt;

/// Get coin to default account.
/// This command only available in dev network.
#[derive(Debug, StructOpt)]
#[structopt(name = "get_coin")]
pub struct GetCoinOpt {
    #[structopt(short = "v")]
    /// if amount absent, transfer 20% of association_address's balance.
    amount: Option<u64>,
}

pub struct GetCoinCommand;

impl CommandAction for GetCoinCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetCoinOpt;
    type ReturnItem = TransactionView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<TransactionView> {
        let opt = ctx.opt();
        let config = ctx.state().config();
        let net = config.net();
        if !net.is_dev() {
            bail!(
                "This command only available in dev network, current network is: {}",
                net
            );
        }
        let client = ctx.state().client();
        let to = client.wallet_default()?.ok_or(format_err!(
            "Can not find default account, Please create account first."
        ))?;

        let pre_mine_address = account_config::association_address();
        let chain_config = net.get_config();
        let pre_mine_config = chain_config
            .pre_mine_config
            .as_ref()
            .expect("Dev net pre mine config must exist.");

        let to_auth_key_prefix = AuthenticationKey::ed25519(&to.public_key).prefix();

        let chain_state_reader = RemoteStateReader::new(client);
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let account_resource = account_state_reader
            .get_account_resource(&pre_mine_address)?
            .expect(format!("pre mine address {} must exist", pre_mine_address).as_str());
        let balance = account_state_reader
            .get_balance(&pre_mine_address)?
            .expect(format!("pre mine address {} balance must exist", pre_mine_address).as_str());
        let amount = opt.amount.unwrap_or(balance * 20 / 100);
        let raw_txn = Executor::build_transfer_txn(
            pre_mine_address,
            vec![],
            to.address,
            to_auth_key_prefix.to_vec(),
            account_resource.sequence_number(),
            amount,
        );
        let txn = pre_mine_config.sign_txn(raw_txn)?;
        client.submit_transaction(txn.clone())?;
        Ok(txn.into())
    }
}
