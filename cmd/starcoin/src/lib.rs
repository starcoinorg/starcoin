// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod chain;
pub mod cli_state;
pub mod crash_handler;
pub mod debug;
pub mod dev;
pub mod helper;
pub mod mutlisig_transaction;
pub mod node;
pub mod state;
pub mod view;
pub mod wallet;
pub use cli_state::CliState;
use scmd::{CmdContext, Command};
pub use starcoin_config::StarcoinOpt;

pub fn add_command(
    context: CmdContext<CliState, StarcoinOpt>,
) -> CmdContext<CliState, StarcoinOpt> {
    context
        .command(
            Command::with_name("wallet")
                .subcommand(wallet::CreateCommand)
                .subcommand(wallet::ShowCommand)
                .subcommand(wallet::TransferCommand)
                .subcommand(wallet::AcceptCoinCommand)
                .subcommand(wallet::ListCommand)
                .subcommand(wallet::PartialSignTxnCommand)
                .subcommand(wallet::UnlockCommand)
                .subcommand(wallet::ExportCommand)
                .subcommand(wallet::ImportCommand)
                .subcommand(wallet::ExecuteBuildInCommand),
        )
        .command(
            Command::with_name("state")
                .subcommand(state::GetCommand)
                .subcommand(state::GetAccountCommand)
                .subcommand(state::GetProofCommand)
                .subcommand(state::GetRootCommand),
        )
        .command(
            Command::with_name("node")
                .subcommand(node::InfoCommand)
                .subcommand(node::PeersCommand)
                .subcommand(node::MetricsCommand),
        )
        .command(
            Command::with_name("chain")
                .subcommand(chain::ShowCommand)
                .subcommand(chain::GetBlockByNumberCommand)
                .subcommand(chain::ListBlockCommand)
                .subcommand(chain::GetTransactionCommand)
                .subcommand(chain::GetTxnByBlockCommand)
                .subcommand(chain::GetTransactionInfoCommand)
                .subcommand(chain::GetEventsCommand)
                .subcommand(chain::GetBlockCommand)
                .subcommand(chain::BranchesCommand),
        )
        .command(
            Command::with_name("dev")
                .subcommand(dev::GetCoinCommand)
                .subcommand(dev::CompileCommand)
                .subcommand(dev::DeployCommand)
                .subcommand(dev::ExecuteCommand)
                .subcommand(dev::DeriveAddressCommand)
                .subcommand(dev::GenerateMultisigTxnCommand)
                .subcommand(dev::ExecuteMultiSignedTxnCommand)
                .subcommand(dev::UpgradeStdlibCommand)
                .subcommand(
                    Command::with_name("subscribe")
                        .subcommand(dev::SubscribeBlockCommand)
                        .subcommand(dev::SubscribeEventCommand)
                        .subcommand(dev::SubscribeNewTxnCommand),
                ),
        )
        .command(
            Command::with_name("debug")
                .subcommand(
                    Command::with_name("log")
                        .subcommand(debug::LogLevelCommand)
                        .subcommand(debug::LogPatternCommand),
                )
                .subcommand(debug::GenTxnCommand)
                .subcommand(debug::PanicCommand),
        )
}
