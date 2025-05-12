use crate::CliState;
use scmd::{CmdContext, CustomCommand};
use starcoin_config::StarcoinOpt;

pub mod accept_token_cmd;

pub fn add_command_vm2(
    context: CmdContext<CliState, StarcoinOpt>,
) -> CmdContext<CliState, StarcoinOpt> {
    context.command(
        CustomCommand::with_name("account").subcommand(accept_token_cmd::AcceptTokenCommand),
    )
}
