// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::TransactionView;
use crate::StarcoinOpt;
use anyhow::{bail, format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::{ed25519::Ed25519PublicKey, ValidCryptoMaterialStringExt};
use starcoin_executor::DEFAULT_EXPIRATION_TIME;
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use starcoin_types::transaction::helpers::get_current_timestamp;
use starcoin_vm_types::token::stc::STC_TOKEN_CODE;
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::chain_config::ChainId;
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
        short = "t",
        long = "token-code",
        name = "token-code",
        help = "token's code, for example: 0x1::STC::STC, default is STC"
    )]
    token_code: Option<TokenCode>,

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
        let token_code = opt
            .token_code
            .clone()
            .unwrap_or_else(|| STC_TOKEN_CODE.clone());
        let raw_txn = starcoin_executor::build_transfer_txn_by_token_type(
            sender.address,
            receiver,
            receiver_auth_key_prefix,
            account_resource.sequence_number(),
            opt.amount,
            opt.gas_price,
            opt.max_gas_amount,
            token_code,
            get_current_timestamp() + DEFAULT_EXPIRATION_TIME,
            ChainId::id(ctx.state().net()),
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
