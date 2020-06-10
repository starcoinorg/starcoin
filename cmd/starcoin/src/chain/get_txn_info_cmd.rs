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
pub struct GetTransactionInfoOpt {
    #[structopt(name = "block-hash")]
    block_hash: HashValue,
    #[structopt(name = "idx", help = "the index(start from 0) of the txn in the block")]
    idx: u64,
}

pub struct GetTransactionInfoCommand;

impl CommandAction for GetTransactionInfoCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetTransactionInfoOpt;
    type ReturnItem = Option<TransactionInfo>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let transaction_info =
            client.chain_get_txn_info_by_block_and_index(opt.block_hash, opt.idx)?;

        Ok(transaction_info)
    }
}
