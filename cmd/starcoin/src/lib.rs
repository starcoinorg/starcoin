// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod account;
pub mod chain;
pub mod cli_state;
pub mod contract;
pub mod debug;
pub mod dev;
pub mod helper;
pub mod mutlisig_transaction;
pub mod node;
pub mod state;
mod txpool;
pub mod view;

use crate::debug::{GenBlockCommand, SleepCommand, TxPoolStatusCommand};
pub use cli_state::CliState;
use scmd::{CmdContext, Command};
pub use starcoin_config::StarcoinOpt;
pub use starcoin_node::crash_handler;

pub fn add_command(
    context: CmdContext<CliState, StarcoinOpt>,
) -> CmdContext<CliState, StarcoinOpt> {
    context
        .command(
            Command::with_name("account")
                .subcommand(account::CreateCommand)
                .subcommand(account::ShowCommand)
                .subcommand(account::TransferCommand)
                .subcommand(account::AcceptTokenCommand)
                .subcommand(account::ListCommand)
                .subcommand(account::import_multisig_cmd::ImportMultisigCommand)
                .subcommand(account::sign_multisig_txn_cmd::GenerateMultisigTxnCommand)
                .subcommand(account::submit_multisig_txn_cmd::SubmitMultiSignedTxnCommand)
                .subcommand(account::UnlockCommand)
                .subcommand(account::ExportCommand)
                .subcommand(account::ImportCommand)
                .subcommand(account::ExecuteScriptFunctionCmd)
                .subcommand(account::ExecuteScriptCommand)
                .subcommand(account::LockCommand)
                .subcommand(account::ChangePasswordCmd)
                .subcommand(account::SignMessageCmd)
                .subcommand(account::VerifySignMessageCmd)
                .subcommand(account::DefaultCommand)
                .subcommand(account::DeriveAddressCommand)
                .subcommand(account::receipt_identifier_cmd::ReceiptIdentifierCommand),
        )
        .command(
            Command::with_name("state")
                .subcommand(state::ListResourceCmd)
                .subcommand(state::GetCommand)
                .subcommand(state::GetProofCommand)
                .subcommand(state::GetRootCommand),
        )
        .command(
            Command::with_name("node")
                .subcommand(node::InfoCommand)
                .subcommand(node::PeersCommand)
                .subcommand(node::MetricsCommand)
                .subcommand(
                    Command::with_name("service")
                        .subcommand(node::service::ListCommand)
                        .subcommand(node::service::StartCommand)
                        .subcommand(node::service::CheckCommand)
                        .subcommand(node::service::StopCommand)
                    //TODO support shutdown by command    
                    //.subcommand(node::service::ShutdownSystemCommand),
                )
                .subcommand(
                    Command::with_name("sync")
                        .subcommand(node::sync::StartCommand)
                        .subcommand(node::sync::StatusCommand)
                        .subcommand(node::sync::ProgressCommand)
                        .subcommand(node::sync::CancelCommand)
                        .subcommand(node::sync::PeerScoreCommand)
                )
                .subcommand(
                Command::with_name("network")
                    .subcommand(node::network::StateCommand)
                    .subcommand(node::network::KnownPeersCommand)
                    .subcommand(node::network::GetAddressCommand)
                    .subcommand(node::network::AddPeerCommand)
                    .subcommand(node::network::CallPeerCommand)
            ),
        )
        .command(
            Command::with_name("chain")
                .subcommand(chain::InfoCommand)
                .subcommand(chain::GetBlockByNumberCommand)
                .subcommand(chain::ListBlockCommand)
                .subcommand(chain::GetTransactionCommand)
                .subcommand(chain::GetTxnByBlockCommand)
                .subcommand(chain::GetTransactionInfoCommand)
                .subcommand(chain::GetEventsCommand)
                .subcommand(chain::GetBlockCommand)
                .subcommand(chain::EpochInfoCommand)
                .subcommand(chain::GetEpochInfoByNumberCommand)
                .subcommand(chain::GetGlobalTimeByNumberCommand)
                .subcommand(chain::TPSCommand)
                .subcommand(
                    Command::with_name("uncle")
                        .subcommand(chain::uncle::UnclePathCommand)
                        .subcommand(chain::uncle::ListEpochUnclesByNumberCommand)
                        .subcommand(chain::uncle::EpochUncleSummaryByNumberCommand),
                )
                .subcommand(
                    Command::with_name("stat")
                        .subcommand(chain::StatTPSCommand)
                        .subcommand(chain::StatEpochCommand)
                        .subcommand(chain::StatBlockCommand),
                )
                .subcommand(
                    Command::with_name("verify")
                        .subcommand(chain::VerifyBlockCommand)
                        .subcommand(chain::VerifyEpochCommand)
                        .subcommand(chain::VerifyNodeCommand),
                ),
        )
        .command(
            Command::with_name("txpool")
                .subcommand(txpool::PendingTxnCommand)
                .subcommand(txpool::PendingTxnsCommand)
                .subcommand(txpool::TxPoolStatusCommand),
        )
        .command(
            Command::with_name("dev")
                .subcommand(dev::GetCoinCommand)
                .subcommand(dev::CompileCommand)
                .subcommand(dev::DeployCommand)
                .subcommand(dev::UpgradeModuleProposalCommand)
                .subcommand(dev::UpgradeModulePlanCommand)
                .subcommand(dev::UpgradeModuleQueueCommand)
                .subcommand(dev::UpgradeModuleExeCommand)
                .subcommand(dev::UpgradeVMConfigProposalCommand)
                .subcommand(dev::PackageCmd)
                .subcommand(dev::CallContractCommand)
                .subcommand(
                    Command::with_name("subscribe")
                        .subcommand(dev::SubscribeBlockCommand)
                        .subcommand(dev::SubscribeEventCommand)
                        .subcommand(dev::SubscribeNewTxnCommand),
                ),
        )
        .command(Command::with_name("contract").subcommand(contract::GetContractDataCommand))
        .command(
            Command::with_name("debug")
                .subcommand(
                    Command::with_name("log")
                        .subcommand(debug::LogLevelCommand)
                        .subcommand(debug::LogPatternCommand),
                )
                .subcommand(debug::TxFactoryCommand)
                .subcommand(debug::PanicCommand)
                .subcommand(debug::GetBlockByUncleCommand)
                .subcommand(TxPoolStatusCommand)
                .subcommand(SleepCommand)
                .subcommand(GenBlockCommand)
                .subcommand(debug::MoveExplain),
        )
}
