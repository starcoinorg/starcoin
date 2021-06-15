// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::TransactionInfoView;
use structopt::StructOpt;

/// Get transaction infos by block hash.
#[derive(Debug, StructOpt)]
#[structopt(name = "get-txn-infos")]
pub struct GetTxnInfosOpt {
    #[structopt(name = "block-hash")]
    block_hash: HashValue,
}

pub struct GetTxnInfosCommand;

impl CommandAction for GetTxnInfosCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetTxnInfosOpt;
    type ReturnItem = Vec<TransactionInfoView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let vec_transaction_info = client.chain_get_block_txn_infos(opt.block_hash)?;

        Ok(vec_transaction_info)
    }
}
