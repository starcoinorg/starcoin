// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::{HashValue, PlainCryptoHash};
use starcoin_executor::DEFAULT_EXPIRATION_TIME;
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_types::token::token_code::TokenCode;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "accept_token")]
pub struct AcceptTokenOpt {
    #[structopt(short = "s")]
    /// if `sender` is absent, use default account.
    sender: Option<AccountAddress>,

    #[structopt(
        short = "g",
        name = "max-gas-amount",
        default_value = "1000000",
        help = "max gas used to deploy the module"
    )]
    max_gas_amount: u64,
    #[structopt(
        short = "p",
        long = "gas-price",
        name = "price of gas",
        default_value = "1",
        help = "gas price used to deploy the module"
    )]
    gas_price: u64,

    #[structopt(
        name = "token-code",
        help = "token's code, for example: 0x1::STC::STC, default is STC"
    )]
    token_code: TokenCode,

    #[structopt(
        short = "b",
        name = "blocking-mode",
        long = "blocking",
        help = "blocking wait txn mined"
    )]
    blocking: bool,
}

pub struct AcceptTokenCommand;

impl CommandAction for AcceptTokenCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = AcceptTokenOpt;
    type ReturnItem = HashValue;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let client = ctx.state().client();
        let node_info = client.node_info()?;
        let sender = ctx.state().wallet_account_or_default(opt.sender)?;
        let chain_state_reader = RemoteStateReader::new(client);
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let account_resource = account_state_reader.get_account_resource(&sender.address)?;

        if account_resource.is_none() {
            bail!(
                "account of module address {} not exists on chain",
                sender.address
            );
        }

        let account_resource = account_resource.unwrap();

        let accept_token_txn = starcoin_executor::build_accept_token_txn(
            sender.address,
            account_resource.sequence_number(),
            opt.gas_price,
            opt.max_gas_amount,
            opt.token_code.clone(),
            node_info.now + DEFAULT_EXPIRATION_TIME,
            ctx.state().net().chain_id(),
        );

        let signed_txn = client.wallet_sign_txn(accept_token_txn)?;
        let txn_hash = signed_txn.crypto_hash();
        let succ = client.submit_transaction(signed_txn)?;
        if let Err(e) = succ {
            bail!("execute-txn is reject by node, reason: {}", &e)
        }
        println!("txn {:#x} submitted.", txn_hash);

        if opt.blocking {
            ctx.state().watch_txn(txn_hash)?;
        }

        Ok(txn_hash)
    }
}
