// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::AddressOrReceipt;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use serde::Deserialize;
use serde::Serialize;
use starcoin_account_api::AccountPublicKey;
use starcoin_crypto::ValidCryptoMaterialStringExt;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::receipt_identifier::ReceiptIdentifier;
use starcoin_types::transaction::authenticator::AuthenticationKey;
use structopt::StructOpt;

/// Encode or decode the receipt_identifier
#[derive(Debug, StructOpt)]
#[structopt(name = "receipt-identifier")]
pub struct ReceiptIdentifierOpt {
    #[structopt(name = "address_or_receipt")]
    address_or_receipt: AddressOrReceipt,

    #[structopt(short = "k")]
    /// When encode address to receipt_identifier, use public_key to generate auth_key
    public_key: Option<String>,
}

pub struct ReceiptIdentifierCommand;

impl CommandAction for ReceiptIdentifierCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = ReceiptIdentifierOpt;
    type ReturnItem = ReceiptIdentifierData;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        match opt.address_or_receipt {
            AddressOrReceipt::Address(address) => {
                let auth_key = opt
                    .public_key
                    .as_ref()
                    .map(|pubkey| AccountPublicKey::from_encoded_string(pubkey.as_str()))
                    .transpose()?
                    .map(|pubkey| pubkey.authentication_key());
                let receipt_identifier = ReceiptIdentifier::v1(address, auth_key);
                Ok(ReceiptIdentifierData {
                    address,
                    auth_key,
                    receipt_identifier,
                })
            }
            AddressOrReceipt::Receipt(receipt_identifier) => Ok(ReceiptIdentifierData {
                address: receipt_identifier.address(),
                auth_key: receipt_identifier.auth_key().cloned(),
                receipt_identifier,
            }),
        }
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct ReceiptIdentifierData {
    pub address: AccountAddress,
    pub auth_key: Option<AuthenticationKey>,
    pub receipt_identifier: ReceiptIdentifier,
}
