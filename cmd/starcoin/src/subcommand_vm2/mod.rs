use crate::CliState;
use scmd::{CmdContext, CustomCommand};
use starcoin_config::StarcoinOpt;

pub mod account;
pub mod dev;

use account::*;
use dev::*;

pub fn add_command_vm2(
    context: CmdContext<CliState, StarcoinOpt>,
) -> CmdContext<CliState, StarcoinOpt> {
    context
        .command(
            CustomCommand::with_name("account2")
                .subcommand(AcceptTokenCommand)
                .subcommand(ChangePasswordCmd)
                .subcommand(CreateCommand)
                .subcommand(DefaultCommand)
                .subcommand(ExecuteScriptCommand)
                .subcommand(ExecuteScriptFunctionCmd)
                .subcommand(ExportCommand)
                .subcommand(ImportCommand)
                .subcommand(ImportMultisigCommand)
                .subcommand(ImportReadonlyCommand)
                .subcommand(ListCommand)
                .subcommand(LockCommand)
                .subcommand(RemoveCommand)
                .subcommand(ShowCommand)
                .subcommand(TransferCommand)
                .subcommand(UnlockCommand),
        )
        .command(
            CustomCommand::with_name("dev2")
                .subcommand(CallApiCommand)
                .subcommand(CallContractCommand)
                .subcommand(CompileCommand)
                .subcommand(SetConcurrencyLevelCommand)
                .subcommand(GetConcurrencyLevelCommand)
                .subcommand(DeployCommand)
                .subcommand(GetCoinCommand)
                .subcommand(MoveExplain)
                .subcommand(GenBlockCommand)
                .subcommand(PackageCmd)
                .subcommand(PanicCommand)
                .subcommand(ResolveCommand)
                .subcommand(SleepCommand),
        )
}
