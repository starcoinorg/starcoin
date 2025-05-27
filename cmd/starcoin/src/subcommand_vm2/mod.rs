use crate::CliState;
use scmd::{CmdContext, CustomCommand};
use starcoin_config::StarcoinOpt;

mod account;
mod dev;

pub fn add_command_vm2(
    context: CmdContext<CliState, StarcoinOpt>,
) -> CmdContext<CliState, StarcoinOpt> {
    context
        .command(
            CustomCommand::with_name("account2")
                .subcommand(account::accept_token_cmd::AcceptTokenCommand)
                .subcommand(account::show_cmd::ShowCommand)
                .subcommand(account::execute_script_cmd::ExecuteScriptCommand)
                .subcommand(account::execute_script_function_cmd::ExecuteScriptFunctionCmd)
                .subcommand(account::unlock_cmd::UnlockCommand),
        )
        .command(CustomCommand::with_name("dev2")
            .subcommand(dev::call_api_cmd::CallApiCommand)
            .subcommand(dev::call_contract_cmd::CallContractCommand)
            .subcommand(dev::compile_cmd::CompileCommand)
            .subcommand(dev::deploy_cmd::DeployCommand)
            .subcommand(dev::gen_block_cmd::GenBlockCommand)
            .subcommand(dev::get_coin_cmd::GetCoinCommand)
            .subcommand(dev::sleep_cmd::SleepCommand)
        )
}
