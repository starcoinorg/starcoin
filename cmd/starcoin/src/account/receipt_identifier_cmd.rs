// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use serde::Deserialize;
use serde::Serialize;
use starcoin_types::account_address::AccountAddress;
use structopt::StructOpt;

/// Encode or decode the receipt_identifier
#[derive(Debug, StructOpt)]
#[structopt(name = "receipt-identifier")]
pub struct ReceiptIdentifierOpt {
    #[structopt(name = "address_or_receipt")]
    address_or_receipt: AccountAddress,
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

        Ok(ReceiptIdentifierData {
            address: opt.address_or_receipt.to_hex(),
            receipt_identifier: opt.address_or_receipt.to_bech32(),
        })
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct ReceiptIdentifierData {
    pub address: String,
    pub receipt_identifier: String,
}
