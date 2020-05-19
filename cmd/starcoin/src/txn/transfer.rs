// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::TransactionView;
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::{ed25519::Ed25519PublicKey, ValidCryptoMaterialStringExt};
use starcoin_executor::executor::Executor;
use starcoin_executor::TransactionExecutor;
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "transfer")]
pub struct TransferOpt {
    #[structopt(short = "f")]
    /// if `from` is absent, use default account.
    from: Option<AccountAddress>,
    #[structopt(short = "t")]
    to: AccountAddress,
    #[structopt(short = "k")]
    /// if `to` account not exist on chain, must provide public_key of the account.
    public_key: Option<String>,
    #[structopt(short = "v")]
    amount: u64,
    #[structopt(
        short = "g",
        long = "max-gas",
        name = "max-gas-amount",
        default_value = "1000000",
        help = "max gas to use"
    )]
    max_gas_amount: u64,
    #[structopt(
        short = "p",
        long = "gas-price",
        name = "price of gas",
        default_value = "1",
        help = "gas price used"
    )]
    gas_price: u64,
}

pub struct TransferCommand;

impl CommandAction for TransferCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = TransferOpt;
    type ReturnItem = TransactionView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let sender = match opt.from {
            Some(from) => client
                .wallet_get(from)?
                .ok_or_else(|| format_err!("Can not find WalletAccount by address: {}", from))?,
            None => client.wallet_default()?.ok_or_else(|| {
                format_err!("Can not find default account, Please input from account.")
            })?,
        };
        let to = opt.to;

        let chain_state_reader = RemoteStateReader::new(client);
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let to_exist_on_chain = account_state_reader.get_account_resource(&to)?.is_some();
        let to_auth_key_prefix = if to_exist_on_chain {
            vec![]
        } else {
            opt.public_key
                .as_ref()
                .ok_or_else(|| {
                    format_err!(
                        "To account {} not exist on chain, please provide public_key",
                        to
                    )
                })
                .and_then(|pubkey_str| {
                    Ok(
                        AuthenticationKey::ed25519(&Ed25519PublicKey::from_encoded_string(
                            pubkey_str,
                        )?)
                        .prefix()
                        .to_vec(),
                    )
                })?
        };
        let account_resource = account_state_reader
            .get_account_resource(sender.address())?
            .ok_or_else(|| {
                format_err!(
                    "Can not find account on chain by address:{}",
                    sender.address()
                )
            })?;
        let raw_txn = Executor::build_transfer_txn(
            sender.address,
            to,
            to_auth_key_prefix,
            account_resource.sequence_number(),
            opt.amount,
            opt.gas_price,
            opt.max_gas_amount,
        );
        let txn = client.wallet_sign_txn(raw_txn)?;
        client.submit_transaction(txn.clone())?;
        Ok(txn.into())
    }
}
