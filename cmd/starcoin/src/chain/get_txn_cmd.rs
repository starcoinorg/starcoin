// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::TransactionInfoView;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "get_txn")]
pub struct GetOpt {
    #[structopt(name = "txn-hash", parse(try_from_str = HashValue::from_hex_literal))]
    hash: HashValue,
}

pub struct GetTransactionCommand;

impl CommandAction for GetTransactionCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetOpt;
    type ReturnItem = Option<TransactionInfoView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let transaction = client.chain_get_transaction_info(opt.hash)?;

        Ok(transaction.map(|t| t.into()))
    }
}
