// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_move_compiler::shared::Address;
use starcoin_move_compiler::{
    command_line::parse_address, compile_or_load_move_file,
};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::transaction::{parse_transaction_argument, TransactionArgument};
use starcoin_vm_runtime::data_cache::{RemoteStorage, TransactionDataCache};
use starcoin_vm_runtime::move_vm::MoveVM;
use starcoin_vm_runtime::starcoin_vm::{convert_txn_args, StarcoinVM};
use starcoin_vm_types::gas_schedule::{CostStrategy, GasAlgebra, GasUnits};
use starcoin_vm_types::transaction::{TransactionOutput, TransactionStatus};
use starcoin_vm_types::vm_error::{StatusCode, VMStatus};
use starcoin_vm_types::{language_storage::TypeTag, parser::parse_type_tag};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "dry-run")]
pub struct DryRunOpt {
    #[structopt(short = "s", long, parse(try_from_str = parse_address))]
    /// hex encoded string, like 0x1, 0x12
    sender: Option<Address>,

    #[structopt(short = "g", long, default_value = "1000000000")]
    initial_gas: u64,

    #[structopt(
    short = "t",
    long = "type_tag",
    name = "type-tag",
    parse(try_from_str = parse_type_tag)
    )]
    /// type arguments, can specify multi type_tag
    type_tags: Vec<TypeTag>,

    #[structopt(long = "arg", name = "transaction-args", parse(try_from_str = parse_transaction_argument))]
    /// script arguments
    args: Vec<TransactionArgument>,

    #[structopt(name = "move_file", parse(from_os_str))]
    /// bytecode file or move script source file
    move_file: PathBuf,

    #[structopt(name = "dependency_path", long = "dep")]
    /// path of dependency used to build, only used when using move source file
    deps: Vec<String>,
}

pub struct DryRunCommand;

impl CommandAction for DryRunCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = DryRunOpt;
    type ReturnItem = TransactionOutput;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();

        let sender = if let Some(sender) = ctx.opt().sender {
            AccountAddress::new(sender.to_u8())
        } else {
            ctx.state().default_account()?.address
        };

        let move_file_path = ctx.opt().move_file.clone();
        let mut deps = stdlib::stdlib_files();
        // add extra deps
        deps.append(&mut ctx.opt().deps.clone());
        let (bytecode, is_script) = compile_or_load_move_file(move_file_path, &deps, sender)?;

        let client = ctx.state().client();
        let chain_state_reader = RemoteStateReader::new(client);
        let remote_storage = RemoteStorage::new(&chain_state_reader);

        let mut data_cache = TransactionDataCache::new(&remote_storage);
        let move_vm = MoveVM::new();

        let cost_table = {
            let mut starcoin_vm = StarcoinVM::new();
            starcoin_vm.load_configs(&chain_state_reader);
            starcoin_vm.get_gas_schedule()?.clone()
        };
        let initial_gas = opt.initial_gas;
        let mut cost_strategy = CostStrategy::transaction(&cost_table, GasUnits::new(initial_gas));
        let vm_result = if is_script {
            move_vm.execute_script(
                bytecode,
                opt.type_tags.clone(),
                convert_txn_args(opt.args.as_slice()),
                sender,
                &mut data_cache,
                &mut cost_strategy,
            )
        } else {
            move_vm.publish_module(bytecode, sender, &mut data_cache)
        };
        let txn_status = match vm_result {
            Err(e) => e,
            Ok(_) => VMStatus::new(StatusCode::EXECUTED),
        };
        let write_set = data_cache.make_write_set()?;
        let events = data_cache.event_data();
        let gas_used = initial_gas - cost_strategy.remaining_gas().get();
        let output = TransactionOutput::new(
            write_set,
            events.to_vec(),
            gas_used,
            0,
            TransactionStatus::from(txn_status),
        );

        Ok(output)
    }
}
