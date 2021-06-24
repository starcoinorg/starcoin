// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::{ExecuteResultView, TransactionOptions};
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_types::account_address::AccountAddress;
use starcoin_vm_types::token::stc::STC_TOKEN_CODE;
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::transaction::TransactionPayload;
use structopt::StructOpt;

/// Transfer token's command, this command will send a transaction to the chain.
#[derive(Debug, StructOpt)]
#[structopt(name = "transfer")]
pub struct TransferOpt {
    #[structopt(short = "r", long = "receiver", alias = "receipt")]
    /// transfer to, accept address (start with 0x) or receipt_identifier (start with stc)
    receiver: AccountAddress,

    #[structopt(short = "k", name = "public-key", long = "public-key")]
    /// this option is deprecated
    _public_key: Option<String>,

    #[structopt(short = "v")]
    amount: u128,

    #[structopt(
        short = "t",
        long = "token-code",
        name = "token-code",
        help = "token's code to transfer, for example: 0x1::STC::STC, default is STC."
    )]
    token_code: Option<TokenCode>,

    #[structopt(flatten)]
    transaction_opts: TransactionOptions,
}

pub struct TransferCommand;

impl CommandAction for TransferCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = TransferOpt;
    type ReturnItem = ExecuteResultView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let receiver_address = opt.receiver;

        let token_code = opt
            .token_code
            .clone()
            .unwrap_or_else(|| STC_TOKEN_CODE.clone());
        let script_function = starcoin_executor::encode_transfer_script_by_token_code(
            receiver_address,
            opt.amount,
            token_code,
        );
        ctx.state().build_and_execute_transaction(
            opt.transaction_opts.clone(),
            TransactionPayload::ScriptFunction(script_function),
        )
    }
}
