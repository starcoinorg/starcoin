// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cli_state::CliState, dev::sign_txn_helper::get_dao_config, view::TransactionOptions,
    view_vm2::ExecuteResultView, StarcoinOpt,
};
use anyhow::Result;
use clap::Parser;
use move_command_line_common::testing::format_diff;
use scmd::{CommandAction, ExecContext};
use starcoin_config::BuiltinNetworkID;
use starcoin_rpc_client::StateRootOption;
use starcoin_vm2_transaction_builder::build_vm_config_upgrade_proposal;
use starcoin_vm2_vm_runtime::starcoin_vm::StarcoinVM;
use starcoin_vm2_vm_types::on_chain_config::VMConfig;

/// Submit a VM config upgrade proposal
#[derive(Debug, Parser)]
#[clap(name = "vm-config-proposal", alias = "vm_config_proposal")]
#[allow(clippy::upper_case_acronyms)]
pub struct UpgradeVMConfigProposalOpt {
    #[clap(flatten)]
    transaction_opts: TransactionOptions,

    #[clap(short = 'n', name = "net", long = "net")]
    /// The network id for copy config
    net: BuiltinNetworkID,
}

#[allow(clippy::upper_case_acronyms)]
pub struct UpgradeVMConfigProposalCommand;

impl CommandAction for UpgradeVMConfigProposalCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = UpgradeVMConfigProposalOpt;
    type ReturnItem = ExecuteResultView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let genesis_config = opt.net.genesis_config().clone();
        let onchain_vm_config = {
            let client = ctx.state().client();
            let reader = client.state_reader2(StateRootOption::Latest)?;
            let mut vm = StarcoinVM::new(None, &reader);
            vm.load_configs(&reader)?;
            let gas_schedule = vm.get_gas_schedule()?.clone();
            VMConfig { gas_schedule }
        };
        let diff = {
            let current_config = serde_json::to_string_pretty(&onchain_vm_config)?;
            let new_config = serde_json::to_string_pretty(&genesis_config.vm_config)?;
            format_diff(current_config, new_config)
        };
        println!("{}", diff);

        let min_action_delay = get_dao_config(ctx.state())?.min_action_delay;
        let vm_config_upgrade_proposal =
            build_vm_config_upgrade_proposal(onchain_vm_config, min_action_delay);
        ctx.state()
            .vm2()?
            .build_and_execute_transaction(opt.transaction_opts.clone(), vm_config_upgrade_proposal)
    }
}
