// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_resource_viewer::AnnotatedMoveValue;
use starcoin_rpc_api::types::ContractCall;
use starcoin_types::transaction::{parse_transaction_argument, TransactionArgument};
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};
use structopt::StructOpt;

/// Call Contract command
///  Some examples:
///  ``` shell
///  # 0x1::Block::get_current_block_number()
///  dev call --module-address 0x1 --module-name Block --func-name get_current_block_number
///  # 0x1::Account::balance<0x1::STC::STC>(0x726098b70ba8aa2cc172af19af8804)
///  dev call --func-name balance --module-address 0x1 --module-name Account -t 0x1::STC::STC --arg 0x726098b70ba8aa2cc172af19af8804
///  ```
#[derive(Debug, StructOpt)]
#[structopt(name = "call")]
pub struct CallContractOpt {
    #[structopt(
        long = "module-address",
        name = "module address",
        help = "hex encoded string, like 0x0, 0x1"
    )]
    module_address: AccountAddress,
    #[structopt(long)]
    module_name: String,
    #[structopt(long)]
    func_name: String,
    #[structopt(
    short = "t",
    long = "type_tag",
    name = "type-tag",
    help = "can specify multi type_tag",
    parse(try_from_str = parse_type_tag)
    )]
    type_tags: Option<Vec<TypeTag>>,

    #[structopt(long = "arg", name = "transaction-args", help = "can specify multi arg", parse(try_from_str = parse_transaction_argument))]
    args: Option<Vec<TransactionArgument>>,
}

pub struct CallContractCommand;

impl CommandAction for CallContractCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = CallContractOpt;
    type ReturnItem = Vec<AnnotatedMoveValue>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();

        let call = ContractCall {
            module_address: opt.module_address,
            module_name: opt.module_name.clone(),
            func: opt.func_name.clone(),
            type_args: opt.type_tags.clone().unwrap_or_default(),
            args: opt.args.clone().unwrap_or_default(),
        };

        let result = ctx.state().client().contract_call(call)?;
        Ok(result)
    }
}
