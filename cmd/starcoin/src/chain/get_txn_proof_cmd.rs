// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{cli_state::CliState, StarcoinOpt};
use anyhow::{ensure, format_err, Result};
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use serde::Serialize;
use starcoin_chain_api::TransactionInfoWithProof;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::TransactionInfoWithProofView;
use starcoin_types::multi_access_path::MultiAccessPath;
use starcoin_vm2_types::view::StrView;

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
    access_path: Option<MultiAccessPath>,
    /// Return raw hex string of transaction info proof
    #[clap(name = "raw", long)]
    raw: bool,
    #[clap(name = "final-transaction-info-id", long, short = 'f')]
    final_transaction_info_id: HashValue,
    #[clap(name = "final-transaction-info-index", long, short = 'i')]
    final_transaction_info_index: u64,
    #[clap(name = "final-access-path", long, short = 'c')]
    final_access_path: Option<MultiAccessPath>,
    #[clap(name = "final-state-root", long, short = 'n')]
    final_state_root: Option<HashValue>,
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
            ViewOrRaw::View(v) => v.serialize(serializer),
            ViewOrRaw::Raw(v) => v.serialize(serializer),
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
        
        // Determine VM type from access_path (if provided)
        if let Some(ref access_path) = opt.access_path {
            let is_vm1 = matches!(access_path, MultiAccessPath::VM1(_));
            
            // Verify transaction exists in the correct VM type's transaction list
            let txn_infos = if is_vm1 {
                client.chain_get_block_txn_infos(opt.block_hash)?
                    .into_iter()
                    .map(|info| info.transaction_global_index.0)
                    .collect::<Vec<_>>()
            } else {
                client.chain_get_block_txn_infos2(opt.block_hash)?
                    .into_iter()
                    .map(|info| info.transaction_global_index.0)
                    .collect::<Vec<_>>()
            };
            
            ensure!(
                txn_infos.contains(&opt.transaction_global_index),
                "Transaction with global index {} not found in block {} for VM type {}",
                opt.transaction_global_index,
                opt.block_hash,
                if is_vm1 { "VM1" } else { "VM2" }
            );
        }
        
        if let Some(ref final_access_path) = opt.final_access_path {
            let is_vm1 = matches!(final_access_path, MultiAccessPath::VM1(_));
            
            let final_txn_infos = if is_vm1 {
                client.chain_get_block_txn_infos(opt.block_hash)?
                    .into_iter()
                    .map(|info| info.transaction_global_index.0)
                    .collect::<Vec<_>>()
            } else {
                client.chain_get_block_txn_infos2(opt.block_hash)?
                    .into_iter()
                    .map(|info| info.transaction_global_index.0)
                    .collect::<Vec<_>>()
            };
            
            ensure!(
                final_txn_infos.contains(&opt.final_transaction_info_index),
                "Transaction with global index {} not found in block {} for VM type {}",
                opt.final_transaction_info_index,
                opt.block_hash,
                if is_vm1 { "VM1" } else { "VM2" }
            );
        }
        
        // Extract VM2 AccessPath for RPC call (RPC only supports VM2)
        let access_path_v2 = opt.access_path.as_ref().and_then(|multi_path| {
            multi_path.clone().to_v2()
        });
        
        let (txn_proof, result) = if opt.raw {
            let txn_proof_hex = client
                .chain_get_transaction_proof2_raw(
                    opt.block_hash,
                    opt.transaction_global_index,
                    opt.event_index,
                    access_path_v2,
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
                .chain_get_transaction_proof2(
                    opt.block_hash,
                    opt.transaction_global_index,
                    opt.event_index,
                    access_path_v2,
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
            opt.final_transaction_info_index,
            opt.final_transaction_info_id,
            opt.event_index,
            opt.access_path.clone(),
            opt.final_access_path.clone(),
            opt.final_state_root,
        )?;
        Ok(result)
    }
}
