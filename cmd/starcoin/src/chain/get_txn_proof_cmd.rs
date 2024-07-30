// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{ensure, format_err, Result};
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use serde::Serialize;
use starcoin_chain_api::TransactionInfoWithProof;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::{StrView, TransactionInfoWithProofView};
use starcoin_vm_types::access_path::AccessPath;
use std::convert::TryInto;

/// Get transaction proof
#[derive(Debug, Parser)]
#[clap(name = "get-txn-proof")]
pub struct GetTransactionProofOpt {
    /// The block hash for get txn accumulator root
    #[clap(name = "block-hash", long, short = 'b')]
    block_hash: HashValue,
    #[clap(name = "transaction-global-index", long, short = 't')]
    transaction_global_index: u64,
    #[clap(name = "event-index", long, short = 'e')]
    event_index: Option<u64>,
    #[clap(name = "access-path", long, short = 'a')]
    access_path: Option<AccessPath>,
    /// Return raw hex string of transaction info proof
    #[clap(name = "raw", long)]
    raw: bool,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ViewOrRaw {
    View(TransactionInfoWithProofView),
    Raw(StrView<Vec<u8>>),
}

impl Serialize for ViewOrRaw {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::View(v) => v.serialize(serializer),
            Self::Raw(v) => v.serialize(serializer),
        }
    }
}

pub struct GetTransactionProofCommand;

impl CommandAction for GetTransactionProofCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetTransactionProofOpt;
    type ReturnItem = ViewOrRaw;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let block = client
            .chain_get_block_by_hash(opt.block_hash, None)?
            .ok_or_else(|| format_err!("Can not find block by hash: {}", opt.block_hash))?;
        let (txn_proof, result) = if opt.raw {
            let txn_proof_hex = client
                .chain_get_transaction_proof_raw(
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
            let txn_proof =
                bcs_ext::from_bytes::<TransactionInfoWithProof>(txn_proof_hex.0.as_slice())?;

            (txn_proof, ViewOrRaw::Raw(txn_proof_hex))
        } else {
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
            let txn_proof: TransactionInfoWithProof = txn_proof_view.clone().try_into()?;
            (txn_proof, ViewOrRaw::View(txn_proof_view))
        };
        ensure!(txn_proof.transaction_info.transaction_global_index == opt.transaction_global_index,
            "response transaction_info.transaction_global_index({}) do not match with opt transaction_global_index({}).",
            opt.transaction_global_index, txn_proof.transaction_info.transaction_global_index);
        txn_proof.verify(
            block.header.txn_accumulator_root,
            opt.transaction_global_index,
            opt.event_index,
            opt.access_path.clone(),
        )?;
        Ok(result)
    }
}
