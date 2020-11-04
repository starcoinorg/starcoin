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
#[structopt(name = "get_txn_by_block")]
pub struct GetOpt {
    #[structopt(name = "hash", parse(try_from_str = HashValue::from_hex_literal))]
    hash: HashValue,
}

pub struct GetTxnByBlockCommand;

impl CommandAction for GetTxnByBlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetOpt;
    type ReturnItem = Vec<TransactionInfoView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let vec_transaction_info = client.chain_get_txn_by_block(opt.hash)?;

        Ok(vec_transaction_info
            .into_iter()
            .map(|t| t.into())
            .collect::<Vec<_>>())
    }
}
