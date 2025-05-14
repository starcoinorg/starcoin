use crate::CliState;
use scmd::{CmdContext, CustomCommand};
use starcoin_config::StarcoinOpt;

mod account;

pub fn add_command_vm2(
    context: CmdContext<CliState, StarcoinOpt>,
) -> CmdContext<CliState, StarcoinOpt> {
    context.command(
        CustomCommand::with_name("account")
            .subcommand(account::accept_token_cmd::AcceptTokenCommand),
    )
}
