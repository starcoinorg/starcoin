// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::AddressOrReceipt;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use serde::Deserialize;
use serde::Serialize;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::receipt_identifier::ReceiptIdentifier;
use structopt::StructOpt;

/// Encode or decode the receipt_identifier
#[derive(Debug, StructOpt)]
#[structopt(name = "receipt-identifier")]
pub struct ReceiptIdentifierOpt {
    #[structopt(name = "address_or_receipt")]
    address_or_receipt: AddressOrReceipt,
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
                let receipt_identifier = ReceiptIdentifier::v1(address);
                Ok(ReceiptIdentifierData {
                    address,
                    receipt_identifier,
                })
            }
            AddressOrReceipt::Receipt(receipt_identifier) => Ok(ReceiptIdentifierData {
                address: receipt_identifier.address(),
                receipt_identifier,
            }),
        }
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct ReceiptIdentifierData {
    pub address: AccountAddress,
    pub receipt_identifier: ReceiptIdentifier,
}
