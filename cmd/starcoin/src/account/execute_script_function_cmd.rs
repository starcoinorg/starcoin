// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state_router::CliStateRouter;
use crate::view::{ExecuteResultView, TransactionOptions};
use crate::StarcoinOpt;
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::FunctionIdView;
use starcoin_types::transaction::{parse_transaction_argument_advance, TransactionArgument};
use starcoin_vm_types::transaction::{ScriptFunction, TransactionPayload};
use starcoin_vm_types::transaction_argument::convert_txn_args;
use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};

/// Execute a script function.
#[derive(Debug, Parser)]
#[clap(name = "execute-function")]
pub struct ExecuteScriptFunctionOpt {
    #[clap(
    short = 't',
    long = "type_tag",
    name = "type-tag",
    parse(try_from_str = parse_type_tag)
    )]
    /// type tags for the script
    type_tags: Option<Vec<TypeTag>>,

    #[clap(long = "arg", name = "transaction-args", parse(try_from_str = parse_transaction_argument_advance))]
    /// args for the script.
    args: Option<Vec<TransactionArgument>>,

    #[clap(flatten)]
    transaction_opts: TransactionOptions,

    #[clap(long = "function", name = "script-function")]
    /// script function to execute, example: 0x1::TransferScripts::peer_to_peer_v2
    script_function: FunctionIdView,
}

pub struct ExecuteScriptFunctionCmd;

impl CommandAction for ExecuteScriptFunctionCmd {
    type State = CliStateRouter;
    type GlobalOpt = StarcoinOpt;
    type Opt = ExecuteScriptFunctionOpt;
    type ReturnItem = ExecuteResultView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let type_tags = opt.type_tags.clone().unwrap_or_default();
        let args = opt.args.clone().unwrap_or_default();
        let script_function = opt.script_function.clone().0;
        ctx.state().build_and_execute_transaction(
            opt.transaction_opts.clone(),
            TransactionPayload::ScriptFunction(ScriptFunction::new(
                script_function.module,
                script_function.function,
                type_tags,
                convert_txn_args(&args),
            )),
        )
    }
}
