use scmd::{CmdContext, CustomCommand};
use starcoin_config::StarcoinOpt;
use crate::cli_state_router::CliStateRouter;

pub mod accept_token_cmd;

pub fn install_command_vm2(context: CmdContext<CliStateRouter, StarcoinOpt>) -> CmdContext<CliStateRouter, StarcoinOpt> {
    context.command(CustomCommand::with_name("account").subcommand(accept_token_cmd::AcceptTokenCommand))
}