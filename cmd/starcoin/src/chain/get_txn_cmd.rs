// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_types::transaction::TransactionInfo;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "get_txn")]
pub struct GetOpt {
    #[structopt(name = "txn-hash")]
    hash: HashValue,
}

pub struct GetTransactionCommand;

impl CommandAction for GetTransactionCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetOpt;
    type ReturnItem = Option<TransactionInfo>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let transaction = client.chain_get_transaction_info(opt.hash)?;

        Ok(transaction)
    }
}
