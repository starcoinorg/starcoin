// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::chain::GetTransactionOption;
use starcoin_vm2_rpc_api::transaction_view2::TransactionView2;

/// Get transaction by txn hash or block hash and txn idx in the block
#[derive(Debug, Parser)]
#[clap(name = "get-txn", alias = "get_txn")]
pub struct GetTransactionOpt {
    #[clap(name = "txn-hash")]
    /// txn hash
    txn_hash: Option<HashValue>,

    #[clap(name = "block-hash", long, required_unless_present = "txn-hash")]
    /// block hash which include the txn, only used when txn-hash is missing.
    block_hash: Option<HashValue>,
    #[clap(name = "idx", long, required_unless_present = "txn-hash")]
    /// the index(start from 0) of the txn in the block
    idx: Option<u64>,
}

pub struct GetTransactionCommand;

impl CommandAction for GetTransactionCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetTransactionOpt;
    type ReturnItem = Option<TransactionView2>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        match &opt.txn_hash {
            Some(txn_hash) => Ok(client
                .chain_get_transaction2(*txn_hash, Some(GetTransactionOption { decode: true }))?),
            None => {
                let block_hash = opt.block_hash.expect("block-hash exists");
                let idx = opt.idx.expect("idx exists");
                match client.chain_get_txn_info_by_block_and_index2(block_hash, idx)? {
                    Some(info) => Ok(client.chain_get_transaction2(
                        info.transaction_hash,
                        Some(GetTransactionOption { decode: true }),
                    )?),
                    None => Ok(None),
                }
            }
        }
    }
}
