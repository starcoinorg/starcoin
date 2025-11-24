// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use serde::Serialize;
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::TransactionInfoView as _TransactionInfoViewVm1;
use starcoin_types::transaction::legacy::RichTransactionInfo;
use starcoin_vm2_types::view::TransactionInfoView as TransactionInfoViewVm2;

#[derive(Clone, Debug, Serialize)]
pub struct TransactionInfoViewVm1 {
    transaction_info_id: HashValue,
    rich_transaction_info: _TransactionInfoViewVm1,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum TransactionInfoView {
    Vm1(TransactionInfoViewVm1),
    Vm2(TransactionInfoViewVm2),
}

/// Get transaction info by txn hash or block hash and txn idx in the block
#[derive(Debug, Parser)]
#[clap(name = "get-txn-info", alias = "get_txn_info")]
pub struct GetTransactionInfoOpt {
    #[clap(name = "txn-hash")]
    /// txn hash
    txn_hash: Option<HashValue>,

    #[clap(name = "block-hash", long, required_unless_present = "txn-hash")]
    /// block hash which include the txn, only used when txn-hash is missing.
    block_hash: Option<HashValue>,
    #[clap(name = "idx", long, required_unless_present = "txn-hash")]
    /// the index(start from 0) of the txn in the block
    idx: Option<u64>,

    #[clap(long = "vm1")]
    /// Show transaction info returned by vm1 APIs.
    vm1: bool,
}

pub struct GetTransactionInfoCommand;

impl CommandAction for GetTransactionInfoCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GetTransactionInfoOpt;
    type ReturnItem = Option<TransactionInfoView>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let use_vm1 = opt.vm1;

        let result = match &opt.txn_hash {
            Some(txn_hash) => {
                if use_vm1 {
                    let mut txn_info_view_vm1 = None;
                    if let Some(txn_info_view) = client.chain_get_transaction_info(*txn_hash)? {
                        let txn_info_id: RichTransactionInfo = txn_info_view.clone().try_into()?;
                        txn_info_view_vm1 = Some(TransactionInfoViewVm1 {
                            transaction_info_id: txn_info_id.id(),
                            rich_transaction_info: txn_info_view,
                        });
                    }
                    txn_info_view_vm1.map(TransactionInfoView::Vm1)
                } else {
                    client
                        .chain_get_transaction_info2(*txn_hash)?
                        .map(TransactionInfoView::Vm2)
                }
            }
            None => {
                let block_hash = opt
                    .block_hash
                    .ok_or_else(|| anyhow::anyhow!("block-hash should exists"))?;
                let idx = opt.idx.ok_or_else(|| anyhow::anyhow!("idx exists"))?;
                if use_vm1 {
                    let mut txn_info_view_vm1 = None;
                    if let Some(txn_info_view) =
                        client.chain_get_txn_info_by_block_and_index(block_hash, idx)?
                    {
                        let txn_info_id: RichTransactionInfo = txn_info_view.clone().try_into()?;
                        txn_info_view_vm1 = Some(TransactionInfoViewVm1 {
                            transaction_info_id: txn_info_id.id(),
                            rich_transaction_info: txn_info_view,
                        });
                    }
                    txn_info_view_vm1.map(TransactionInfoView::Vm1)
                } else {
                    client
                        .chain_get_txn_info_by_block_and_index2(block_hash, idx)?
                        .map(TransactionInfoView::Vm2)
                }
            }
        };

        Ok(result)
    }
}
