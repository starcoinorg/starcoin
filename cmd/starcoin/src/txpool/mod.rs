// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::HashValue;
use starcoin_rpc_api::types::SignedUserTransactionV2View;
use starcoin_txpool_api::TxPoolStatus;
use starcoin_vm_types::account_address::AccountAddress;

/// Get txn data by its hash
#[derive(Debug, Parser)]
#[clap(name = "pending-txn")]
pub struct PendingTxnOpt {
    #[clap(name = "hash", help = "hash of the txn")]
    hash: HashValue,
}

pub struct PendingTxnCommand;

impl CommandAction for PendingTxnCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = PendingTxnOpt;
    type ReturnItem = Option<SignedUserTransactionV2View>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let txn = client.get_pending_txn_by_hash(ctx.opt().hash)?;

        Ok(txn)
    }
}

/// Get pending txns of sender
#[derive(Debug, Parser)]
#[clap(name = "pending-txns")]
pub struct PendingTxnsOpt {
    #[clap(name = "sender", help = "sender of pending txns")]
    sender: AccountAddress,
    #[clap(name = "max-len", long = "max", help = "max num to return")]
    max_len: Option<u32>,
}

pub struct PendingTxnsCommand;

impl CommandAction for PendingTxnsCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = PendingTxnsOpt;
    type ReturnItem = Vec<SignedUserTransactionV2View>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let txns = client.get_pending_txns_of_sender(ctx.opt().sender, ctx.opt().max_len)?;

        Ok(txns)
    }
}

///Get tx pool status
#[derive(Debug, Parser)]
#[clap(name = "status")]
pub struct TxPoolStatusOpt {}

pub struct TxPoolStatusCommand;

impl CommandAction for TxPoolStatusCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = TxPoolStatusOpt;
    type ReturnItem = TxPoolStatus;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        client.txpool_status()
    }
}
