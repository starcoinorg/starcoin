// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{ensure, format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_chain_api::TransactionInfoWithProof;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::TransactionInfoWithProofView;
use starcoin_vm_types::access_path::AccessPath;
use std::convert::TryInto;
use structopt::StructOpt;

/// Get transaction proof
#[derive(Debug, StructOpt)]
#[structopt(name = "get-txn-proof")]
pub struct GetTransactionProofOpt {
    /// The block hash for get txn accumulator root
    #[structopt(name = "block-hash", long, short = "b")]
    block_hash: HashValue,
    #[structopt(name = "transaction-global-index", long, short = "t")]
    transaction_global_index: u64,
    #[structopt(name = "event-index", long, short = "e")]
    event_index: Option<u64>,
    #[structopt(name = "access-path", long, short = "a")]
    access_path: Option<AccessPath>,
}

pub struct GetTransactionProofCommand;

impl CommandAction for GetTransactionProofCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetTransactionProofOpt;
    type ReturnItem = TransactionInfoWithProofView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let block = client
            .chain_get_block_by_hash(opt.block_hash, None)?
            .ok_or_else(|| format_err!("Can not find block by hash: {}", opt.block_hash))?;
        let txn_proof_view = client
            .chain_get_transaction_proof(
                opt.block_hash,
                opt.transaction_global_index,
                opt.event_index,
                opt.access_path.clone(),
            )?
            .ok_or_else(|| {
                format_err!(
                    "Can not find transaction info by global index:{}",
                    opt.transaction_global_index
                )
            })?;
        ensure!(txn_proof_view.transaction_info.transaction_global_index.0 == opt.transaction_global_index,
            "response transaction_info.transaction_global_index({}) do not match with opt transaction_global_index({}).",
            opt.transaction_global_index, txn_proof_view.transaction_info.transaction_global_index.0);
        let txn_proof: TransactionInfoWithProof = txn_proof_view.clone().try_into()?;
        txn_proof.verify(
            block.header.txn_accumulator_root,
            opt.transaction_global_index,
            opt.event_index,
            opt.access_path.clone(),
        )?;
        Ok(txn_proof_view)
    }
}
