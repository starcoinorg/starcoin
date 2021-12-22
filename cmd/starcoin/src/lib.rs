// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod account;
pub mod chain;
pub mod cli_state;
pub mod contract;
pub mod dev;
pub mod helper;
pub mod mutlisig_transaction;
pub mod node;
pub mod state;
mod txpool;
pub mod view;

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
                .subcommand(account::ChangePasswordCmd)
                .subcommand(account::DefaultCommand)
                .subcommand(account::remove_cmd::RemoveCommand)
                .subcommand(account::LockCommand)
                .subcommand(account::UnlockCommand)
                .subcommand(account::ExportCommand)
                .subcommand(account::ImportCommand)
                .subcommand(account::import_readonly_cmd::ImportReadonlyCommand)
                .subcommand(account::ExecuteScriptFunctionCmd)
                .subcommand(account::ExecuteScriptCommand)
                .subcommand(account::sign_multisig_txn_cmd::GenerateMultisigTxnCommand)
                .subcommand(account::submit_txn_cmd::SubmitSignedTxnCommand)
                .subcommand(account::SignMessageCmd)
                .subcommand(account::VerifySignMessageCmd)
                .subcommand(account::DeriveAddressCommand)
                .subcommand(account::receipt_identifier_cmd::ReceiptIdentifierCommand)
                .subcommand(account::generate_keypair::GenerateKeypairCommand)
                .subcommand(account::nft_cmd::NFTCommand),
        )
        .command(
            Command::with_name("state")
                .subcommand(state::ListCmd)
                .subcommand(state::GetCommand)
                .subcommand(state::GetProofCommand)
                .subcommand(state::GetRootCommand),
        )
        .command(
            Command::with_name("node")
                .subcommand(node::InfoCommand)
                .subcommand(node::PeersCommand)
                .subcommand(node::MetricsCommand)
                .subcommand(node::manager::NodeManagerCommand)
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
                    .subcommand(node::network::SetPeerReputation)
                    .subcommand(node::network::BanPeerCommand)

            ),
        )
        .command(
            Command::with_name("chain")
                .subcommand(chain::InfoCommand)
                .subcommand(chain::GetBlockCommand)
                .subcommand(chain::ListBlockCommand)
                .subcommand(chain::GetTransactionCommand)
                .subcommand(chain::GetTxnInfosCommand)
                .subcommand(chain::GetTransactionInfoCommand)
                .subcommand(chain::GetEventsCommand)
                .subcommand(chain::EpochInfoCommand)
                .subcommand(chain::GetTransactionInfoListCommand)
                .subcommand(chain::get_txn_proof_cmd::GetTransactionProofCommand)
                .subcommand(chain::GetBlockInfoCommand),
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
                .subcommand(dev::move_explain::MoveExplain)
                .subcommand(dev::CompileCommand)
                .subcommand(dev::DeployCommand)
                .subcommand(dev::UpgradeModuleProposalCommand)
                .subcommand(dev::UpgradeModulePlanCommand)
                .subcommand(dev::UpgradeModuleQueueCommand)
                .subcommand(dev::UpgradeModuleExeCommand)
                .subcommand(dev::UpgradeVMConfigProposalCommand)
                .subcommand(dev::PackageCmd)
                .subcommand(dev::CallContractCommand)
                .subcommand(dev::resolve_cmd::ResolveCommand)
                .subcommand(dev::call_api_cmd::CallApiCommand)
                .subcommand(
                    Command::with_name("subscribe")
                        .with_about("Subscribe the chain events")
                        .subcommand(dev::SubscribeBlockCommand)
                        .subcommand(dev::SubscribeEventCommand)
                        .subcommand(dev::SubscribeNewTxnCommand),
                )
                .subcommand(
                    Command::with_name("log")
                        .with_about("Set node's log level and pattern.")
                        .subcommand(dev::log_cmd::LogLevelCommand)
                        .subcommand(dev::log_cmd::LogPatternCommand),
                )
                .subcommand(dev::panic_cmd::PanicCommand)
                .subcommand(dev::sleep_cmd::SleepCommand)
                .subcommand(dev::gen_block_cmd::GenBlockCommand),
        )
        .command(Command::with_name("contract").subcommand(contract::GetContractDataCommand))
}
