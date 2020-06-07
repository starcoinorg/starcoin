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
#[structopt(name = "get_txn_info")]
pub struct GetOpt {
    #[structopt(name = "block_id")]
    block_id: String,
    #[structopt(name = "index of the txn in the block")]
    idx: u64,
}

pub struct GetTransactionCommand;

impl CommandAction for GetTransactionCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetOpt;
    type ReturnItem = TransactionInfo;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let block_id = HashValue::from_hex(&opt.block_id).unwrap();
        let idx = opt.idx;
        let transaction_info = client.chain_get_transaction_info(block_id, idx)?;

        Ok(transaction_info)
    }
}
