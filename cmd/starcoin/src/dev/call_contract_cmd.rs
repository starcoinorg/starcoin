// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_abi_decoder::DecodedMoveValue;
use starcoin_rpc_api::types::{ContractCall, FunctionIdView, TransactionArgumentView, TypeTagView};
use structopt::StructOpt;

/// Call Contract command
///  Some examples:
///  ``` shell
///  # 0x1::Block::get_current_block_number()
///  dev call --function 0x1::Block::current_block_number
///  # 0x1::Account::balance<0x1::STC::STC>(0x726098b70ba8aa2cc172af19af8804)
///  dev call --function 0x1::Account::balance -t 0x1::STC::STC --arg 0x726098b70ba8aa2cc172af19af8804
///  ```
#[derive(Debug, StructOpt)]
#[structopt(name = "call")]
pub struct CallContractOpt {
    #[structopt(long)]
    /// function to execute, example: 0x1::Block::current_block_number
    function: FunctionIdView,
    #[structopt(
        short = "t",
        long = "type_tag",
        name = "type-tag",
        help = "can specify multi type_tag"
    )]
    type_tags: Option<Vec<TypeTagView>>,

    #[structopt(
        long = "arg",
        name = "transaction-args",
        help = "can specify multi arg"
    )]
    args: Option<Vec<TransactionArgumentView>>,
}

pub struct CallContractCommand;

impl CommandAction for CallContractCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = CallContractOpt;
    type ReturnItem = Vec<DecodedMoveValue>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();

        let call = ContractCall {
            function_id: opt.function.clone(),
            type_args: opt.type_tags.clone().unwrap_or_default(),
            args: opt.args.clone().unwrap_or_default(),
        };

        let result = ctx.state().client().contract_call(call)?;
        Ok(result)
    }
}
