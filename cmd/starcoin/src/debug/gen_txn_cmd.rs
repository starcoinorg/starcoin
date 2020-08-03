// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, format_err, Result};
use scmd::{CommandAction, ExecContext};
use serde::{Deserialize, Serialize};
use starcoin_executor::DEFAULT_MAX_GAS_AMOUNT;
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_transaction_builder::DEFAULT_EXPIRATION_TIME;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_vm_types::transaction::helpers::get_current_timestamp;
use std::time::Duration;
use structopt::StructOpt;

///Generate transfer transaction and submit to chain, only work for dev network.
///Use the default account to sender transaction.
#[derive(Debug, StructOpt)]
#[structopt(name = "gen_txn")]
pub struct GenTxnOpt {
    ///Default account's password
    #[structopt(short = "p", default_value = "")]
    password: String,

    ///Txn count
    #[structopt(short = "c", default_value = "1")]
    count: usize,

    ///Transfer to the address that must already be in the wallet.
    ///If absent, a new account is generated.
    #[structopt(short = "t", conflicts_with("random"))]
    to: Option<AccountAddress>,

    ///Random generate new account, those accounts will be discarded.
    #[structopt(short = "r")]
    random: bool,

    ///Transfer amount of every transaction, default is 1.
    #[structopt(short = "v", default_value = "1")]
    amount: u128,
}

pub struct GenTxnCommand;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct GenerateResult {
    count: usize,
    total_amount: u128,
    submit_success: usize,
    submit_fail: usize,
    //TODO add execute result and gas_used after watch api provider.
}

impl CommandAction for GenTxnCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GenTxnOpt;
    type ReturnItem = GenerateResult;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let client = ctx.state().client();
        let net = ctx.state().net();
        if !net.is_test() && !net.is_dev() && !net.is_halley() {
            bail!("This command only work for test or dev or halley network");
        }
        let account_provider: Box<dyn Fn() -> (AccountAddress, Vec<u8>)> = if opt.random {
            Box::new(|| -> (AccountAddress, Vec<u8>) {
                let auth_key = AuthenticationKey::random();
                (auth_key.derived_address(), auth_key.prefix().to_vec())
            })
        } else {
            let to_account = match opt.to {
                Some(to) => client.wallet_get(to),
                None => Ok(None),
            }
            .and_then(|to| match to {
                Some(to) => Ok(to),
                None => client.wallet_create("".to_string()),
            })?;
            let address = to_account.address;
            let auth_prefix = AuthenticationKey::ed25519(&to_account.public_key)
                .prefix()
                .to_vec();
            Box::new(move || -> (AccountAddress, Vec<u8>) { (address, auth_prefix.clone()) })
        };
        let sender = client
            .wallet_default()?
            .expect("Default account should exist.");
        client.wallet_unlock(
            sender.address,
            opt.password.clone(),
            Duration::from_secs(3600),
        )?;
        let chain_state_reader = RemoteStateReader::new(client);
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let account_resource = account_state_reader
            .get_account_resource(sender.address())?
            .ok_or_else(|| {
                format_err!(
                    "Can not find account on chain by address:{}",
                    sender.address()
                )
            })?;
        let sequence_number = account_resource.sequence_number();
        let mut gen_result = GenerateResult::default();
        gen_result.count = opt.count;
        for i in 0..opt.count {
            let (to, to_auth_key_prefix) = account_provider.as_ref()();

            let raw_txn = starcoin_executor::build_transfer_txn(
                sender.address,
                to,
                to_auth_key_prefix,
                sequence_number + i as u64,
                opt.amount,
                1,
                DEFAULT_MAX_GAS_AMOUNT,
                get_current_timestamp() + DEFAULT_EXPIRATION_TIME,
                net.chain_id(),
            );
            gen_result.total_amount += opt.amount;
            let txn = client.wallet_sign_txn(raw_txn)?;
            let result = client.submit_transaction(txn.clone())?;
            if result.is_ok() {
                gen_result.submit_success += 1;
            } else {
                gen_result.submit_fail += 1;
            }
        }

        Ok(gen_result)
    }
}
