// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{view::TransactionOptions, view_vm2::ExecuteResultView, CliState, StarcoinOpt};
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_vm2_vm_types::{
    account_config::core_code_address,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    token::token_code::TokenCode,
    transaction::{EntryFunction, TransactionPayload},
};
use std::convert::TryInto;

/// Accept a new token, this operator will call 0x1::Account::accept_token function.
#[derive(Debug, Parser)]
#[clap(name = "accept-token", alias = "accept_token")]
pub struct AcceptTokenOpt {
    #[clap(flatten)]
    transaction_opts: TransactionOptions,

    #[clap(
        name = "token-code",
        help = "token's code to accept, for example:  0x1::DummyToken::DummyToken "
    )]
    token_code: TokenCode,
}

pub struct AcceptTokenCommand;

impl CommandAction for AcceptTokenCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = AcceptTokenOpt;
    type ReturnItem = ExecuteResultView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        ctx.state().vm2()?.build_and_execute_transaction(
            opt.transaction_opts.clone(),
            TransactionPayload::EntryFunction(EntryFunction::new(
                ModuleId::new(core_code_address(), Identifier::new("Account")?),
                Identifier::new("accept_token")?,
                vec![TypeTag::Struct(Box::new(
                    opt.token_code.clone().try_into()?,
                ))],
                vec![],
            )),
        )
    }
}
