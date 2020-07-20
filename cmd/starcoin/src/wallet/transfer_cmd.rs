// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::TransactionView;
use crate::StarcoinOpt;
use anyhow::{bail, format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::{ed25519::Ed25519PublicKey, ValidCryptoMaterialStringExt};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::language_storage::TypeTag;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_vm_types::account_config::stc_type_tag;
use starcoin_vm_types::parser::parse_type_tag;
use structopt::StructOpt;

//TODO this command should be a wallet sub command?

#[derive(Debug, StructOpt)]
#[structopt(name = "transfer")]
pub struct TransferOpt {
    #[structopt(short = "s")]
    /// if `sender` is absent, use default account.
    sender: Option<AccountAddress>,
    #[structopt(short = "r")]
    receiver: AccountAddress,
    #[structopt(short = "k")]
    /// if `to` account not exist on chain, must provide public_key of the account.
    public_key: Option<String>,
    #[structopt(short = "v")]
    amount: u128,
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

    #[structopt(
    short = "c",
    long = "coin",
    name = "coin_type",
    help = "coin's type tag, for example: 0x0::STC::T, default is STC",
    parse(try_from_str = parse_type_tag)
    )]
    coin_type: Option<TypeTag>,

    #[structopt(
        short = "b",
        name = "blocking-mode",
        long = "blocking",
        help = "blocking wait txn mined"
    )]
    blocking: bool,
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
        let sender = match opt.sender {
            Some(from) => client
                .wallet_get(from)?
                .ok_or_else(|| format_err!("Can not find WalletAccount by address: {}", from))?,
            None => client.wallet_default()?.ok_or_else(|| {
                format_err!("Can not find default account, Please input from account.")
            })?,
        };
        let receiver = opt.receiver;

        let chain_state_reader = RemoteStateReader::new(client);
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let receiver_exist_on_chain = account_state_reader
            .get_account_resource(&receiver)?
            .is_some();
        let receiver_auth_key_prefix = if receiver_exist_on_chain {
            vec![]
        } else {
            opt.public_key
                .as_ref()
                .ok_or_else(|| {
                    format_err!(
                        "To account {} not exist on chain, please provide public_key",
                        receiver
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
        let coin_type = opt.coin_type.clone().unwrap_or_else(stc_type_tag);
        let raw_txn = starcoin_executor::build_transfer_txn_by_coin_type(
            sender.address,
            receiver,
            receiver_auth_key_prefix,
            account_resource.sequence_number(),
            opt.amount,
            opt.gas_price,
            opt.max_gas_amount,
            coin_type,
        );
        let txn = client.wallet_sign_txn(raw_txn)?;
        let succ = client.submit_transaction(txn.clone())?;
        if let Err(e) = succ {
            bail!("execute-txn is reject by node, reason: {}", &e)
        }

        if opt.blocking {
            ctx.state().watch_txn(txn.crypto_hash())?;
        }
        Ok(txn.into())
    }
}
