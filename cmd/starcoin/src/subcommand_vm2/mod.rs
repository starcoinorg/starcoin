use crate::CliState;
use scmd::{CmdContext, CustomCommand};
use starcoin_config::StarcoinOpt;

mod account;

pub fn add_command_vm2(
    context: CmdContext<CliState, StarcoinOpt>,
) -> CmdContext<CliState, StarcoinOpt> {
    context.command(
        CustomCommand::with_name("account")
            .subcommand(account::accept_token_cmd::AcceptTokenCommand)
            .subcommand(account::show_cmd::ShowCommand)
            .subcommand(account::execute_script_cmd::ExecuteScriptCommand)
            .subcommand(account::execute_script_function_cmd::ExecuteScriptFunctionCmd)
            .subcommand(account::unlock_cmd::UnlockCommand),
    )
}
