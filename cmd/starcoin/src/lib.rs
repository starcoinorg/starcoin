// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod account;
pub mod chain;
pub mod cli_state;
mod cli_state_vm2;
pub mod dev;
pub mod helper;
pub mod mutlisig_transaction;
pub mod mutlisig_transaction_vm2;
pub mod node;
pub mod state;
pub mod txpool;
pub mod view;
pub mod view_vm2;

pub use cli_state::CliState;
use scmd::{CmdContext, CustomCommand};
pub use starcoin_config::StarcoinOpt;
pub use starcoin_node::crash_handler;

pub fn add_command(
    context: CmdContext<CliState, StarcoinOpt>,
) -> CmdContext<CliState, StarcoinOpt> {
    context
        .command(
            CustomCommand::with_name("account")
                .subcommand(account::CreateCommand)
                .subcommand(account::ShowCommand)
                .subcommand(account::TransferCommand)
                .subcommand(account::ListCommand)
                .subcommand(account::ImportMultisigCommand)
                .subcommand(account::ChangePasswordCmd)
                .subcommand(account::DefaultCommand)
                .subcommand(account::RemoveCommand)
                .subcommand(account::LockCommand)
                .subcommand(account::UnlockCommand)
                .subcommand(account::ExportCommand)
                .subcommand(account::ImportCommand)
                .subcommand(account::ImportReadonlyCommand)
                .subcommand(account::ExecuteScriptFunctionCmd)
                .subcommand(account::ExecuteScriptCommand)
                .subcommand(account::GenerateMultisigTxnCommand)
                .subcommand(account::SubmitSignedTxnCommand)
                .subcommand(account::SignMessageCmd)
                .subcommand(account::VerifySignMessageCmd)
                .subcommand(account::DeriveAddressCommand)
                .subcommand(account::receipt_identifier_cmd::ReceiptIdentifierCommand)
                .subcommand(account::generate_keypair::GenerateKeypairCommand)
                .subcommand(account::rotate_auth_key_cmd::RotateAuthenticationKeyCommand),
            // TODO(BobOng): [multi-vm] to handle nft related command for vm2
            // .subcommand(account::nft_cmd::NFTCommand),
        )
        .command(
            CustomCommand::with_name("state")
                .subcommand(state::ListCmd)
                .subcommand(state::GetCommand)
                .subcommand(state::GetProofCommand)
                .subcommand(state::GetRootCommand),
        )
        .command(
            CustomCommand::with_name("node")
                .subcommand(node::InfoCommand)
                .subcommand(node::PeersCommand)
                .subcommand(node::MetricsCommand)
                .subcommand(node::manager::NodeManagerCommand)
                .subcommand(
                    CustomCommand::with_name("service")
                        .subcommand(node::service::ListCommand)
                        .subcommand(node::service::StartCommand)
                        .subcommand(node::service::CheckCommand)
                        .subcommand(node::service::StopCommand), //TODO support shutdown by command
                                                                 //.subcommand(node::service::ShutdownSystemCommand),
                )
                .subcommand(
                    CustomCommand::with_name("sync")
                        .subcommand(node::sync::StartCommand)
                        .subcommand(node::sync::StatusCommand)
                        .subcommand(node::sync::ProgressCommand)
                        .subcommand(node::sync::CancelCommand)
                        .subcommand(node::sync::PeerScoreCommand),
                )
                .subcommand(
                    CustomCommand::with_name("network")
                        .subcommand(node::network::StateCommand)
                        .subcommand(node::network::KnownPeersCommand)
                        .subcommand(node::network::GetAddressCommand)
                        .subcommand(node::network::AddPeerCommand)
                        .subcommand(node::network::CallPeerCommand)
                        .subcommand(node::network::SetPeerReputation)
                        .subcommand(node::network::BanPeerCommand),
                ),
        )
        .command(
            CustomCommand::with_name("chain")
                .subcommand(chain::InfoCommand)
                .subcommand(chain::GetBlockCommand)
                .subcommand(chain::ListBlockCommand)
                .subcommand(chain::GetTransactionCommand)
                .subcommand(chain::GetTxnInfosCommand)
                .subcommand(chain::GetTransactionInfoCommand)
                .subcommand(chain::GetEventsCommand)
                .subcommand(chain::EpochInfoCommand)
                .subcommand(chain::GetTransactionInfoListCommand)
                .subcommand(chain::GetTransactionProofCommand)
                .subcommand(chain::GetBlockInfoCommand),
        )
        .command(
            CustomCommand::with_name("txpool")
                .subcommand(txpool::PendingTxnCommand)
                .subcommand(txpool::PendingTxnsCommand)
                .subcommand(txpool::TxPoolStatusCommand),
        )
        .command(
            CustomCommand::with_name("dev")
                .subcommand(dev::GetCoinCommand)
                .subcommand(dev::MinimalTxnsEachBlockCommand)
                .subcommand(dev::MoveExplain)
                .subcommand(dev::DeployCommand)
                .subcommand(dev::CallContractCommand)
                .subcommand(dev::ResolveCommand)
                .subcommand(dev::CallApiCommand)
                .subcommand(
                    CustomCommand::with_name("subscribe")
                        .with_about("Subscribe the chain events")
                        .subcommand(dev::SubscribeNewMintBlockCommand)
                        .subcommand(dev::SubscribeBlockCommand)
                        .subcommand(dev::SubscribeEventCommand)
                        .subcommand(dev::SubscribeNewTxnCommand),
                )
                .subcommand(
                    CustomCommand::with_name("log")
                        .with_about("Set node's log level and pattern.")
                        .subcommand(dev::LogLevelCommand)
                        .subcommand(dev::LogPatternCommand),
                )
                .subcommand(dev::PanicCommand)
                .subcommand(dev::SleepCommand)
                .subcommand(dev::GenBlockCommand)
                .subcommand(dev::SetConcurrencyLevelCommand)
                .subcommand(dev::GetConcurrencyLevelCommand)
                .subcommand(dev::SetLoggerBalanceAmoutCommand)
                .subcommand(dev::GetLoggerBalanceAmountCommand)
                .subcommand(dev::UpgradeModuleExeCommand)
                .subcommand(dev::UpgradeModulePlanCommand)
                .subcommand(dev::UpgradeModuleProposalCommand)
                .subcommand(dev::UpgradeModuleQueueCommand)
                .subcommand(dev::UpgradeVMConfigProposalCommand),
        )
}
