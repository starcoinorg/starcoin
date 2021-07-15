// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::{ExecuteResultView, TransactionOptions};
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_vm_types::account_config::core_code_address;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::{ModuleId, TypeTag};
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::transaction::{ScriptFunction, TransactionPayload};
use std::convert::TryInto;
use structopt::StructOpt;

/// Accept a new token, this operator will call 0x1::Account::accept_token function.
#[derive(Debug, StructOpt)]
#[structopt(name = "accept-token", alias = "accept_token")]
pub struct AcceptTokenOpt {
    #[structopt(flatten)]
    transaction_opts: TransactionOptions,

    #[structopt(
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
        ctx.state().build_and_execute_transaction(
            opt.transaction_opts.clone(),
            TransactionPayload::ScriptFunction(ScriptFunction::new(
                ModuleId::new(core_code_address(), Identifier::new("Account").unwrap()),
                Identifier::new("accept_token").unwrap(),
                vec![TypeTag::Struct(opt.token_code.clone().try_into().unwrap())],
                vec![],
            )),
        )
    }
}
