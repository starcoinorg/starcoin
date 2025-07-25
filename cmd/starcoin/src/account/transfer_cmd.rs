// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::Parser;

use scmd::{CommandAction, ExecContext};
use starcoin_vm2_transaction_builder::encode_transfer_script_by_token_code;
use starcoin_vm2_types::account_address::AccountAddress;
use starcoin_vm2_vm_types::token::{stc::G_STC_TOKEN_CODE, token_code::TokenCode};

use crate::{
    cli_state::CliState, view::TransactionOptions, view_vm2::ExecuteResultView, StarcoinOpt,
};

/// Transfer token's command, this command will send a transaction to the chain.
#[derive(Debug, Parser)]
#[clap(name = "transfer")]
pub struct TransferOpt {
    #[clap(short = 'r', long = "receiver", alias = "receipt")]
    /// transfer to, accept address (start with 0x) or receipt_identifier (start with stc)
    receiver: AccountAddress,

    #[clap(short = 'k', name = "public-key", long = "public-key")]
    /// this option is deprecated
    _public_key: Option<String>,

    #[clap(short = 'v')]
    amount: u128,

    #[clap(
        short = 't',
        long = "token-code",
        name = "token-code",
        help = "token's code to transfer, for example: 0x1::STC::STC, default is STC."
    )]
    token_code: Option<TokenCode>,

    #[clap(flatten)]
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
            .unwrap_or_else(|| G_STC_TOKEN_CODE.clone());

        ctx.state().vm2()?.build_and_execute_transaction(
            opt.transaction_opts.clone(),
            encode_transfer_script_by_token_code(receiver_address, opt.amount, token_code),
        )
    }
}
